use notify::{Watcher, RecursiveMode, EventKind, event::ModifyKind, RecommendedWatcher, Config};
use regex::RegexSet;
use std::path::Path;
use anyhow::{Result, bail};
use std::process::Command;
use suppaftp::{
    NativeTlsFtpStream, NativeTlsConnector,
    native_tls::TlsConnector,
};


trait Action {
    fn exec_if_match(&self, path: &str) -> bool;
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
    fn exec_if_match(&self, path: &str) -> bool {
        if !self.regex.is_match(path) { return false; }
        println!("{}", path);

        let mut command = Command::new(self.prog);
        for arg in &self.args {
            command.arg(arg);
        }
        let Ok(out) = command.output() else {
                return false;
            };
        println!("{:?}", out);
        return true;
    }
}


fn main() -> Result<()> {
    let actions = [
        CommandAction::new(r".*.scss$", "ls -la")?,
    ];

    let (tx, rx) = std::sync::mpsc::channel();

    let mut watcher = RecommendedWatcher::new(tx, Config::default())?;
    watcher.watch(Path::new("."), RecursiveMode::Recursive)?;

    let ftp_stream = NativeTlsFtpStream::connect("eu-central-1.sftpcloud.io:21")
        .expect("Could not connect to FTP server..");
    let mut ftp_stream = ftp_stream
        .into_secure(
            NativeTlsConnector::from(TlsConnector::new()?), 
            "eu-central-1.sftpcloud.io"
        )?;
    ftp_stream.login("ca15da887bb8490fafb83eb5e7a36ca7ex3", "AXRftfDO0CBWAvZLsOLo7l3G1Y9vtUdyesu")?;

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
                                if action.exec_if_match(path) {
                                    let mut file_reader = std::fs::File::open(path)?;
                                    ftp_stream.transfer_type(suppaftp::types::FileType::Binary)?;
                                    ftp_stream.put_file("test.scss", &mut file_reader)?;
                                    //ftp_stream.mkdir("/test")?;
                                }
                            }
                        }
                    },
                    _ => {},
                }
            }
            Err(e) => println!("watch error: {:?}", e),
        }
    }
    let _ = ftp_stream.quit();
    Ok(())
}
