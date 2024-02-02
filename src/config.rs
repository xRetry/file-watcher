use serde::Deserialize;
use anyhow::Result;

#[derive(Debug, Deserialize)]
pub struct CommandConfig {
    pub regex: String,
    pub cmd: Option<String>,
    pub chain: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
pub struct Config {
    pub paths: Option<Vec<String>>,
    pub commands: Vec<CommandConfig>,
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
