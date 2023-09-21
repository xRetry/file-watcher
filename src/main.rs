use notify::{Watcher, RecursiveMode, EventKind, event::ModifyKind, RecommendedWatcher};
use regex::RegexSet;
use std::path::{Path, PathBuf};
use anyhow::{Result, bail};
use std::process::Command;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct CommandConfig {
    regex: String,
    cmd: String,
}

#[derive(Debug, Deserialize)]
struct Config {
    path: Option<String>,
    command: Vec<CommandConfig>,
}

trait Action {
    fn exec_if_match(&self, path: &PathBuf) -> bool;
}

struct CommandAction<'a> {
    regex: RegexSet,
    cmd: &'a str,
    args: Vec<&'a str>,
}

impl<'a> CommandAction<'a> {
    fn new(regex: &str, command: &'a str) -> Result<Self> {
        let regex = RegexSet::new(&[regex])?;

        let mut cmd_split = command.split_whitespace();
        let Some(cmd) = cmd_split.next() else {
            bail!("Invalid command!")
        };
        let args = cmd_split.collect();

        Ok(Self{ regex, cmd, args })
    }
}

impl Action for CommandAction<'_> {
    fn exec_if_match(&self, path: &PathBuf) -> bool {
        let path_str = path.to_str()
            .unwrap();
        if !self.regex.is_match(path_str) { return false; }
        print!("{} ... ", path_str);

        let file = path.file_name()
            .and_then(|x| x.to_str());
        let file_name = path.file_stem()
            .and_then(|x| x.to_str());
        let file_dir = path.parent()
            .and_then(|x| x.to_str());

        let mut command = Command::new(self.cmd);
        for arg in &self.args {
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
        
        return match command.output() {
            Ok(_) => {
                print!("done\n");
                true
            },
            Err(e) => {
                println!("error");
                println!(">>> Command: {:?}", command);
                println!(">>> Error: {}", e);
                false
            },
        };
    }
}


fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();
    let config_path = args.get(1)
        .expect("No config path provided!");
    let reader = std::fs::File::open(config_path)
        .expect(&format!("Config file not found: {}", config_path));
    let config: Config = serde_yaml::from_reader(reader)?;

    let mut actions: Vec<Box<dyn Action>> = Vec::new();
    for cmd in &config.command {
        let Ok(c) = CommandAction::new(&cmd.regex, &cmd.cmd) else { continue };
        actions.push(Box::new(c));
    }

    let (tx, rx) = std::sync::mpsc::channel();
    let mut watcher = RecommendedWatcher::new(tx, notify::Config::default())?;
    watcher.watch(Path::new(&config.path.unwrap_or(".".to_string())), RecursiveMode::Recursive)?;
    watcher.unwatch(Path::new("target"))?;

    println!("Config loaded - Waiting for changes..");

    for event in rx {
        match event {
            Ok(event) => {
                match event.kind {
                    EventKind::Modify(ModifyKind::Any) => {
                        for path_buff in &event.paths {
                            for action in &actions {
                                action.exec_if_match(path_buff);
                            }
                        }
                    },
                    _ => {},
                }
            }
            Err(e) => println!("watch error: {:?}", e),
        }
    }
    Ok(())
}
