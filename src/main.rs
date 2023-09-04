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
struct SassConfig {
    out_dir: String,
}

#[derive(Debug, Deserialize)]
struct Config {
    sass: Option<SassConfig>,
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

struct SassAction {
    regex: RegexSet, 
    out_dir: PathBuf,
}

impl SassAction {
    fn new(out_dir: &str) -> Self {
        Self {
            regex: RegexSet::new(&[r".*.scss$"]).unwrap(),
            out_dir: Path::new(out_dir).to_path_buf(),
        }
    }
}

impl Action for SassAction {
    fn exec_if_match(&self, path: &PathBuf) -> bool {
        let path_str = path.to_str().unwrap();
        if !self.regex.is_match(path_str) { return false; }

        let Some(file_name) = path.file_stem() else { return false };

        let out_path = if self.out_dir.is_absolute() {
            self.out_dir.join(file_name).join(".css")
        } else {
            let Some(file_dir) = path.parent() else { return false };
            file_dir.join(&self.out_dir).join(file_name).join(".css")
        };

        let mut command = Command::new("sass");
        command.arg(path_str);
        command.arg(out_path);
        println!("{:?}", command);

        let Ok(out) = command.output() else {
                return false;
            };
        println!("{:?}", out);
        return true;
    }
}

fn main() -> Result<()> {
    let reader = std::fs::File::open("config.yml")?;
    let config: Config = serde_yaml::from_reader(reader)?;
    println!("Config loaded - Waiting for changes..");

    let mut actions: Vec<Box<dyn Action>> = Vec::new();
    match config.sass {
        Some(sass_config) => actions.push(Box::new(SassAction::new(&sass_config.out_dir))),
        None => (),
    }
    for cmd in &config.command {
        let Ok(c) = CommandAction::new(&cmd.regex, &cmd.cmd) else { continue };
        actions.push(Box::new(c));
    }

    // TODO: Create and add CommandActions
    
    //return Ok(());
    //let actions = [
    //    CommandAction::new(r".*.scss$", "ls -la")?,
    //];

    let (tx, rx) = std::sync::mpsc::channel();

    let mut watcher = RecommendedWatcher::new(tx, notify::Config::default())?;
    watcher.watch(Path::new("."), RecursiveMode::Recursive)?;

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
