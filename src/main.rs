mod config;
mod prices;
mod inventory;
mod parser;

use chrono::{Utc, DateTime };

use crate::config::Config;
use crate::prices::PriceInformation;
use crate::inventory::Inventory;
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

fn main() {
    let config = Config::new("config.yaml").unwrap();

    let mut inventories = Inventory::new(
        config.method,
        config.currency_precision);
    let mut price_information = PriceInformation::new(
        config.api_key.clone());
    let mut parser = Parser::new(
        &mut inventories,
        &mut price_information,
        config.base_asset.clone());

    parser.parse_sheet("transactions.csv");
    inventories.write_log();
}
