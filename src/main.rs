use notify::{Watcher, RecursiveMode, Event, EventKind, event::ModifyKind};
use regex::Regex;
use std::path::Path;
use anyhow::Result;


fn main() -> Result<()> {
    let reg = [
        (Regex::new(r".*.scss$").unwrap(), |path: &str| {
            println!("{}", path);
        }),
    ];

    let mut watcher = notify::recommended_watcher(move |res: notify::Result<Event>| {
        match res {
            Ok(event) => {
                //println!("event: {:?}", &event);
                match event.kind {
                    EventKind::Modify(ModifyKind::Data(_)) => {
                        for path_buff in &event.paths {
                            let Some(path) = path_buff.to_str() else {
                                continue;
                            };

                            for (reg, cb) in &reg {
                                if reg.is_match(path) {
                                    cb(path);
                                }
                            }
                        }
                    },
                    _ => {},
                }
            }
            Err(e) => println!("watch error: {:?}", e),
        }
    })?;

    loop {
        watcher.watch(Path::new("."), RecursiveMode::Recursive)?;
    }
    Ok(())
}
