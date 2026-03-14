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
        let args = parts;

        match command {
            "exit" => return,
            "echo" => {
                println!("{}", args.collect::<Vec<_>>().join(" "));
            }
            _ => {
                eprintln!("{command}: command not found");
            }
        }
    }
}
