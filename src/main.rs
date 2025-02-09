use std::io::{self, Write};
use std::{env, fmt, fs, process::ExitCode};

#[derive(Debug)]
enum Command {
    Exit(u8),
    Echo(String),
    Type(Result<PathAndType, String>),
    Invalid(String),
}

#[derive(Debug)]
struct PathAndType {
    path: Option<String>,
    command: String,
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
    fn try_parse(input_parts: &[&str]) -> Result<Self, ShellErrors> {
        match input_parts {
            ["exit", code] => {
                let code = code
                    .parse::<u8>()
                    .map_err(|_| ShellErrors::TypeInvalidCommand)?;
                Ok(Self::Exit(code))
            }
            // to help `Type` command
            ["exit"] => Ok(Self::Exit(42)),
            ["echo", arg @ ..] => Ok(Self::Echo(arg.join(" "))),
            ["type", arg @ ..] => {
                let arg = Self::try_parse(arg);
                match &arg {
                    Ok(command) => match command {
                        Command::Invalid(_invalid_type) => {
                            // maybe it's in the path env var
                            ()
                        }
                        _ => {
                            return Ok(Self::Type(Ok(PathAndType {
                                path: None,
                                command: command.to_string(),
                            })))
                        }
                    },
                    Err(_) => return Err(ShellErrors::TypeInvalidCommand),
                }
                if let Ok(path) = env::var("PATH") {
                    match Command::find_in_user_paths(path, &arg) {
                        Ok(command) => {
                            return Ok(command);
                        }
                        Err(err) => match err {
                            ShellErrors::TypeInvalidCommand => return Err(err),
                            ShellErrors::FileNotFoundInPath | ShellErrors::InvalidCommand => {
                                return Ok(Self::Type(Err(arg.unwrap().to_string())));
                            }
                        },
                    }
                }
                unreachable!("assume user always have env var $PATH")
            }
            _ => Ok(Self::Invalid(input_parts.join(" "))),
        }
    }

    fn find_in_user_paths(
        path: String,
        arg: &Result<Command, ShellErrors>,
    ) -> Result<Self, ShellErrors> {
        for path_part in path.split(':') {
            for entry in
                fs::read_dir(path_part).map_err(|_e| return ShellErrors::FileNotFoundInPath)?
            {
                let entry = entry.unwrap();
                let path = entry.path();
                if let Some(command_file) = path.file_name() {
                    match arg {
                        Ok(ref command) => match &command {
                            Command::Invalid(_invalid_type) => {
                                if command.to_string() == command_file.to_str().unwrap().to_string()
                                {
                                    return Ok(Self::Type(Ok(PathAndType {
                                        path: Some(path.to_str().unwrap().to_owned()),
                                        command: command.to_string(),
                                    })));
                                }
                                continue;
                            }
                            _ => {
                                if command.to_string() == command_file.to_str().unwrap().to_string()
                                {
                                    return Ok(Self::Type(Ok(PathAndType {
                                        path: Some(path.to_str().unwrap().to_owned()),
                                        command: command.to_string(),
                                    })));
                                } else {
                                    continue;
                                }
                            }
                        },
                        Err(_) => {
                            return Err(ShellErrors::TypeInvalidCommand);
                        }
                    }
                }
            }
        }
        return Err(ShellErrors::InvalidCommand);
    }
}

#[derive(thiserror::Error, Debug)]
enum ShellErrors {
    #[error("not found")]
    TypeInvalidCommand,
    #[error("didn't find the file")]
    FileNotFoundInPath,
    #[error("invliad commandz")]
    InvalidCommand,
}

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
                            let PathAndType { path, command } = valid_type;
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
            },
            Err(_) => {
                println!("{}: command not found", input.trim_end());
                io::stdout().flush().unwrap();
            }
        }
    }
}
