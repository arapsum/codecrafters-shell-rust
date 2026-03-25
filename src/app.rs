use std::{
    fs::OpenOptions,
    io::{self, Write},
    path::PathBuf,
};

use crate::commands::{self, CommandType, Redirect, RedirectKind, RedirectTarget};

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

            let parsed = ParsedLine::parse(&line);

            match commands::parse_command(parsed.tokens) {
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
                CommandType::Echo(cmd) => {
                    let output = cmd.args.join(" ");

                    if let Some(mut w) =
                        open_redirect_writer(&parsed.redirects, RedirectKind::Stdout)?
                    {
                        writeln!(w, "{}", output)?;
                    } else {
                        println!("{}", cmd.args.join(" "));
                    }
                }
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
                CommandType::Programme(cmd) => cmd.run(&parsed.redirects).unwrap(),
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct ParsedLine {
    pub tokens: Vec<String>,
    pub redirects: Vec<Redirect>,
}

impl ParsedLine {
    pub fn parse(line: &str) -> Self {
        let tokens = shell_words::split(line).unwrap_or_default();

        let mut command_tokens = Vec::new();
        let mut redirects = Vec::new();

        let mut i = 0;

        while i < tokens.len() {
            match tokens[i].as_str() {
                ">>" | "1>>" => {
                    if let Some(target) = tokens.get(i + 1) {
                        redirects.push(Redirect {
                            kind: RedirectKind::StdoutAppend,
                            target: RedirectTarget::File(PathBuf::from(target)),
                        });
                        i += 2;
                        continue;
                    }
                }
                ">" | "1>" => {
                    if let Some(target) = tokens.get(i + 1) {
                        redirects.push(Redirect {
                            kind: RedirectKind::Stdout,
                            target: RedirectTarget::File(PathBuf::from(target)),
                        });
                        i += 2;
                        continue;
                    }
                }
                "2>>" => {
                    if let Some(target) = tokens.get(i + 1) {
                        redirects.push(Redirect {
                            kind: RedirectKind::StderrAppend,
                            target: RedirectTarget::File(PathBuf::from(target)),
                        });
                        i += 2;
                        continue;
                    }
                }
                "2>" => {
                    if let Some(target) = tokens.get(i + 1) {
                        redirects.push(Redirect {
                            kind: RedirectKind::Stderr,
                            target: RedirectTarget::File(PathBuf::from(target)),
                        });
                        i += 2;
                        continue;
                    }
                }
                _ => command_tokens.push(tokens[i].clone()),
            }
            i += 1;
        }

        Self {
            tokens: command_tokens,
            redirects,
        }
    }
}
pub fn open_redirect_writer(
    redirects: &[Redirect],
    kind: RedirectKind,
) -> anyhow::Result<Option<Box<dyn Write>>> {
    for r in redirects {
        let is_match = matches!(
            (&r.kind, &kind),
            (
                RedirectKind::Stderr | RedirectKind::StderrAppend,
                RedirectKind::Stderr
            ) | (
                RedirectKind::Stdout | RedirectKind::StdoutAppend,
                RedirectKind::Stdout
            )
        );

        if is_match {
            let RedirectTarget::File(target) = &r.target;

            let file = match r.kind {
                RedirectKind::Stdout | RedirectKind::Stderr => OpenOptions::new()
                    .write(true)
                    .create(true)
                    .truncate(true)
                    .open(target)?,
                RedirectKind::StdoutAppend | RedirectKind::StderrAppend => {
                    OpenOptions::new().create(true).append(true).open(target)?
                }
            };

            return Ok(Some(Box::new(file)));
        }
    }
    Ok(None)
}
