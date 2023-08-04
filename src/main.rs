use notify::{Watcher, RecursiveMode, EventKind, event::ModifyKind, RecommendedWatcher, Config};
use regex::RegexSet;
use std::path::Path;
use anyhow::{Result, bail};
use std::process::Command;


trait Action {
    fn exec_if_match(&self, path: &str);
}

struct CommandAction<'a> {
    regex: RegexSet,
    prog: &'a str,
    args: Vec<&'a str>,
}

impl<'a> CommandAction<'a> {
    fn new(regex: &str, command: &'a str) -> Result<Self> {
        let regex = RegexSet::new(&[regex])?;

        let mut cmd_split = command.split_whitespace();
        let Some(prog) = cmd_split.next() else {
            bail!("Invalid command!")
        };
        let args = cmd_split.collect();

        Ok(Self{ regex, prog, args })
    }
}

impl Action for CommandAction<'_> {
    fn exec_if_match(&self, path: &str) {
        if !self.regex.is_match(path) { return }
        println!("{}", path);

        let mut command = Box::new(Command::new(self.prog));
        for arg in &self.args {
            command.arg(arg);
        }
        let Ok(out) = command.output() else {
                return
            };
        println!("{:?}", out);
    }
}

fn main() -> Result<()> {
    let actions = [
        CommandAction::new(r".*.scss$", "ls -la")?,
    ];

    let (tx, rx) = std::sync::mpsc::channel();

    let mut watcher = RecommendedWatcher::new(tx, Config::default())?;
    watcher.watch(Path::new("."), RecursiveMode::Recursive)?;

    for event in rx {
        match event {
            Ok(event) => {
                //println!("event: {:?}", &event);
                match event.kind {
                    EventKind::Modify(ModifyKind::Data(_)) => {
                        for path_buff in &event.paths {
                            let Some(path) = path_buff.to_str() else {
                                continue;
                            };

                            for action in &actions {
                                action.exec_if_match(path);
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
