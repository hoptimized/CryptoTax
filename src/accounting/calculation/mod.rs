mod inventory;

use std::collections::HashMap;

use crate::prices::PriceInformation;
use crate::accounting::{AccountingMethod, CashflowRecord, Inflow, Outflow, TransactionRecord};
use crate::accounting::calculation::inventory::Inventory;

pub fn calculate_capital_gains<'a>(
    records_path: &'a str,
    price_information: &'a mut PriceInformation,
    accounting_method: AccountingMethod,
    base_asset: &'a str,
    currency_precision: f64,
) -> Vec<CashflowRecord> {
    let mut calculation = CapitalGainsCalculation::new(
        price_information,
        accounting_method,
        base_asset,
        currency_precision,
    );

    let mut reader = csv::ReaderBuilder::new()
        .has_headers(false)
        .from_path(records_path)
        .unwrap();

    let records = reader
        .deserialize::<TransactionRecord>()
        .skip(1)
        .enumerate();

    for (_, record) in records
    {
        calculation.process_record(record.unwrap());
    }

    calculation.log
}

struct CapitalGainsCalculation<'a> {
    price_information: &'a mut PriceInformation,
    assets: HashMap<String, Inventory>,
    log: Vec<CashflowRecord>,
    accounting_method: AccountingMethod,
    base_asset: &'a str,
    currency_precision: f64,
}

impl<'a> CapitalGainsCalculation<'a> {
    fn new(
        price_information: &'a mut PriceInformation,
        accounting_method: AccountingMethod,
        base_asset: &'a str,
        currency_precision: f64,
    ) -> CapitalGainsCalculation<'a> {
        CapitalGainsCalculation {
            price_information,
            accounting_method,
            base_asset,
            currency_precision,
            assets: HashMap::new(),
            log: Vec::new(),
        }
    }

    fn process_record(&mut self, row: TransactionRecord) {
        match row.tx_type.as_str() {
            "Trade" => {
                let out_asset = row.out_asset.clone().unwrap();
                if row.in_asset != out_asset {
                    if out_asset == self.base_asset && row.in_asset != self.base_asset {
                        self.process_trade_base_to_other(row);
                    } else if out_asset != self.base_asset && row.in_asset == self.base_asset {
                        self.process_trade_other_to_base(row);
                    } else {
                        self.process_trade_other_to_other(row);
                    }
                }
            },
            "Staking Reward" => {
                self.process_staking_reward(row);
            },
            _ => {
                panic!("Unknown transaction type")
            }
        };
    }

    fn process_trade_base_to_other(
        &mut self,
        record: TransactionRecord
    ) {
        // selling the base asset to receive another asset;
        // thus, the base asset's amount (out_amount) defines the base_value
        // of the transaction

        let out_amount = record.out_amount.unwrap();
        let base_value = out_amount;
        let base_price = base_value / record.in_amount;

        // record the inflow of the other asset;
        // skip recording the base asset, because it is neutral on gains
        self.deposit(&record.in_asset, Inflow {
            tx_id: record.tx_id,
            datetime: record.datetime,
            amount: record.in_amount,
            base_price,
            actual_costs: base_price * record.in_amount
        });
    }

    fn process_trade_other_to_base(
        &mut self,
        record: TransactionRecord
    ) {
        // selling another asset in exchange for the base asset;
        // thus, the base asset's amount (in_amount) is the proceeds

        let out_asset = record.out_asset.unwrap();
        let out_amount = record.out_amount.unwrap();

        // record the outflow of the other asset;
        // skip recording the base asset, because it is neutral on gains
        self.withdraw(&out_asset, Outflow {
            tx_id: record.tx_id,
            datetime: record.datetime,
            amount: out_amount,
            proceeds: record.in_amount,
        });
    }

    fn process_trade_other_to_other(
        &mut self,
        record: TransactionRecord
    ) {
        // sell another asset to receive another asset;
        // we need to split this up into two transactions:
        //  1) sell other asset for base asset, at market price
        //  2) sell base asset for the target asset

        // unwrap optional parameters
        let out_asset = record.out_asset.unwrap();
        let out_amount = record.out_amount.unwrap();

        // query market price for the asset that we'd like to dispose of
        let out_base_price = self.price_information.get(
            &out_asset,
            &self.base_asset,
            record.datetime);

        // calculate value (in base asset terms) of the transaction
        let base_value = out_base_price * out_amount;

        // calculate price of the asset that we receive, against base asset
        let in_base_price = base_value / record.in_amount;

        // record the outflow of the disposed asset
        self.withdraw(&out_asset, Outflow {
            tx_id: record.tx_id,
            datetime: record.datetime,
            amount: out_amount,
            proceeds: base_value,
        });

        // record the inflow of the received asset
        self.deposit(&record.in_asset, Inflow {
            tx_id: record.tx_id,
            datetime: record.datetime,
            amount: record.in_amount,
            base_price: in_base_price,
            actual_costs: in_base_price * record.in_amount,
        });
    }

    fn process_staking_reward(
        &mut self,
        record: TransactionRecord
    ) {
        // receive an asset without any costs;
        // record the inflow at the asset's market price

        // query market price of inflowing asset
        let reward_base_price = self.price_information.get(
            record.in_asset.as_str(),
            &self.base_asset,
            record.datetime);

        // record the inflow
        self.deposit(&record.in_asset, Inflow {
            tx_id: record.tx_id,
            datetime: record.datetime,
            amount: record.in_amount,
            base_price: reward_base_price,
            actual_costs: 0f64,
        });
    }

    pub fn deposit(&mut self, asset: &str, inflow: Inflow) {
        let inventory = self.assets
            .entry(asset.to_string())
            .or_insert(Inventory::new(
                asset.to_string(),
                self.accounting_method.clone(),
                self.currency_precision));
        let log_entry = inventory.deposit(inflow);
        self.log.push(log_entry);
    }

    pub fn withdraw(&mut self, asset: &str, outflow: Outflow) {
        let inventory = self.assets
            .entry(asset.to_string())
            .or_insert(Inventory::new(
                asset.to_string(),
                self.accounting_method.clone(),
                self.currency_precision));
        let mut log_entries = inventory.withdraw(outflow);
        self.log.append(&mut log_entries);
    }
}
