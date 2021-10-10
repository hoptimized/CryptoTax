use chrono::{DateTime, Utc};
use serde::{Deserialize};

use crate::{Inflow, Outflow};
use crate::inventory::Inventory;
use crate::prices::PriceInformation;

pub struct Parser<'a> {
    inventory: &'a mut Inventory,
    price_information: &'a mut PriceInformation,
    base_asset: String,
}

impl<'a> Parser<'a> {
    pub fn new(
        inventory: &'a mut Inventory,
        price_information: &'a mut PriceInformation,
        base_asset: String
    ) -> Parser<'a> {
        Parser {
            inventory,
            price_information,
            base_asset,
        }
    }

    pub fn parse_sheet(&mut self, path: &str) {
        let mut reader = csv::ReaderBuilder::new()
            .has_headers(false)
            .from_path(path)
            .unwrap();

        for (_, result) in reader
            .deserialize()
            .skip(1)
            .enumerate()
        {
            self.parse_row(result.unwrap());
        }
    }

    fn parse_row(&mut self, row: TransactionRecord) {
        match row.tx_type.as_str() {
            "Trade" => {
                let out_asset = row.out_asset.clone().unwrap();
                if row.in_asset != out_asset {
                    if out_asset == self.base_asset && row.in_asset != self.base_asset {
                        self.parse_trade_base_to_other(row);
                    } else if out_asset != self.base_asset && row.in_asset == self.base_asset {
                        self.parse_trade_other_to_base(row);
                    } else {
                        self.parse_trade_other_to_other(row);
                    }
                }
            },
            "Staking Reward" => {
                self.parse_staking_reward(row);
            },
            _ => {
                panic!("Unknown transaction type")
            }
        };
    }

    fn parse_trade_base_to_other(&mut self, record: TransactionRecord) {
        let out_amount = record.out_amount.unwrap();
        let base_value = out_amount;
        let base_price = base_value / record.in_amount;
        self.inventory.deposit(&record.in_asset, Inflow {
            tx_id: record.tx_id,
            datetime: record.datetime,
            amount: record.in_amount,
            base_price,
            actual_costs: base_price * record.in_amount
        });
    }

    fn parse_trade_other_to_base(&mut self, record: TransactionRecord) {
        let out_asset = record.out_asset.unwrap();
        let out_amount = record.out_amount.unwrap();
        self.inventory.withdraw(&out_asset, Outflow {
            tx_id: record.tx_id,
            datetime: record.datetime,
            amount: out_amount,
            proceeds: record.in_amount,
        });
    }

    fn parse_trade_other_to_other(&mut self, record: TransactionRecord) {
        let out_asset = record.out_asset.unwrap();
        let out_amount = record.out_amount.unwrap();
        let out_base_price = self.price_information.get(
            &out_asset,
            &self.base_asset,
            record.datetime);
        let base_value = out_base_price * out_amount;
        let in_base_price = base_value / record.in_amount;
        self.inventory.withdraw(&out_asset, Outflow {
            tx_id: record.tx_id,
            datetime: record.datetime,
            amount: out_amount,
            proceeds: base_value,
        });
        self.inventory.deposit(&record.in_asset, Inflow {
            tx_id: record.tx_id,
            datetime: record.datetime,
            amount: record.in_amount,
            base_price: in_base_price,
            actual_costs: in_base_price * record.in_amount,
        });
    }

    fn parse_staking_reward(&mut self, record: TransactionRecord) {
        let reward_base_price = self.price_information.get(
            record.in_asset.as_str(),
            &self.base_asset,
            record.datetime);

        self.inventory.deposit(&record.in_asset, Inflow {
            tx_id: record.tx_id,
            datetime: record.datetime,
            amount: record.in_amount,
            base_price: reward_base_price,
            actual_costs: 0f64,
        });
    }
}

#[derive(Debug, Deserialize)]
struct TransactionRecord {
    tx_id: u32,
    datetime: DateTime<Utc>,
    account: String,
    tx_type: String,
    out_asset: Option<String>,
    out_amount: Option<f64>,
    in_asset: String,
    in_amount: f64,
    fee_asset: Option<String>,
    fee_amount: Option<f64>,
}
