use notify_debouncer_full::notify::{Watcher, RecursiveMode, EventKind, event::{ModifyKind, AccessKind, AccessMode, CreateKind}};
use std::path::Path;
use anyhow::Result;
use notify_debouncer_full::new_debouncer;

pub mod config;
use config::Config;

mod actions;
use actions::{Action, CommandAction};

pub fn run_watcher(config: Config) -> Result<()> {
    let mut actions: Vec<Box<dyn Action>> = Vec::new();
    for cmd in config.command {
        let c = CommandAction::new(cmd);
        actions.push(Box::new(c));
    }

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
                    match event.event.kind {
                        EventKind::Modify(ModifyKind::Data(_)) | // Linux
                        EventKind::Create(CreateKind::File) | // Linux + Neovim
                        EventKind::Modify(ModifyKind::Any) => { // Windows
                            for path_buff in &event.paths {
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
