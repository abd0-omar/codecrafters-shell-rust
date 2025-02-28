use crate::utils::{locate_command_in_paths, PathAndType};
use std::sync::OnceLock;
use std::{collections::HashSet, env};

pub enum MyShellCommand {
    Exit(u8),
    Echo(String),
    Type(Result<PathAndType, String>),
    ExternalProgram(ExternalProgramNameAndArgs),
    Invalid,
}

static BUILT_IN_COMMANDS: OnceLock<HashSet<&'static str>> = OnceLock::new();

fn get_built_in_commands() -> &'static HashSet<&'static str> {
    BUILT_IN_COMMANDS.get_or_init(|| {
        let mut hs = HashSet::new();
        hs.insert("exit");
        hs.insert("echo");
        hs.insert("type");
        hs
    })
}

#[derive(Debug)]
pub struct ExternalProgramNameAndArgs {
    pub name: String,
    pub args: Vec<String>,
}

impl MyShellCommand {
    pub fn try_parse(input: &str) -> Self {
        let ShellCommand {
            name: command_name,
            args,
        } = parse_command_args(input);

        let path_env = env::var("PATH").ok();

        match command_name.as_str() {
            "exit" => Self::parse_exit(&args.first()),
            "echo" => Self::parse_echo(&args),
            "type" => Self::parse_type(&args, &path_env),
            _ => Self::parse_external_programs(&command_name, &args, &path_env),
        }
    }

    fn parse_echo(args: &[String]) -> Self {
        Self::Echo(args.join(" "))
    }

    fn parse_external_programs(command_name: &str, args: &[String], path: &Option<String>) -> Self {
        if let Some(path) = path {
            locate_command_in_paths(&path, command_name, Some(args))
                .unwrap_or_else(|_| Self::Invalid)
        } else {
            Self::Invalid
        }
    }

    fn parse_exit(code: &Option<&String>) -> Self {
        match code.map(|code| {
            code.parse::<u8>()
                .map(Self::Exit)
                .unwrap_or_else(|_| Self::Invalid)
        }) {
            Some(command) => command,
            None => Self::Invalid,
        }
    }

    fn parse_type(arg: &[String], path: &Option<String>) -> Self {
        let command_name = &arg[0];

        if get_built_in_commands().contains(command_name.as_str()) {
            return Self::Type(Ok(PathAndType {
                path: None,
                command: command_name.to_string(),
            }));
        } else {
            if let Some(path) = path {
                locate_command_in_paths(path, &command_name, None)
                    .unwrap_or_else(|_| Self::Type(Err(command_name.to_string())))
            } else {
                Self::Invalid
            }
        }
    }
}

#[derive(thiserror::Error, Debug)]
pub enum ShellErrors {
    #[error("File is not in $PATH")]
    FileNotFoundInPath,
    #[error("No executable files were found in $PATH")]
    NoFilesInPATH,
}

struct ShellCommand {
    name: String,
    args: Vec<String>,
}

fn parse_command_args(input: &str) -> ShellCommand {
    let (mut in_single_quote, mut in_double_quote) = (false, false);
    let mut handle_backslash = false;
    let mut cur_arg = String::new();
    let mut total_args: Vec<String> = Vec::new();
    let mut idx = 0;

    while idx < input.len() {
        let c = input.as_bytes()[idx] as char;
        if handle_backslash {
            if in_single_quote {
                cur_arg.push('\\');
                cur_arg.push(c);
            } else if in_double_quote {
                // escape certain symbols
                if ['\\', '$', '"'].contains(&c) {
                    cur_arg.push(c);
                } else {
                    cur_arg.push('\\');
                    cur_arg.push(c);
                }
            } else {
                cur_arg.push(c);
            }
            handle_backslash = false;
        } else {
            match c {
                '"' => {
                    if in_single_quote {
                        cur_arg.push(c);
                    } else {
                        in_double_quote = !in_double_quote;
                    }
                }
                '\'' => {
                    if in_double_quote {
                        cur_arg.push(c);
                    } else {
                        in_single_quote = !in_single_quote;
                    }
                }
                ' ' => {
                    if in_single_quote || in_double_quote {
                        cur_arg.push(c);
                    } else {
                        if !cur_arg.is_empty() {
                            total_args.push(cur_arg);
                            cur_arg = String::new();
                        }
                    }
                }
                '\\' => {
                    handle_backslash = true;
                }
                _ => cur_arg.push(c),
            }
        }
        idx += 1;
    }

    if !cur_arg.is_empty() {
        total_args.push(cur_arg);
    }

    let (command_name, args) = total_args.split_at(1);
    ShellCommand {
        name: command_name[0].clone(),
        args: args.to_vec(),
    }
}
