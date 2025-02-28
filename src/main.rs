mod command;
mod input_handler;
mod trie;

use command::MyShellCommand;
use crossterm::terminal::{disable_raw_mode, enable_raw_mode};
use input_handler::read_line_with_tab_detection;
use std::io::{self, Write};
use std::process::{Command, ExitCode};
use trie::{initialize_trie, Trie};

fn main() -> ExitCode {
    let mut trie = Trie::new();
    initialize_trie(&mut trie);

    loop {
        enable_raw_mode().unwrap();
        let stdout = io::stdout();
        let mut stdout = stdout.lock();

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
