use std::{env, fmt, fs};

pub enum MyShellCommand {
    Exit(u8),
    Echo(String),
    Type(Result<PathAndType, String>),
    ExternalProgram(ExternalProgramNameAndArgs),
    Invalid(String),
}

#[derive(Debug)]
pub struct ExternalProgramNameAndArgs {
    pub name: String,
    pub args: Vec<String>,
}

pub struct PathAndType {
    pub path: Option<String>,
    pub command: String,
}

impl fmt::Display for MyShellCommand {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MyShellCommand::Exit(_) => write!(f, "exit"),
            MyShellCommand::Echo(_) => write!(f, "echo"),
            MyShellCommand::Type(_) => write!(f, "type"),
            MyShellCommand::ExternalProgram(program_name) => write!(f, "{}", program_name.name),
            MyShellCommand::Invalid(arg) => write!(f, "{}", arg),
        }
    }
}

impl MyShellCommand {
    pub fn try_parse(input_parts: &[&str]) -> Self {
        match input_parts {
            ["exit", code] => Self::parse_exit(code),
            // to help `Type` command
            ["exit"] => Self::Exit(42),
            ["echo", arg @ ..] => Self::Echo(arg.join(" ")),
            ["type", arg @ ..] => {
                // dbg!(went_to_type);
                return Self::parse_type(arg);
            }
            // _ => Self::Invalid(input_parts.join(" ")),
            [arg @ ..] => Self::parse_external_programs(arg),
            // _ => Self::Invalid(input_parts.join(" ")),
        }
    }

    fn parse_external_programs(arg: &[&str]) -> Self {
        // dbg!(went_to_type);
        // dbg!("in parse_external_programs");
        // dbg!(&arg);
        if let Ok(path) = env::var("PATH") {
            if let Some(program_name) = arg.first() {
                // dbg!(program_name);
                MyShellCommand::find_in_user_paths_2(&path, program_name, arg).unwrap_or_else(
                    |_| {
                        // dbg!("ana feen");
                        // return Self::Invalid(arg.join(" "));
                        // return Self::Type(Err(program_name.to_string()));
                        return Self::ExternalProgram(ExternalProgramNameAndArgs {
                            name: program_name.to_string(),
                            args: arg.iter().map(|arg| arg.to_string()).collect::<Vec<_>>(),
                        });
                    },
                )
            } else {
                Self::Invalid(arg.join(" "))
            }
        } else {
            Self::Invalid(arg.join(" "))
        }
    }

    fn parse_exit(code: &str) -> Self {
        code.parse::<u8>()
            .map(Self::Exit)
            .unwrap_or_else(|_| Self::Invalid(format!("Invalid exit code: {}", code)))
    }

    fn parse_type(arg: &[&str]) -> Self {
        let command = Self::try_parse(arg);
        // dbg!("went in parse_type");
        // dbg!(&command.to_string());
        match &command {
            MyShellCommand::Invalid(_) | MyShellCommand::ExternalProgram(_) => {
                // maybe it's in the path env var
                if let Ok(path) = env::var("PATH") {
                    MyShellCommand::find_in_user_paths(&path, &command.to_string())
                        .unwrap_or_else(|_| Self::Type(Err(command.to_string())))
                } else {
                    Self::Invalid(command.to_string())
                }
            }
            // built-in shell command
            _ => {
                return Self::Type(Ok(PathAndType {
                    path: None,
                    command: command.to_string(),
                }));
            }
        }
    }

    pub fn find_in_user_paths_2(path: &str, name: &str, arg: &[&str]) -> Result<Self, ShellErrors> {
        // dbg!("in find_in_user_paths_2 fn");
        for path_part in path.split(':') {
            for entry in fs::read_dir(path_part).map_err(|_| ShellErrors::FileNotFoundInPath)? {
                let entry = entry.unwrap();
                let path = entry.path();
                if let Some(command_file) = path.file_name() {
                    if name == command_file.to_str().unwrap() {
                        // dbg!(&arg);
                        // dbg!(&command_file.to_str().unwrap());
                        // return Ok(Self::Type(Ok(PathAndType {
                        //     path: Some(path.to_str().unwrap().to_owned()),
                        //     command: arg.to_owned(),
                        // })));
                        // return Ok(Self::Invalid(arg.to_string()));
                        return Ok(Self::ExternalProgram(ExternalProgramNameAndArgs {
                            name: name.to_owned(),
                            args: arg
                                .iter()
                                .skip(1)
                                .map(|arg| arg.to_string())
                                .collect::<Vec<_>>(),
                        }));
                    }
                }
            }
        }
        // the for loop didn't start, so no files found in $PATH
        Err(ShellErrors::NoFilesInPATH)
    }

    pub fn find_in_user_paths(path: &str, arg: &str) -> Result<Self, ShellErrors> {
        for path_part in path.split(':') {
            for entry in fs::read_dir(path_part).map_err(|_| ShellErrors::FileNotFoundInPath)? {
                let entry = entry.unwrap();
                let path = entry.path();
                if let Some(command_file) = path.file_name() {
                    if arg == command_file.to_str().unwrap() {
                        // dbg!(&arg);
                        // dbg!(&command_file.to_str().unwrap());
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
