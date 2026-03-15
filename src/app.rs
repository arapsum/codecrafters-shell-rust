use std::{
    io::{self, Write},
    path::PathBuf,
};

use crate::commands::{self, Command};

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

            match commands::parse_command(&line) {
                Command::Exit(code) => std::process::exit(code.unwrap_or(0)),
                Command::Echo(msg) => println!("{msg}"),
                Command::Type(cmd_type) => println!("{cmd_type}"),
                Command::Unkown(err) => eprintln!("{err}"),
                Command::Pwd(pwd) => println!("{pwd}"),
                Command::Programme(_path, name, args) => {
                    match std::process::Command::new(name.to_string())
                        .args(args)
                        .spawn()
                    {
                        Ok(mut child) => {
                            let status = child.wait()?;
                            anyhow::ensure!(
                                status.success(),
                                "Programme exited with non-zero status"
                            );
                        }
                        Err(err) => eprintln!("{err}"),
                    }
                }
            }
        }
    }
}
