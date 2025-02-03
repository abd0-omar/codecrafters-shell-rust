#[allow(unused_imports)]
use std::io::{self, Write};
use std::process::ExitCode;

enum Command {
    Exit(u8),
    Other,
}

impl Command {
    fn try_parse(input_parts: &[&str]) -> Result<Self, InvalidCommand> {
        match input_parts {
            ["exit", code] => {
                let code = code.parse::<u8>().map_err(|_| InvalidCommand)?;
                Ok(Self::Exit(code))
            }
            ["exit"] => Err(InvalidCommand),
            _ => Ok(Self::Other),
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
                Command::Exit(_) | Command::Other => {
                    println!("{}: command not found", input.trim_end());
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
