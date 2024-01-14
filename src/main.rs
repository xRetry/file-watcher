use anyhow::Result;
use file_watcher::run_watcher;
use file_watcher::config::parse_args;

fn main() -> Result<()> {
    let config = parse_args()?;
    return run_watcher(config);
}
