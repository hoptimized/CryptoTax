use std::collections::{VecDeque};

use crate::accounting::{AccountingMethod, InventoryChange};

pub struct Inventory {
    layers: VecDeque<InventoryChange>,
    accounting_method: AccountingMethod,
    currency_precision: f64
}

impl Inventory {
    pub fn new(
        accounting_method: AccountingMethod,
        currency_precision: f64
    ) -> Inventory {
        Inventory {
            layers: VecDeque::new(),
            accounting_method,
            currency_precision,
        }
    }

    pub fn deposit(&mut self, inflow: InventoryChange) {
        // add layer to inventory
        match self.accounting_method {
            AccountingMethod::FIFO => self.layers.push_back(inflow),
            AccountingMethod::LIFO => self.layers.push_front(inflow),
        };
    }

    pub fn withdraw(&mut self, mut amount_to_withdraw: f64) -> Vec<InventoryChange> {
        let mut res: Vec<InventoryChange> = Vec::new();

        // withdraw assets layer by layer, using FIFO/LIFO accounting
        while let Some(layer) = self.layers.front_mut() {
            // determine costs
            let amount = amount_to_withdraw.min(layer.amount); // see how much we can take

            // add withdrawal to results
            res.push(InventoryChange {
                tx_id: layer.tx_id,
                datetime: layer.datetime,
                amount,
                base_price: layer.base_price,
            });

            // subtract amount from layer, remove layer if empty
            layer.amount -= amount;
            if layer.amount < self.currency_precision {
                self.layers.pop_front();
            }

            // subtract from remaining amount, exit if none left
            amount_to_withdraw -= amount;
            if amount_to_withdraw < self.currency_precision {
                break;
            }
        }

        if amount_to_withdraw > self.currency_precision {
            panic!("insufficient funds");
        };

        res
    }
}