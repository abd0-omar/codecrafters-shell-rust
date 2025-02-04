#[allow(unused_imports)]
use std::io::{self, Write};
use std::{fmt, process::ExitCode};

enum Command {
    Exit(u8),
    Echo(String),
    Type(Result<String, String>),
    Invalid(String),
}

impl fmt::Display for Command {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Command::Exit(_) => write!(f, "exit"),
            Command::Echo(_) => write!(f, "echo"),
            Command::Type(_) => write!(f, "type"),
            Command::Invalid(arg) => write!(f, "{}", arg),
        }
    }
}

impl Command {
    fn try_parse(input_parts: &[&str]) -> Result<Self, InvalidCommand> {
        match input_parts {
            ["exit", code] => {
                let code = code.parse::<u8>().map_err(|_| InvalidCommand)?;
                Ok(Self::Exit(code))
            }
            // to help `Type` command
            ["exit"] => Ok(Self::Exit(42)),
            ["echo", arg @ ..] => Ok(Self::Echo(arg.join(" "))),
            ["type", arg @ ..] => {
                let arg = Self::try_parse(arg);
                match arg {
                    Ok(command) => match command {
                        Command::Invalid(invalid_type) => return Ok(Self::Type(Err(invalid_type))),
                        _ => return Ok(Self::Type(Ok(command.to_string()))),
                    },
                    Err(_) => return Err(InvalidCommand),
                }
            }
            _ => Ok(Self::Invalid(input_parts.join(" "))),
        }
    }
}

struct InvalidCommand;

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
            Ok(command) => match command {
                Command::Exit(0) => {
                    return ExitCode::SUCCESS;
                }
                Command::Exit(_) | Command::Invalid(_) => {
                    println!("{}: command not found", input.trim_end());
                    io::stdout().flush().unwrap();
                }
                Command::Echo(arg) => {
                    println!("{}", arg);
                    io::stdout().flush().unwrap();
                }
                Command::Type(arg) => {
                    match arg {
                        Ok(valid_type) => {
                            println!("{} is a shell builtin", valid_type);
                        }
                        Err(invalid_type) => {
                            println!("{}: not found", invalid_type);
                        }
                    }
                    io::stdout().flush().unwrap();
                }
            },
            Err(_) => {
                println!("{}: command not found", input.trim_end());
                io::stdout().flush().unwrap();
            }
        }
    }
}
