#[allow(unused_imports)]
use std::io::{self, Write};
use std::{
    borrow::Cow,
    path::{Path, PathBuf},
    str::SplitWhitespace,
};

const BUILTINS: &[&str] = &["exit", "echo", "type"];

fn main() {
    loop {
        print!("$ ");
        io::stdout().flush().unwrap();

        let mut input = String::new();
        io::stdin()
            .read_line(&mut input)
            .expect("Failed to read line");

        match parse_command(input.trim()) {
            Some(Command::Exit(code)) => {
                let exit_code = code.and_then(|code| code.parse::<i32>().ok()).unwrap_or(0);
                std::process::exit(exit_code);
            }
            Some(Command::Echo(text)) => {
                println!("{text}");
            }
            Some(Command::Type(file)) => println!("{file}"),
            Some(Command::NotFound(err)) => eprintln!("{err}"),
            Some(Command::Programme(_, name, args)) => {
                match std::process::Command::new(name.to_string())
                    .args(args)
                    .spawn()
                {
                    Ok(mut child) => {
                        let _ = child.wait();
                    }
                    Err(e) => eprintln!("{e}"),
                }
            }
            _ => {
                let mut parts = input.trim().split_whitespace();
                let name = parts.next().unwrap();

                eprintln!("{name} not found",);
            }
        }
    }
}

pub enum Command<'a> {
    Exit(Option<Cow<'a, str>>),
    Echo(Cow<'a, str>),
    Type(Cow<'a, str>),
    Programme(PathBuf, Cow<'a, str>, SplitWhitespace<'a>),
    NotFound(String),
}

fn parse_command(input: &str) -> Option<Command<'_>> {
    let mut parts = input.trim().split_whitespace();
    let command = parts.next()?;
    let args = parts;

    match command {
        "exit" => {
            let code = args.collect::<Vec<&str>>().get(0).copied();
            Some(Command::Exit(code.map(|code| Cow::Owned(code.into()))))
        }
        "echo" => {
            let message = args.collect::<Vec<&str>>().join(" ");
            Some(Command::Echo(Cow::Owned(message)))
        }
        "type" => {
            let file = args.collect::<Vec<&str>>().get(0).copied().unwrap();
            if is_builtin(file) {
                Some(Command::Type(Cow::Owned(format!(
                    "{} is a shell builtin",
                    file
                ))))
            } else if let Some(exe) = find_executable(file) {
                Some(Command::Type(Cow::Owned(format!(
                    "{file} is {}",
                    exe.display()
                ))))
            } else {
                Some(Command::NotFound(format!("{file}: not found")))
            }
        }
        _ => {
            if let Some(exe) = find_executable(command) {
                Some(Command::Programme(
                    exe,
                    Cow::Owned(command.to_string()),
                    args,
                ))
            } else {
                Some(Command::NotFound(format!("{command}: not found")))
            }
        }
    }
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

fn is_builtin(command: &str) -> bool {
    BUILTINS.contains(&command)
}
