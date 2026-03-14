#[allow(unused_imports)]
use std::io::{self, Write};

const BUILTINS: &[&str] = &["exit", "echo", "type"];

fn main() {
    loop {
        print!("$ ");
        io::stdout().flush().unwrap();

        let mut input = String::new();
        io::stdin()
            .read_line(&mut input)
            .expect("Failed to read line");

        let mut parts = input.trim().split_whitespace();
        let command = parts.next().unwrap();
        let args = parts.collect::<Vec<_>>();

        match command {
            "exit" => return,
            "echo" => {
                println!("{}", args.join(" "));
            }
            "type" => {
                if args.is_empty() {
                    eprintln!("type: missing operand");
                } else {
                    let arg = args[0];
                    if BUILTINS.contains(&arg) {
                        println!("{} is a shell builtin", arg);
                    } else {
                        eprintln!("{}: not found", arg);
                    }
                }
            }
            _ => {
                eprintln!("{command}: command not found");
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum ShellCommand {
    Exit,
    Echo(String),
    Type(String),
    NotFound,
}

impl ShellCommand {
    pub fn as_str(&self) -> &str {
        match self {
            Self::Exit => "exit",
            Self::Echo(_) => "echo",
            Self::Type(_) => "type",
            Self::NotFound => "not found",
        }
    }
}
