mod config;
mod prices;
mod inventory;
mod parser;

use std::process;
use clap::{Arg, App};
use chrono::{Utc, DateTime};

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
    let matches = App::new("CryptoTax")
        .version("0.1.0")
        .author("Tim Hopp")
        .about("Processes transaction statements into capital gains statements")
        .arg(Arg::with_name("input_path")
            .short("i")
            .long("input")
            .takes_value(true)
            .help("Transaction file to process"))
        .arg(Arg::with_name("output_path")
            .short("o")
            .long("output")
            .takes_value(true)
            .help("Capital Gains Statement to write"))
        .arg(Arg::with_name("config_path")
            .short("c")
            .long("config")
            .takes_value(true)
            .help("Config file"))
        .arg(Arg::with_name("clear")
            .long("clear")
            .takes_value(false)
            .help("Clears the price cache"))
        .get_matches();

    let input_path = matches.value_of("input_path").unwrap_or("transactions.csv");
    let output_path = matches.value_of("output_path").unwrap_or("cashflows.csv");
    let conf_path = matches.value_of("config_path").unwrap_or("config.yaml");
    let clear_cache = matches.is_present("clear");

    let config = Config::new(conf_path).unwrap_or_else(|_| {
        println!("Unable to read config file \"{}\"", conf_path);
        process::exit(1);
    });

    let mut price_information = PriceInformation::new(
        config.api_key.clone());
    if clear_cache {
        price_information.clear();
    }

    let mut inventories = Inventory::new(
        config.method,
        config.currency_precision);
    let mut parser = Parser::new(
        &mut inventories,
        &mut price_information,
        config.base_asset.clone());

    parser.parse_sheet(input_path);
    inventories.write_log(output_path);
}
