mod command;

use command::MyShellCommand;
use std::io::{self, Write};
use std::process::{Command, ExitCode};

fn main() -> ExitCode {
    loop {
        print!("$ ");
        io::stdout().flush().unwrap();
        let stdin = io::stdin();
        let mut input = String::new();
        stdin.read_line(&mut input).unwrap();

        let input_parts: Vec<_> = input.split_whitespace().collect();

        let command = MyShellCommand::try_parse(&input_parts);

        match command {
            MyShellCommand::Exit(0) => {
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
                // dbg!("external!!");
                // dbg!(&external_program);
                // Command::new(external_program.name)
                //     .args(external_program.args)
                //     .spawn()
                //     .unwrap()
                //     .wait()
                //     .unwrap();
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
                // io::stdout().flush().unwrap();
            }
            MyShellCommand::Exit(_) | MyShellCommand::Invalid(_) => {
                println!("{}: command not found", input.trim_end());
                io::stdout().flush().unwrap();
            }
        }
    }
}
