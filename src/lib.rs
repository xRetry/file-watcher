use notify_debouncer_full::notify::{Watcher, RecursiveMode, EventKind, event::ModifyKind};
use regex::RegexSet;
use std::path::{Path, PathBuf};
use anyhow::Result;
use std::process::Command;
use serde::Deserialize;
use shlex::Shlex;
use notify_debouncer_full::new_debouncer;

#[derive(Debug, Deserialize)]
pub struct CommandConfig {
    pub regex: String,
    pub cmd: Option<String>,
    pub chain: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
pub struct Config {
    pub paths: Option<Vec<String>>,
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
                let mut cmd_split = Shlex::new(&cmd);
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
        //let path = path.canonicalize().unwrap();
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

    //let exclude_dirs = config.exclude 
    //    .unwrap_or(Vec::new());
    //let exclude_dirs: HashSet<_> = exclude_dirs
    //    .iter()
    //    .map(|path_str| Path::new(path_str))
    //    .collect();

    let (tx, rx) = std::sync::mpsc::channel();

    let mut debouncer = new_debouncer(std::time::Duration::from_millis(200), None, tx)?;

    let watcher = debouncer.watcher();

    let paths = config.paths.unwrap_or(vec![".".to_string()]);
    for path in paths {
        watcher.watch(Path::new(&path), RecursiveMode::Recursive)?;
    }

    println!("Config loaded - Waiting for changes..");

    for events in rx {
        match events {
            Ok(events) => {
                for event in events {
                    match event.kind {
                        EventKind::Modify(ModifyKind::Data(_)) | // Linux
                        EventKind::Modify(ModifyKind::Any) => { // Windows
                            for path_buff in &event.paths {
                                // TODO(marco): Fix path matching for exclude dirs
                                //if exclude_dirs.contains(path_buff.as_path()) {
                                //        continue;
                                //}

                                for action in &actions {
                                    action.exec_if_match(path_buff);
                                }
                            }
                        },
                        EventKind::Other => {
                            if event.info() == Some("exit") {
                                return Ok(());
                            }
                        },
                        _ => {},
                    } // match event.kind
                } // for
            },
            Err(e) => println!("watch error: {:?}", e),
        } // match events
    }
    Ok(())
}
