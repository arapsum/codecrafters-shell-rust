use std::{
    borrow::Cow,
    path::{Path, PathBuf},
    str::SplitWhitespace,
};

use crate::commands;

const BUILTINS: &[&str] = &["exit", "echo", "type", "pwd"];

pub enum Command<'a> {
    Exit(Option<i32>),
    Echo(Cow<'a, str>),
    Type(Cow<'a, str>),
    Pwd(Cow<'a, str>),
    Programme(PathBuf, Cow<'a, str>, SplitWhitespace<'a>),
    Unkown(Cow<'a, str>),
}

pub fn parse_command(input: &str) -> Command<'_> {
    let mut parts = input.trim().split_whitespace();
    let cmd = parts.next().unwrap_or("");
    let mut args = parts;

    match cmd {
        "exit" => Command::Exit(args.next().map(|code| code.parse::<i32>().unwrap_or(0))),
        "echo" => Command::Echo(Cow::Owned(args.collect::<Vec<&str>>().join(" "))),
        "pwd" => Command::Pwd(Cow::Owned(
            std::env::current_dir()
                .unwrap_or_default()
                .display()
                .to_string(),
        )),
        "type" => {
            let cmd_name = args.next().expect("type command requires a <NAME>");

            if is_builtin(cmd_name) {
                Command::Type(Cow::Owned(format!("{cmd_name} is a shell builtin")))
            } else if let Some(path) = find_executable(cmd_name) {
                Command::Type(Cow::Owned(format!("{cmd_name} is {}", path.display())))
            } else {
                Command::Unkown(Cow::Owned(format!("{cmd_name} not found")))
            }
        }
        _ => {
            if let Some(executable) = commands::find_executable(cmd) {
                Command::Programme(executable, Cow::Owned(cmd.to_string()), args)
            } else {
                Command::Unkown(Cow::Owned(format!("{cmd}: not found")))
            }
        }
    }
}

fn is_builtin(command: &str) -> bool {
    BUILTINS.contains(&command)
}

fn find_executable(name: &str) -> Option<PathBuf> {
    let path_var = std::env::var("PATH").ok()?;
    let delimiter = ":";

    for dir in path_var.split(delimiter) {
        let mut full_path = PathBuf::from(dir);
        full_path.push(name);
        if is_executable(&full_path) {
            return Some(full_path);
        }
    }

    None
}

fn is_executable(path: &Path) -> bool {
    if !path.exists() {
        return false;
    }

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;

        if let Ok(metadata) = std::fs::metadata(path) {
            let permissions = metadata.permissions();
            return permissions.mode() & 0o111 != 0;
        }
        false
    }
}
