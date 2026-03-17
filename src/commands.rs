use std::{
    borrow::Cow,
    path::{Path, PathBuf},
};

use crate::commands;

const BUILTINS: &[&str] = &["exit", "echo", "type", "pwd", "cd"];

pub enum Command<'a> {
    Cd(PathBuf),
    Exit(Option<i32>),
    Echo(Cow<'a, str>),
    Type(Cow<'a, str>),
    Pwd(Cow<'a, str>),
    Programme(PathBuf, Cow<'a, str>, Vec<String>),
    Unkown(Cow<'a, str>),
}

pub fn parse_command(input: &str) -> Command<'_> {
    let input = input.trim();
    let mut parts = input.split_whitespace();

    let cmd = parts.next().unwrap();

    let mut args = parts;

    match cmd {
        "exit" => Command::Exit(args.next().map(|code| code.parse::<i32>().unwrap_or(0))),
        "echo" => {
            let tokens = tokeniser(input);

            let args: Vec<&str> = tokens.iter().skip(1).map(|s| s.as_str()).collect();

            Command::Echo(Cow::Owned(args.join(" ")))
        }
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
        "cd" => {
            let target = args.next().unwrap_or("~");

            let dir = if target.starts_with("~") {
                let home = std::env::home_dir().unwrap();
                let rest = target.trim_start_matches("~").trim_start_matches("/");
                home.join(rest)
            } else {
                PathBuf::from(target)
            };

            Command::Cd(dir)
        }
        _ => {
            if let Some(executable) = commands::find_executable(cmd) {
                let tokens = tokeniser(input);
                Command::Programme(
                    executable,
                    Cow::Owned(cmd.to_string()),
                    tokens.into_iter().skip(1).collect(),
                )
            } else {
                Command::Unkown(Cow::Owned(format!("{cmd}: not found")))
            }
        }
    }
}

fn tokeniser(input: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut current = String::new();
    let mut in_quotes = false;

    for c in input.chars() {
        match c {
            '\'' => {
                in_quotes = !in_quotes;
            }
            ' ' if !in_quotes => {
                if !current.is_empty() {
                    tokens.push(current.clone());
                    current.clear();
                }
            }
            _ => current.push(c),
        }
    }

    if !current.is_empty() {
        tokens.push(current);
    }

    tokens
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
