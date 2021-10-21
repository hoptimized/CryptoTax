pub mod accountant;
pub mod reports;
mod calculation;

use chrono::{DateTime, Utc};
use serde::{Serialize,Deserialize};

#[derive(Clone, Debug, Deserialize)]
pub struct TransactionRecord {
    tx_id: u32,
    datetime: DateTime<Utc>,
    account: String,
    tx_type: String, // TODO: refactor to enum
    out_asset: Option<String>,
    out_amount: Option<f64>,
    in_asset: String,
    in_amount: f64,
    fee_asset: Option<String>,
    fee_amount: Option<f64>,
}

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

#[derive(Copy, Clone, Debug, Deserialize)]
pub enum AccountingMethod {
    FIFO,
    LIFO,
}

#[derive(Copy, Clone)]
pub struct Purchase {
    tx_id: u32,
    datetime: DateTime<Utc>,
    amount: f64,
    base_price: f64,
    actual_costs: f64,
}

#[derive(Copy, Clone)]
pub struct Sale {
    tx_id: u32,
    datetime: DateTime<Utc>,
    amount: f64,
    proceeds: f64,
}

#[derive(Copy, Clone)]
pub struct InventoryChange {
    tx_id: u32,
    datetime: DateTime<Utc>,
    amount: f64,
    base_price: f64,
}

#[derive(Copy, Clone)]
pub struct Withdrawal {
    tx_id: u32,
    datetime: DateTime<Utc>,
    amount: f64,
}