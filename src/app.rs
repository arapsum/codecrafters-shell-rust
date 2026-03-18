use std::{
    io::{self, Write},
    path::PathBuf,
};

use crate::commands::{self, CommandType};

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct App {
    cwd: PathBuf,
}

impl App {
    pub fn new() -> Self {
        Self {
            cwd: std::env::current_dir().unwrap(),
        }
    }

    pub fn run(&self) -> anyhow::Result<()> {
        loop {
            print!("$ ");
            io::stdout().flush().expect("Failed to flush stdout");

            let mut line = String::new();
            io::stdin()
                .read_line(&mut line)
                .expect("Failed to read line");

            if line.ends_with('\n') {
                line.pop();

                if line.ends_with('\r') {
                    line.pop();
                }
            }

            if line.is_empty() {
                continue;
            }

            match commands::parse_command(&line) {
                CommandType::Cd(cmd) => {
                    let mut target = cmd.args[0].as_str();
                    if target.is_empty() {
                        target = "~";
                    }

                    let dir = if target.starts_with("~") {
                        let home = std::env::home_dir().unwrap();
                        let rest = target.trim_start_matches("~").trim_start_matches("/");
                        home.join(rest)
                    } else {
                        PathBuf::from(target)
                    };

                    if std::env::set_current_dir(&dir).is_err() {
                        eprintln!("{}: No such file or directory", dir.display());
                    }
                }
                CommandType::Echo(cmd) => println!("{}", cmd.args.join(" ")),
                CommandType::Exit(code) => std::process::exit(code),
                CommandType::Pwd(cmd) => {
                    if let Some(path) = cmd.path {
                        println!("{}", path.display());
                    }
                }
                CommandType::Type(cmd) => {
                    for arg in cmd.args {
                        if commands::is_builtin(&arg) {
                            println!("{} is a shell builtin", arg);
                            continue;
                        }

                        if let Some(exe) = commands::find_executable(&arg) {
                            if commands::is_executable(&exe) {
                                println!("{} is {}", &arg, exe.display());
                            } else {
                                println!("{}: not found", arg)
                            }
                        } else {
                            println!("{}: not found", arg)
                        }
                    }
                }
                CommandType::Unkown(err) => eprintln!("{err}"),
                CommandType::Programme(cmd) => cmd.run().unwrap(),
            }
        }
    }
}
