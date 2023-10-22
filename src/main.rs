use anyhow::Result;
use file_watcher::{run_watcher, parse_args};

fn main() -> Result<()> {
    let config = parse_args()?;
    return run_watcher(config, None);
}
