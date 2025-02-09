mod command;

use command::Command;
use std::io::{self, Write};
use std::process::ExitCode;

fn main() -> ExitCode {
    loop {
        print!("$ ");
        io::stdout().flush().unwrap();
        let stdin = io::stdin();
        let mut input = String::new();
        stdin.read_line(&mut input).unwrap();

        let input_parts: Vec<_> = input.split_whitespace().collect();

        let command = Command::try_parse(&input_parts);

        match command {
            Command::Exit(0) => {
                return ExitCode::SUCCESS;
            }
            Command::Echo(arg) => {
                println!("{}", arg);
                io::stdout().flush().unwrap();
            }
            Command::Type(arg) => {
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
            Command::Exit(_) | Command::Invalid(_) => {
                println!("{}: command not found", input.trim_end());
                io::stdout().flush().unwrap();
            }
        }
    }
}
