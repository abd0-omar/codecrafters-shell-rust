use std::{env, fmt, fs};

pub enum MyShellCommand {
    Exit(u8),
    Echo(Result<String, String>),
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
    pub fn try_parse(input: &str) -> Self {
        let input_parts: Vec<_> = input.split_whitespace().collect();

        match input_parts.as_slice() {
            ["exit", code] => Self::parse_exit(code),
            // to help `Type` command
            ["exit"] => Self::Exit(42),
            ["echo", ref arg @ ..] => Self::parse_echo(arg, input),
            ["type", ref arg @ ..] => Self::parse_type(arg),
            [ref arg @ ..] => Self::parse_external_programs(arg, input),
        }
    }
    fn parse_echo(arg: &[&str], input: &str) -> Self {
        let arg = arg.join(" ");
        // it could not start with '\'' and end with '\\\'' and still be valid,
        // leave today's work for tomorrow
        if arg.starts_with('\'') && !arg.ends_with('\'')
            || !arg.starts_with('\'') && arg.ends_with('\'')
        {
            Self::Echo(Err(arg))
        } else if arg.starts_with('\'') && arg.ends_with('\'') {
            Self::Echo(Ok(Self::single_quotes_parser(input).join(" ")))
        } else {
            Self::Echo(Ok(arg))
        }
    }

    fn single_quotes_parser(input: &str) -> Vec<String> {
        // handle quote in the middle
        // r#"
        // 'it\'s me"
        // #
        let mut result: Vec<String> = Vec::new();
        let mut current = String::new();
        let mut in_quotes = false;
        let mut chars = input.chars().peekable();
        while let Some(ch) = chars.next() {
            match ch {
                '\\' => {
                    if let Some(next_char) = chars.next() {
                        current.push(next_char);
                    }
                }
                '\'' => {
                    if let Some(&next_char) = chars.peek() {
                        if next_char == '\'' {
                            chars.next();
                            continue;
                        }
                    }
                    if in_quotes {
                        result.push(current.clone());
                        current.clear();
                    }
                    in_quotes = !in_quotes;
                }
                _ => {
                    if in_quotes {
                        current.push(ch);
                    }
                }
            }
        }

        // if !cur.is_empty, you could add it to result
        result
    }

    fn parse_external_programs(arg: &[&str], input: &str) -> Self {
        if let Ok(path) = env::var("PATH") {
            if let Some(program_name) = arg.first() {
                MyShellCommand::locate_command_in_paths(&path, program_name, arg, input)
                    .unwrap_or_else(|_| Self::Invalid(arg.join(" ")))
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
        let command = Self::try_parse(&arg.join(" "));
        match &command {
            MyShellCommand::Invalid(_) | MyShellCommand::ExternalProgram(_) => {
                // maybe it's in the path env var
                if let Ok(path) = env::var("PATH") {
                    MyShellCommand::locate_command_type_in_paths(&path, &command.to_string())
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

    pub fn locate_command_in_paths(
        path: &str,
        name: &str,
        arg: &[&str],
        input: &str,
    ) -> Result<Self, ShellErrors> {
        for path_part in path.split(':') {
            for entry in fs::read_dir(path_part).map_err(|_| ShellErrors::FileNotFoundInPath)? {
                let entry = entry.unwrap();
                let path = entry.path();
                if let Some(command_file) = path.file_name() {
                    if name == command_file.to_str().unwrap() {
                        if name == "cat" {
                            let input_without_cat = input.strip_prefix("cat").unwrap().trim();
                            if input_without_cat.starts_with('\'')
                                && input_without_cat.ends_with('\'')
                            {
                                return Ok(Self::ExternalProgram(ExternalProgramNameAndArgs {
                                    name: name.to_owned(),
                                    args: Self::single_quotes_parser(input_without_cat),
                                }));
                            }
                        }
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

    pub fn locate_command_type_in_paths(path: &str, arg: &str) -> Result<Self, ShellErrors> {
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
