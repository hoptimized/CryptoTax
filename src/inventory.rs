use std::collections::{HashMap, VecDeque};
use chrono::{DateTime, Utc};
use csv::Writer;
use serde::{Serialize};

use crate::{CONFIG, Inflow, Outflow};

#[derive(Debug, Serialize)]
pub struct CashflowRecord {
    tx_in: u32,
    datetime_in: DateTime<Utc>,
    tx_out: Option<u32>,
    datetime_out: Option<DateTime<Utc>>,
    amount: f64,
    base_price: f64,
    actual_costs: f64,
    actual_proceeds: Option<f64>,
    gains_short_term: Option<f64>,
    gains_long_term: Option<f64>,
}

pub struct Inventories {
    inventories: HashMap<String, Inventory>,
}

impl Inventories {
    pub fn new() -> Inventories {
        Inventories {
            inventories: HashMap::new(),
        }
    }

    pub fn get(&mut self, asset: &str) -> &mut Inventory {
        let inventory = self.inventories
            .entry(asset.to_string())
            .or_insert(Inventory::new(asset.to_string()));
        inventory
    }

    pub fn write_log(&self) {
        for (asset, inventory) in self.inventories.iter() {
            let mut writer = Writer::from_path(
                format!("data/Assets/{}.csv", asset)).unwrap();
            for entry in inventory.log.iter() {
                writer.serialize(entry).unwrap();
            }
        }
    }
}

pub struct Inventory {
    asset: String,
    layers: VecDeque<Inflow>, //TODO: add a wrapper so that we can use FIFO and LIFO (queue, stack)
    log: Vec<CashflowRecord>,
}

impl Inventory {
    pub fn new(asset: String) -> Inventory {
        Inventory {
            asset,
            layers: VecDeque::new(),
            log: Vec::new(),
        }
    }

    pub fn deposit(&mut self, inflow: Inflow) {
        let gains_raw = inflow.amount * inflow.base_price - inflow.actual_costs;

        let gains : Option<f64>;
        if gains_raw < CONFIG.currency_precision {
            gains = None;
        } else {
            gains = Some(gains_raw);
        }

        self.log.push(CashflowRecord {
            tx_in: inflow.tx_id,
            datetime_in: inflow.datetime,
            tx_out: None,
            datetime_out: None,
            amount: inflow.amount,
            base_price: inflow.base_price,
            actual_costs: inflow.actual_costs,
            actual_proceeds: None,
            gains_short_term: gains,
            gains_long_term: None,
        });
        self.layers.push_back(inflow);
    }

    pub fn withdraw(&mut self, outflow: Outflow) {
        let mut remaining = outflow.amount;
        while let Some(layer) = self.layers.front_mut() {
            // calculate gains
            let amount = remaining.min(layer.amount);
            let costs = layer.base_price * amount;
            let proceeds;
            if outflow.amount > CONFIG.currency_precision {
                proceeds = outflow.proceeds / outflow.amount * amount;
            } else {
                proceeds = outflow.proceeds
            };
            let gains = proceeds - costs;

            // prepare log record
            let mut log = CashflowRecord {
                tx_in: layer.tx_id,
                datetime_in: layer.datetime,
                tx_out: Some(outflow.tx_id),
                datetime_out: Some(outflow.datetime),
                amount: -amount,
                base_price: layer.base_price,
                actual_costs: costs,
                actual_proceeds: Some(proceeds),
                gains_short_term: None,
                gains_long_term: None,
            };

            // move gains to correct column (long-term/short-term)
            let duration = outflow.datetime.signed_duration_since(layer.datetime).num_days();
            if duration > 365 {
                log.gains_long_term = Some(gains);
            } else {
                log.gains_short_term = Some(gains);
            }

            // submit log entry
            self.log.push(log);

            // subtract amount from layer, remove layer if empty
            layer.amount -= amount;
            if layer.amount < CONFIG.currency_precision {
                self.layers.pop_front();
            }

            // subtract from remaining amount, exit if none left
            remaining -= amount;
            if remaining <= CONFIG.currency_precision {
                break;
            }
        }

        if remaining > CONFIG.currency_precision {
            panic!("insufficient funds");
        };
    }
}