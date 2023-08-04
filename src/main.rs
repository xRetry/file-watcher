use notify::{Watcher, RecursiveMode, EventKind, event::ModifyKind, RecommendedWatcher, Config};
use regex::RegexSet;
use std::path::Path;
use anyhow::Result;


trait Action {
    fn exec_if_match(&self, path: &str);
}

struct SassAction {
    regex: RegexSet,
}

impl SassAction {
    fn new(regex: &str) -> Result<Self> {
        let regex = RegexSet::new(&[regex])?;
        Ok(Self{ regex })
    }
}

impl Action for SassAction {
    fn exec_if_match(&self, path: &str) {
        if !self.regex.is_match(path) { return }
        println!("{}", path);
    }
}

fn main() -> Result<()> {
    let actions = [
        SassAction::new(r".*.scss$")?,
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
