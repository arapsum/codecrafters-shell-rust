use std::path::{Path, PathBuf};

use crate::commands;

const BUILTINS: &[&str] = &["exit", "echo", "type", "pwd", "cd"];

#[derive(Debug, Clone)]
pub struct Command {
    pub name: String,
    pub path: Option<PathBuf>,
    pub args: Vec<String>,
}

impl Command {
    pub fn new(name: &str, path: Option<PathBuf>, args: Vec<String>) -> Self {
        Self {
            name: name.into(),
            path,
            args,
        }
    }

    pub fn run(&self) -> anyhow::Result<()> {
        std::process::Command::new(&self.name)
            .args(&self.args)
            .spawn()?
            .wait()?;
        Ok(())
    }
}

pub enum CommandType {
    Cd(Command),
    Echo(Command),
    Exit(i32),
    Programme(Command),
    Pwd(Command),
    Type(Command),
    Unkown(String),
}

pub fn parse_command(input: &str) -> CommandType {
    let words = shell_words::split(input).unwrap();

    let cmd = &words[0];
    let args: Vec<String> = words[1..].iter().map(ToString::to_string).collect();
    match cmd.as_str() {
        "cd" => CommandType::Cd(Command::new(cmd, None, args)),
        "echo" => CommandType::Echo(Command::new(cmd, None, args)),
        "exit" => {
            if args.is_empty() {
                CommandType::Exit(0)
            } else {
                let status = args[0].parse::<i32>().unwrap_or(1);
                CommandType::Exit(status)
            }
        }
        "pwd" => CommandType::Pwd(Command::new(
            cmd,
            Some(std::env::current_dir().unwrap_or_default()),
            args,
        )),
        "type" => CommandType::Type(Command::new(cmd, None, args)),
        _ => {
            if let Some(executable) = commands::find_executable(cmd) {
                let command = Command::new(cmd, Some(executable), args);
                CommandType::Programme(command)
            } else {
                CommandType::Unkown(format!("{cmd}: not found"))
            }
        }
    }
}

// fn tokeniser(input: &str) -> Vec<String> {
//     let mut tokens = Vec::new();
//     let mut current = String::new();
//     let mut in_single_quotes = false;
//     let mut is_double_quotes = false;
//
//     for c in input.chars() {
//         match c {
//             '\'' => {
//                 in_single_quotes = !in_single_quotes;
//             }
//             '"' => {
//                 is_double_quotes = !is_double_quotes;
//             }
//             ' ' if !in_single_quotes => {
//                 if !current.is_empty() {
//                     tokens.push(current.clone());
//                     current.clear();
//                 }
//             }
//             _ => current.push(c),
//         }
//     }
//
//     if !current.is_empty() {
//         tokens.push(current);
//     }
//
//     tokens
// }

pub fn is_builtin(command: &str) -> bool {
    BUILTINS.contains(&command)
}

pub fn find_executable(name: &str) -> Option<PathBuf> {
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

pub fn is_executable(path: &Path) -> bool {
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
