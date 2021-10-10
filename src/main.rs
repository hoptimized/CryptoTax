mod config;
mod prices;
mod inventory;
mod parser;

use chrono::{Utc, DateTime };
use once_cell::sync::Lazy;

use crate::config::Config;
use crate::prices::PriceInformation;
use crate::inventory::Inventories;
use crate::parser::Parser;

pub struct Inflow {
    tx_id: u32,
    datetime: DateTime<Utc>,
    amount: f64,
    base_price: f64,
    actual_costs: f64,
}

pub struct Outflow {
    tx_id: u32,
    datetime: DateTime<Utc>,
    amount: f64,
    proceeds: f64,
}

pub static CONFIG: Lazy<Config> = Lazy::new(|| {
    let config = Config::new("config.yaml").unwrap();
    config
});

fn main() {
    println!("Starting tax calculator for base currency {}", CONFIG.base_asset);

    let mut inventories = Inventories::new();
    let mut price_information : PriceInformation = PriceInformation::new();
    let mut parser = Parser::new(&mut inventories, &mut price_information);

    parser.parse_sheet("data/Transactions.csv");
    inventories.write_log();
}
