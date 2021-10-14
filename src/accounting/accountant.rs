use crate::prices::PriceInformation;
use crate::accounting::{AccountingMethod, calculation};
use crate::accounting::reports::CapitalGainsReport;

pub struct Accountant<'a> {
    price_information: &'a mut PriceInformation,
    accounting_method: AccountingMethod,
    base_asset: &'a str,
    currency_precision: f64,
}

impl<'a> Accountant<'a> {
    pub fn new(price_information: &'a mut PriceInformation) -> Accountant<'a> {
        Accountant {
            price_information,
            accounting_method: AccountingMethod::FIFO,
            base_asset: "EUR",
            currency_precision: 0.00000001f64,
        }
    }

    pub fn method(&mut self, method: AccountingMethod) -> &'a mut Accountant {
        self.accounting_method = method;
        self
    }

    pub fn base_asset(&mut self, base_asset: &'a str) -> &'a mut Accountant {
        self.base_asset = base_asset;
        self
    }

    pub fn precision(&mut self, precision: f64) -> &'a mut Accountant {
        self.currency_precision = precision;
        self
    }

    pub fn analyze_file(&mut self, path: &str) -> CapitalGainsReport {
        let report = calculation::calculate_capital_gains(
            path,
            self.price_information,
            self.accounting_method,
            self.base_asset,
            self.currency_precision,
        );
        CapitalGainsReport::new(report)
    }
}