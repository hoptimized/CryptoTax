use chrono::{Utc, DateTime, SecondsFormat};
use serde::{Deserialize};
use std::collections::{HashMap};
use std::collections::hash_map::Entry::{Occupied, Vacant};
use std::error;
use std::fs::File;

#[derive(Deserialize, Debug)]
pub struct ExchangeRateRecord {
    time : DateTime<Utc>,
    asset_id_base : String,
    asset_id_quote : String,
    rate: f64,
}

pub struct PriceInformation {
    price_cache : HashMap<String, HashMap<DateTime<Utc>, f64>>,
    api_keys : HashMap<String, String>,
}

impl PriceInformation {
    pub fn new(api_keys : HashMap<String, String>) -> PriceInformation {
        let mut res = PriceInformation {
            price_cache: HashMap::new(),
            api_keys
        };
        res.load().expect("Cannot load price cache!");
        res
    }

    pub fn clear(&mut self) {
        self.price_cache.clear();
    }

    pub fn get(
        &mut self,
        asset_id_base: &str,
        asset_id_quote: &str,
        datetime: DateTime<Utc>
    ) -> f64 {
        let prices_for_asset = self.price_cache
            .entry(asset_id_base.to_string())
            .or_insert(HashMap::new());

        let entry = prices_for_asset.entry(datetime.clone());

        // if price was found in cache, return the cached value
        if let Occupied(entry) = entry {
            return entry.get().clone();
        }

        // prepare API call to retrieve price
        let url = format!(
            "https://rest.coinapi.io/v1/exchangerate/{asset_id_base}/{asset_id_quote}?time={time}",
            asset_id_base = asset_id_base,
            asset_id_quote = asset_id_quote,
            time = datetime.to_rfc3339_opts(SecondsFormat::Secs, true));

        if let Vacant(entry) = entry {
            let api_key = self.api_keys
                .get("coinapi")
                .expect("Currently only supporting CoinAPI");

            let response = reqwest::blocking::Client::new()
                .get(url)
                .header("X-CoinAPI-Key", api_key)
                .send()
                .unwrap();
            let response : ExchangeRateRecord = response
                .json()
                .unwrap();
            let price = response.rate;

            entry.insert(price);

            return price;
        }

        panic!("Should never reach here");
    }

    fn load(&mut self) -> Result<(), Box<dyn error::Error>> {
        self.clear();

        let file = std::fs::File::open("data/.price_cache")?;
        let data = ::serde_yaml::from_reader(file)?;
        self.price_cache = data;

        Ok(())
    }

    fn save(&self) -> Result<(), Box<dyn error::Error>> {
        ::serde_yaml::to_writer(
            &File::create("data/.price_cache")?,
            &self.price_cache)?;
        Ok(())
    }
}

impl Drop for PriceInformation {
    fn drop(&mut self) {
        self.save().expect("Could not save price cache!");
    }
}