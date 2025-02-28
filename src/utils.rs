use std::fs;

use crate::command::{ExternalProgramNameAndArgs, MyShellCommand, ShellErrors};

pub fn locate_command_in_paths(
    path: &str,
    command_name: &str,
    args: Option<&[String]>,
) -> Result<MyShellCommand, ShellErrors> {
    for path_part in path.split(':') {
        for entry in fs::read_dir(path_part).map_err(|_| ShellErrors::FileNotFoundInPath)? {
            let entry = entry.unwrap();
            let path = entry.path();
            if let Some(command_file) = path.file_name() {
                if command_name == command_file.to_str().unwrap() {
                    if let Some(args) = args {
                        return Ok(MyShellCommand::ExternalProgram(
                            ExternalProgramNameAndArgs {
                                name: command_name.to_owned(),
                                args: args.to_vec(),
                            },
                        ));
                    } else {
                        return Ok(MyShellCommand::Type(Ok(PathAndType {
                            path: Some(path.to_str().unwrap().to_owned()),
                            command: command_name.to_string(),
                        })));
                    }
                }
            }
        }
    }
    Err(ShellErrors::NoFilesInPATH)
}

pub struct PathAndType {
    pub path: Option<String>,
    pub command: String,
}
