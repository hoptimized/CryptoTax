use clap::{Arg};

pub struct App {
    input_path: String,
    output_path: String,
    conf_path: String,
    clear_cache: bool,
}

impl App {
    pub fn new() -> App {
        let matches = clap::App::new("CryptoTax")
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

        App {
            input_path: matches.value_of("input_path")
                .unwrap_or("transactions.csv")
                .to_string(),
            output_path: matches.value_of("output_path")
                .unwrap_or("cashflows.csv")
                .to_string(),
            conf_path: matches.value_of("config_path")
                .unwrap_or("config.yaml")
                .to_string(),
            clear_cache: matches.is_present("clear"),
        }
    }

    pub fn get_config_path(&self) -> &str { &self.conf_path }
    pub fn get_input_path(&self) -> &str { &self.input_path }
    pub fn get_output_path(&self) -> &str { &self.output_path }
    pub fn get_clear_cache(&self) -> bool { self.clear_cache }
}