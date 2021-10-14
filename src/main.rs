mod app;
mod config;
mod prices;
mod accounting;

use std::process;

use crate::app::App;
use crate::config::Config;
use crate::prices::PriceInformation;
use crate::accounting::accountant::Accountant;

fn main() {
    let app = App::new();

    let config = Config::new(app.get_config_path()).unwrap_or_else(|_| {
        println!("Unable to read config file \"{}\"", app.get_config_path());
        process::exit(1);
    });

    let mut price_information = PriceInformation::new(
        config.api_key.clone());
    if app.get_clear_cache() {
        price_information.clear();
    }

    Accountant::new(&mut price_information)
        .method(config.method)
        .base_asset(config.base_asset.as_str())
        .precision(config.currency_precision)
        .analyze_file(app.get_input_path())
        .write_to_file(app.get_output_path());
}
