use std::{env, fmt, fs};

pub enum Command {
    Exit(u8),
    Echo(String),
    Type(Result<PathAndType, String>),
    Invalid(String),
}

pub struct PathAndType {
    pub path: Option<String>,
    pub command: String,
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
    pub fn try_parse(input_parts: &[&str]) -> Self {
        match input_parts {
            ["exit", code] => Self::parse_exit(code),
            // to help `Type` command
            ["exit"] => Self::Exit(42),
            ["echo", arg @ ..] => Self::Echo(arg.join(" ")),
            ["type", arg @ ..] => Self::parse_type(arg),
            _ => Self::Invalid(input_parts.join(" ")),
        }
    }

    fn parse_exit(code: &str) -> Self {
        code.parse::<u8>()
            .map(Self::Exit)
            .unwrap_or_else(|_| Self::Invalid(format!("Invalid exit code: {}", code)))
    }

    fn parse_type(arg: &[&str]) -> Self {
        let command = Self::try_parse(arg);
        match &command {
            Command::Invalid(_) => {
                // maybe it's in the path env var
                if let Ok(path) = env::var("PATH") {
                    Command::find_in_user_paths(&path, &command.to_string())
                        .unwrap_or_else(|_| Self::Type(Err(command.to_string())))
                } else {
                    Self::Invalid(command.to_string())
                }
            }
            // built-in shell command
            _ => Self::Type(Ok(PathAndType {
                path: None,
                command: command.to_string(),
            })),
        }
    }

    pub fn find_in_user_paths(path: &str, arg: &str) -> Result<Self, ShellErrors> {
        for path_part in path.split(':') {
            for entry in fs::read_dir(path_part).map_err(|_| ShellErrors::FileNotFoundInPath)? {
                let entry = entry.unwrap();
                let path = entry.path();
                if let Some(command_file) = path.file_name() {
                    if arg == command_file.to_str().unwrap() {
                        return Ok(Self::Type(Ok(PathAndType {
                            path: Some(path.to_str().unwrap().to_owned()),
                            command: arg.to_owned(),
                        })));
                    }
                }
            }
        }
        // the for loop didn't start, so no files found in $PATH
        Err(ShellErrors::NoFilesInPATH)
    }
}

#[derive(thiserror::Error, Debug)]
pub enum ShellErrors {
    #[error("File is not in $PATH")]
    FileNotFoundInPath,
    #[error("No executable files were found in $PATH")]
    NoFilesInPATH,
}
