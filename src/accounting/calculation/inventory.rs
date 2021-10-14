use std::collections::{VecDeque};

use crate::accounting::{AccountingMethod, CashflowRecord, Inflow, Outflow};

pub struct Inventory {
    asset: String,
    layers: VecDeque<Inflow>,
    accounting_method: AccountingMethod,
    currency_precision: f64
}

impl Inventory {
    pub fn new(
        asset: String,
        accounting_method: AccountingMethod,
        currency_precision: f64
    ) -> Inventory {
        Inventory {
            asset,
            layers: VecDeque::new(),
            accounting_method,
            currency_precision,
        }
    }

    pub fn deposit(&mut self, inflow: Inflow) -> CashflowRecord {
        // calculate possible gains from discounts / gifts / staking rewards etc.
        let gains_raw = inflow.amount * inflow.base_price - inflow.actual_costs;
        let gains : Option<f64> = match gains_raw >= self.currency_precision {
            true => Some(gains_raw),
            false => None,
        };

        // create log entry
        let log_entry = CashflowRecord {
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
        };

        // add layer to inventory
        match self.accounting_method {
            AccountingMethod::FIFO => self.layers.push_back(inflow),
            AccountingMethod::LIFO => self.layers.push_front(inflow),
        };

        log_entry
    }

    pub fn withdraw(&mut self, outflow: Outflow) -> Vec<CashflowRecord> {
        let mut remaining_amount = outflow.amount;
        let mut log_entries: Vec<CashflowRecord> = Vec::new();

        // withdraw assets layer by layer, using FIFO/LIFO accounting
        while let Some(layer) = self.layers.front_mut() {
            // determine costs
            let amount = remaining_amount.min(layer.amount); // see how much we can take
            let costs = layer.base_price * amount;

            // calculate proceeds proportional to the amount that we are taking from the layer;
            // if we are taking close to zero, put all proceeds on this layer
            let proceeds = match outflow.amount >= self.currency_precision {
                true => outflow.proceeds / outflow.amount * amount,
                false => outflow.proceeds,
            };

            // calculate gains
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
            match duration > 365 {
                true => log.gains_long_term = Some(gains),
                false => log.gains_short_term = Some(gains),
            }

            // submit log entry
            log_entries.push(log);

            // subtract amount from layer, remove layer if empty
            layer.amount -= amount;
            if layer.amount < self.currency_precision {
                self.layers.pop_front();
            }

            // subtract from remaining amount, exit if none left
            remaining_amount -= amount;
            if remaining_amount < self.currency_precision {
                break;
            }
        }

        if remaining_amount > self.currency_precision {
            panic!("insufficient funds");
        };

        log_entries
    }
}