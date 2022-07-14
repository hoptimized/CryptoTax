mod inventory;

use std::collections::HashMap;

use crate::prices::PriceInformation;
use crate::accounting::{AccountingMethod, CashflowRecord, Purchase, InventoryChange, Sale, TransactionRecord, Withdrawal};
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

    for (i, record) in records
    {
        println!("Processing record #{}", i + 1);
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
                    if out_asset == self.base_asset || row.in_asset == self.base_asset {
                        self.process_trade_simple(row);
                    } else {
                        self.process_trade_foreign_to_foreign(row);
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

    fn process_trade_simple(
        &mut self,
        record: TransactionRecord
    ) {
        // a simple trade involves the base asset; it constitutes either:
        //  A) a FOREX purchase (base to foreign)
        //  B) a FOREX sale (foreign to base)

        // extract trade parameters
        let out_asset = record.out_asset.unwrap();
        let in_asset = record.in_asset;
        let mut out_amount = record.out_amount.unwrap();
        let mut in_amount = record.in_amount;

        // trade must involve the base asset and one foreign asset
        if (in_asset != self.base_asset && out_asset != self.base_asset)
            || (in_asset == out_asset)
        {
            return;
        }

        // calculate fees, if any
        if let (Some(fee_asset), Some(fee_amount)) = (record.fee_asset, record.fee_amount) {
            if fee_asset == out_asset {
                // add fees to expenses
                out_amount += fee_amount;
            } else if fee_asset == in_asset {
                // deduct fees from proceeds
                in_amount -= fee_amount;
            } else {
                // convert fee into base asset and attribute to base_asset
                let fee_base_value = self.process_withdrawal(&fee_asset, Withdrawal {
                    tx_id: record.tx_id,
                    datetime: record.datetime,
                    amount: fee_amount,
                });
                if out_asset == self.base_asset {
                    out_amount += fee_base_value;
                } else if in_asset == self.base_asset {
                    in_amount -= fee_base_value;
                }
            }
        }

        // process the purchase / sale
        if out_asset == self.base_asset {
            self.process_purchase(&in_asset, Purchase {
                tx_id: record.tx_id,
                datetime: record.datetime,
                amount: in_amount,
                base_price: out_amount / in_amount,
                actual_costs: out_amount,
            });
        } else if in_asset == self.base_asset {
            self.process_sale(&out_asset, Sale {
                tx_id: record.tx_id,
                datetime: record.datetime,
                amount: out_amount,
                proceeds: in_amount,
            });
        }
    }

    fn process_trade_foreign_to_foreign(
        &mut self,
        record: TransactionRecord
    ) {
        // sell a foreign asset to receive a foreign asset;
        // we need to split this up into two transactions:
        //  1) sell foreign asset for base asset, at market price
        //  2) sell base asset for the other foreign asset

        // unwrap out asset
        let out_asset = record.out_asset.clone().unwrap();
        let out_amount = record.out_amount.unwrap();

         // query market price for the asset that we'd like to dispose of
        let out_base_price = self.price_information.get(
            &out_asset,
            &self.base_asset,
            record.datetime);

        // calculate value (in base asset terms) of the transaction, net of fees
        let out_base_value = out_base_price * out_amount;

        // distribute fees, if any
        let (sale_fee_amount, purchase_fee_amount) = match (record.fee_asset.clone(), record.fee_amount) {
            (Some(fee_asset), Some(fee_amount)) => {
                if fee_asset == out_asset {
                    (Some(fee_amount), Some(0f64))
                } else if fee_asset == record.in_asset {
                    (Some(0f64), Some(fee_amount))
                } else {
                    (Some(fee_amount / 2f64), Some(fee_amount / 2f64))
                }
            },
            _ => (None, None),
        };

        // sell out_asset for base_asset
        self.process_trade_simple(TransactionRecord {
            out_asset: record.out_asset.clone(),
            out_amount: record.out_amount,
            in_asset: self.base_asset.to_string(),
            in_amount: out_base_value,
            fee_asset: record.fee_asset.clone(),
            fee_amount: sale_fee_amount,
            ..record.clone()
        });

        // buy in_asset with base_asset
        self.process_trade_simple(TransactionRecord {
            out_asset: Some(self.base_asset.to_string()),
            out_amount: Some(out_base_value),
            in_asset: record.in_asset.clone(),
            in_amount: record.in_amount,
            fee_asset: record.fee_asset.clone(),
            fee_amount: purchase_fee_amount,
            ..record.clone()
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
        self.process_purchase(&record.in_asset, Purchase {
            tx_id: record.tx_id,
            datetime: record.datetime,
            amount: record.in_amount,
            base_price: reward_base_price,
            actual_costs: 0f64,
        });
    }

    pub fn process_purchase(&mut self, asset: &str, purchase: Purchase) {
        let inventory = self.assets
            .entry(asset.to_string())
            .or_insert(Inventory::new(
                self.accounting_method.clone(),
                self.currency_precision));

        // deposit asset
        inventory.deposit(InventoryChange {
            tx_id: purchase.tx_id,
            datetime: purchase.datetime,
            amount: purchase.amount,
            base_price: purchase.base_price,
        });

        // calculate possible gains from discounts / gifts / staking rewards etc.
        let gains_raw = purchase.amount * purchase.base_price - purchase.actual_costs;
        let gains : Option<f64> = match gains_raw >= self.currency_precision {
            true => Some(gains_raw),
            false => None,
        };

        // create log entry
        self.log.push(CashflowRecord {
            asset: asset.to_string(),
            tx_out: None,
            datetime_out: None,
            tx_in: purchase.tx_id,
            datetime_in: purchase.datetime,
            amount: purchase.amount,
            base_price: purchase.base_price,
            actual_costs: purchase.actual_costs,
            actual_proceeds: None,
            gains_short_term: gains,
            gains_long_term: None,
        });
    }

    pub fn process_sale(&mut self, asset: &str, sale: Sale) {
        let inventory = self.assets
            .entry(asset.to_string())
            .or_insert(Inventory::new(
                self.accounting_method.clone(),
                self.currency_precision));

        // withdraw asset from inventory
        let outflows = inventory.withdraw(sale.amount);

        // loop through outflows
        for outflow in outflows {
            let costs = outflow.base_price * outflow.amount;

            // calculate proceeds proportional to the amount that we are taking from the layer;
            // if we are taking close to zero, put all proceeds on this layer
            let proceeds = match sale.amount >= self.currency_precision {
                true => sale.proceeds / sale.amount * outflow.amount,
                false => sale.proceeds,
            };

            // calculate gains
            let gains = proceeds - costs;

            // calculate holding duration
            let duration = sale.datetime.signed_duration_since(outflow.datetime).num_days();
            let is_longterm = duration > 365;

            // submit log entry
            self.log.push(CashflowRecord {
                asset: asset.to_string(),
                tx_out: Some(sale.tx_id),
                datetime_out: Some(sale.datetime),
                tx_in: outflow.tx_id,
                datetime_in: outflow.datetime,
                amount: -outflow.amount,
                base_price: outflow.base_price,
                actual_costs: costs,
                actual_proceeds: Some(proceeds),
                gains_short_term: if is_longterm {None} else {Some(gains)},
                gains_long_term: if is_longterm {Some(gains)} else {None},
            });
        }
    }

    fn process_withdrawal(&mut self, asset: &str, withdrawal: Withdrawal) -> f64 {
        // gain-neutral withdrawal of assets from inventory;
        // primarily used to withdraw assets to pay for fees

        // find the correct inventory
        let inventory = self.assets
            .entry(asset.to_string())
            .or_insert(Inventory::new(
                self.accounting_method.clone(),
                self.currency_precision));

        // withdraw asset from inventory
        let outflows = inventory.withdraw(withdrawal.amount);

        // loop through outflows; generate logs and calculate total costs
        let mut total_costs = 0f64;
        for outflow in outflows {
            let costs = outflow.amount * outflow.base_price;
            total_costs += costs;

            // submit log entry
            self.log.push(CashflowRecord {
                asset: asset.to_string(),
                tx_out: Some(withdrawal.tx_id),
                datetime_out: Some(withdrawal.datetime),
                tx_in: outflow.tx_id,
                datetime_in: outflow.datetime,
                amount: -outflow.amount,
                base_price: outflow.base_price,
                actual_costs: costs,
                actual_proceeds: Some(costs),
                gains_short_term: None,
                gains_long_term: None,
            });
        }

        total_costs
    }
}
