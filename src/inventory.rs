use std::collections::{HashMap, VecDeque};
use chrono::{DateTime, Utc};
use csv::Writer;
use serde::{Serialize};

use crate::{Inflow, Outflow};
use crate::config::AccountingMethod;

#[derive(Debug, Serialize)]
pub struct CashflowRecord {
    tx_out: Option<u32>,
    datetime_out: Option<DateTime<Utc>>,
    tx_in: u32,
    datetime_in: DateTime<Utc>,
    asset: String,
    amount: f64,
    base_price: f64,
    actual_costs: f64,
    actual_proceeds: Option<f64>,
    gains_short_term: Option<f64>,
    gains_long_term: Option<f64>,
}

pub struct Inventory {
    assets: HashMap<String, SingleAssetInventory>,
    log: Vec<CashflowRecord>,
    accounting_method: AccountingMethod,
    currency_precision: f64,
}

impl Inventory {
    pub fn new(
        accounting_method: AccountingMethod,
        currency_precision: f64
    ) -> Inventory {
        Inventory {
            assets: HashMap::new(),
            log: Vec::new(),
            accounting_method,
            currency_precision,
        }
    }

    pub fn deposit(&mut self, asset: &str, inflow: Inflow) {
        let inventory = self.assets
            .entry(asset.to_string())
            .or_insert(SingleAssetInventory::new(
                asset.to_string(),
                self.accounting_method.clone(),
                self.currency_precision));
        inventory.deposit(inflow, &mut self.log);
    }

    pub fn withdraw(&mut self, asset: &str, outflow: Outflow) {
        let inventory = self.assets
            .entry(asset.to_string())
            .or_insert(SingleAssetInventory::new(
                asset.to_string(),
                self.accounting_method.clone(),
                self.currency_precision));
        inventory.withdraw(outflow, &mut self.log);
    }

    pub fn write_log(&self) {
        let mut writer = Writer::from_path("cashflows.csv").unwrap();
        for entry in self.log.iter() {
            writer.serialize(entry).unwrap();
        }
    }
}

struct SingleAssetInventory {
    asset: String,
    layers: VecDeque<Inflow>,
    accounting_method: AccountingMethod,
    currency_precision: f64
}

impl SingleAssetInventory {
    pub fn new(
        asset: String,
        accounting_method: AccountingMethod,
        currency_precision: f64
    ) -> SingleAssetInventory {
        SingleAssetInventory {
            asset,
            layers: VecDeque::new(),
            accounting_method,
            currency_precision,
        }
    }

    pub fn deposit(&mut self, inflow: Inflow, log_stash: &mut Vec<CashflowRecord>) {
        // calculate gains
        let gains_raw = inflow.amount * inflow.base_price - inflow.actual_costs;
        let gains : Option<f64>;
        if gains_raw < self.currency_precision {
            gains = None;
        } else {
            gains = Some(gains_raw);
        }

        // submit log entry
        log_stash.push(CashflowRecord {
            asset: self.asset.clone(),
            tx_out: None,
            datetime_out: None,
            tx_in: inflow.tx_id,
            datetime_in: inflow.datetime,
            amount: inflow.amount,
            base_price: inflow.base_price,
            actual_costs: inflow.actual_costs,
            actual_proceeds: None,
            gains_short_term: gains,
            gains_long_term: None,
        });

        // add layer to inventory
        match self.accounting_method {
            AccountingMethod::FIFO => self.layers.push_back(inflow),
            AccountingMethod::LIFO => self.layers.push_front(inflow),
        };
    }

    pub fn withdraw(&mut self, outflow: Outflow, log_stash: &mut Vec<CashflowRecord>) {
        let mut remaining = outflow.amount;
        while let Some(layer) = self.layers.front_mut() {
            // calculate gains
            let amount = remaining.min(layer.amount);
            let costs = layer.base_price * amount;
            let proceeds;
            if outflow.amount > self.currency_precision {
                proceeds = outflow.proceeds / outflow.amount * amount;
            } else {
                proceeds = outflow.proceeds
            };
            let gains = proceeds - costs;

            // prepare log record
            let mut log = CashflowRecord {
                asset: self.asset.to_string(),
                tx_out: Some(outflow.tx_id),
                datetime_out: Some(outflow.datetime),
                tx_in: layer.tx_id,
                datetime_in: layer.datetime,
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
            log_stash.push(log);

            // subtract amount from layer, remove layer if empty
            layer.amount -= amount;
            if layer.amount < self.currency_precision {
                self.layers.pop_front();
            }

            // subtract from remaining amount, exit if none left
            remaining -= amount;
            if remaining <= self.currency_precision {
                break;
            }
        }

        if remaining > self.currency_precision {
            panic!("insufficient funds");
        };
    }
}