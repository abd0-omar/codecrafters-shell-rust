mod command;
mod input_handler;

use command::{MyShellCommand, Trie};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode};
use input_handler::read_line_with_tab_detection;
use std::io::{self, Write};
use std::{
    fs,
    process::{Command, ExitCode},
};

fn initialize_trie(trie: &mut Trie) {
    // could be added to a sqlite db
    for path in std::env::var("PATH").unwrap_or_default().split(':') {
        if let Ok(entries) = fs::read_dir(path) {
            for entry in entries.filter_map(Result::ok) {
                if let Some(command_file) = entry.file_name().to_str() {
                    trie.insert(command_file);
                }
            }
        }
    }
    trie.insert("exit");
}

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
