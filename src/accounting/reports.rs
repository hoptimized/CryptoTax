use csv::Writer;

use crate::accounting::CashflowRecord;

pub struct CapitalGainsReport {
    records: Vec<CashflowRecord>,
}

impl CapitalGainsReport {
    pub fn new(records : Vec<CashflowRecord>) -> CapitalGainsReport {
        CapitalGainsReport {
            records,
        }
    }

    pub fn write_to_file(&self, path: &str) {
        let mut writer = Writer::from_path(path.to_string()).unwrap();
        for entry in self.records.iter() {
            writer.serialize(entry).unwrap();
        }
    }
}