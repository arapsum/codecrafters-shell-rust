#[allow(unused_imports)]
use std::io::{self, Write};

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
                let arg: Result<BuiltInCommand, ()> = args[0].try_into();
                match arg {
                    Ok(arg) => {
                        println!("{} is a shell builtin", arg.as_str());
                    }
                    Err(_) => {
                        eprintln!("{}: not found", args[0]);
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
pub enum BuiltInCommand {
    Exit,
    Echo,
    Type,
}

impl BuiltInCommand {
    pub fn as_str(&self) -> &str {
        match self {
            BuiltInCommand::Exit => "exit",
            BuiltInCommand::Echo => "echo",
            BuiltInCommand::Type => "type",
        }
    }
}

impl TryFrom<&str> for BuiltInCommand {
    type Error = ();

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "exit" => Ok(BuiltInCommand::Exit),
            "echo" => Ok(BuiltInCommand::Echo),
            "type" => Ok(BuiltInCommand::Type),
            _ => Err(()),
        }
    }
}
