use std::collections::HashMap;
use std::error;
use serde::{Deserialize};

use crate::accounting::AccountingMethod;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub base_asset: String,
    pub method: AccountingMethod,
    pub currency_precision: f64,
    pub api_key: HashMap<String, String>,
}

impl Config {
    pub fn new(config_path: &str) -> Result<Config, Box<dyn error::Error>> {
        let file = std::fs::File::open(config_path)?;
        let config : Config = ::serde_yaml::from_reader(file)?;
        Ok(config)
    }
}