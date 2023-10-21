use notify::{Watcher, RecursiveMode, EventKind, event::ModifyKind, RecommendedWatcher};
use regex::RegexSet;
use std::path::{Path, PathBuf};
use anyhow::Result;
use std::process::Command;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct CommandConfig {
    pub regex: String,
    pub cmd: Option<String>,
    pub chain: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
pub struct Config {
    pub path: Option<String>,
    pub exclude: Option<Vec<String>>,
    pub command: Vec<CommandConfig>,
}

trait Action {
    fn exec_if_match(&self, path: &PathBuf) -> bool;
}

struct CommandAction {
    regex: RegexSet,
    commands: Vec<(String, Vec<String>)>
}

impl CommandAction {
    fn new(config: CommandConfig) -> Result<Self> {
        let regex = RegexSet::new(&[config.regex])?;

        let mut commands = Vec::new();
        if let Some(cmd) = config.cmd {
            commands = vec![cmd];
        } else if let Some(chain) = config.chain {
            commands = chain;
        }

        let commands = commands.into_iter()
            .map(|cmd| {
                let mut cmd_split = cmd.split_whitespace().map(String::from);
                let Some(c) = cmd_split.next() else {
                    panic!("Invalid command!");
                };
                let a = cmd_split.collect();
                return (c, a)
            })
            .collect();

        return Ok(Self{ regex, commands });
    }
}

impl Action for CommandAction {
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
                Ok(_) => {
                    print!("done\n");
                },
                Err(e) => {
                    println!("error");
                    println!(">>> Command: {:?}", command);
                    println!(">>> Error: {}", e);
                    return false
                },
            };
        }

        return true;
    }
}

pub fn parse_args() -> Result<Config> {
    let args: Vec<String> = std::env::args().collect();
    let config_path = args.get(1)
        .expect("No config path provided!");
    let reader = std::fs::File::open(config_path)
        .expect(&format!("Config file not found: {}", config_path));
    let config: Config = serde_yaml::from_reader(reader)?;
    return Ok(config);
}

pub fn run_watcher(config: Config) -> Result<()> {
    let mut actions: Vec<Box<dyn Action>> = Vec::new();
    for cmd in config.command {
        let Ok(c) = CommandAction::new(cmd) else { continue };
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
