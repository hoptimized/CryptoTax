use std::collections::HashMap;
use std::error;
use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub(crate) base_asset: String,
    pub(crate) currency_precision: f64,
    pub(crate) api_key: HashMap<String, String>,
}

impl Config {
    pub fn new(config_path: &str) -> Result<Config, Box<dyn error::Error>> {
        let file = std::fs::File::open(config_path)?;
        let config : Config = ::serde_yaml::from_reader(file)?;
        Ok(config)
    }
}