use regex::RegexSet;
use shlex::Shlex;
use std::{path::PathBuf, process::Command};

use crate::config::CommandConfig;

pub trait Action {
    fn exec_if_match(&self, path: &PathBuf) -> bool;
}

pub struct CommandAction {
    regex: RegexSet,
    commands: Vec<(String, Vec<String>)>,
    exclude: Option<RegexSet>,
}

impl CommandAction {
    pub fn new(config: CommandConfig) -> Self {
        let regex = RegexSet::new(&[config.regex]).unwrap();

        let exclude = config.exclude.map(
            |x| RegexSet::new(x).unwrap()
        );

        let mut commands = Vec::new();
        if let Some(cmd) = config.cmd {
            commands = vec![cmd];
        } else if let Some(chain) = config.chain {
            commands = chain;
        }

        let commands = commands.into_iter()
            .map(|cmd| {
                let mut cmd_split = Shlex::new(&cmd);
                let Some(c) = cmd_split.next() else {
                    panic!("Invalid command!");
                };
                let a = cmd_split.collect();
                return (c, a)
            })
            .collect();

        return Self{ regex, commands, exclude };
    }
}

impl Action for CommandAction {
    fn exec_if_match(&self, path: &PathBuf) -> bool {
        //let path = path.canonicalize().unwrap();
        let path_str = path.to_str()
            .unwrap();

        if self.exclude.as_ref().is_some_and(|e| e.is_match(path_str))
        || !self.regex.is_match(path_str) { 
            return false; 
        }

        print!("{} ... ", path_str);

        let file = path.file_name()
            .and_then(|x| x.to_str());
        let file_name = path.file_stem()
            .and_then(|x| x.to_str());
        let file_dir = path.parent()
            .and_then(|x| x.to_str());

        for (cmd, args) in &self.commands {
            let mut command = Command::new(cmd);
            for arg in args {
                let mut arg = arg.to_string();

                if file.is_some() {
                    arg = arg.replace("{file}", file.unwrap());
                }

                if file_name.is_some() {
                    arg = arg.replace("{file_name}", file_name.unwrap());
                }

                if file_name.is_some() {
                    arg = arg.replace("{file_dir}", file_dir.unwrap());
                }

                command.arg(arg);
            }
            
            match command.output() {
                Ok(output) => {
                    match output.status.code() {
                        Some(0) => (),
                        _ => {
                            println!("error");
                            println!(">>> Command: {:?}", command);
                            println!(">>> StdOut: {}", &std::str::from_utf8(&output.stderr).unwrap());
                            return false;
                        },
                    }
                },
                Err(e) => {
                    println!("error");
                    println!(">>> Command: {:?}", command);
                    println!(">>> Error: {}", e);
                    return false;
                },
            };
        }

        println!("done");
        return true;
    }
}
