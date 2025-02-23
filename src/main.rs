mod command;

use command::{MyShellCommand, Trie};
use crossterm::{
    cursor::MoveToColumn,
    event::{self, Event, KeyCode, KeyEvent, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, Clear, ClearType},
};
use std::io::{self, StdoutLock, Write};
use std::process::{Command, ExitCode};

fn main() -> ExitCode {
    let mut trie = Trie::new();

    loop {
        enable_raw_mode().unwrap();
        let stdout = io::stdout();
        let mut stdout = stdout.lock();
        // could be added to a sqlite db
        trie.insert("echo");
        trie.insert("exit");

        let input = match read_line_with_tab_detection(&mut stdout, &mut trie) {
            Ok(line) => line,
            Err(_) => continue,
        };

        let command = MyShellCommand::try_parse(&input);

        match command {
            MyShellCommand::Exit(0) => {
                disable_raw_mode().unwrap();
                return ExitCode::SUCCESS;
            }
            MyShellCommand::Echo(arg) => {
                println!("{}", arg);
                io::stdout().flush().unwrap();
            }
            MyShellCommand::Type(arg) => {
                match arg {
                    Ok(command::PathAndType { path, command }) => {
                        if let Some(path) = path {
                            println!("{} is {}", command, path);
                        } else {
                            println!("{} is a shell builtin", command);
                        }
                    }
                    Err(invalid_type) => {
                        println!("{}: not found", invalid_type);
                    }
                }
                io::stdout().flush().unwrap();
            }
            MyShellCommand::ExternalProgram(external_program) => {
                match Command::new(external_program.name)
                    .args(external_program.args)
                    .spawn()
                {
                    Ok(mut command) => {
                        command.wait().unwrap();
                    }
                    Err(_) => {
                        println!("{}: command not found", input.trim_end());
                        io::stdout().flush().unwrap();
                    }
                };
            }
            MyShellCommand::Exit(_) | MyShellCommand::Invalid(_) => {
                println!("{}: command not found", input.trim_end());
                io::stdout().flush().unwrap();
            }
        }
    }
}

fn read_line_with_tab_detection(
    stdout: &mut StdoutLock<'static>,
    trie: &mut Trie,
) -> io::Result<String> {
    enable_raw_mode().unwrap(); // Keep raw mode enabled
    let mut line = String::new();

    print!("$ ");
    io::stdout().flush().unwrap();

    loop {
        if let Event::Key(key_event) = event::read()? {
            match key_event {
                KeyEvent {
                    modifiers: KeyModifiers::CONTROL,
                    code: KeyCode::Char('j'),
                    ..
                } => {
                    print!("\r\n");
                    io::stdout().flush().unwrap();
                    break;
                }
                _ => (),
            }
            match key_event.code {
                KeyCode::Enter => {
                    print!("\r\n");
                    io::stdout().flush().unwrap();
                    break;
                }
                KeyCode::Tab => {
                    let words = trie.get_words_with_prefix(&line);
                    if !words.is_empty() {
                        execute!(stdout, MoveToColumn(0), Clear(ClearType::CurrentLine)).unwrap();
                        line = format!("{} ", words[0].clone());
                        print!("$ {}", line);
                        io::stdout().flush().unwrap();
                    } else {
                        print!("\x07");
                        io::stdout().flush().unwrap();
                    }
                }
                KeyCode::Backspace => {
                    if !line.is_empty() {
                        line.pop();
                        execute!(stdout, MoveToColumn(0), Clear(ClearType::CurrentLine)).unwrap();
                        print!("$ {}", line);
                        io::stdout().flush().unwrap();
                    }
                }
                KeyCode::Char(c) => {
                    line.push(c);
                    print!("{}", c);
                    io::stdout().flush().unwrap();
                }
                _ => {}
            }
        }
    }

    disable_raw_mode().unwrap();
    Ok(line)
}
