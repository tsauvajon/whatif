//! Historical data from https://www.investing.com/crypto/bitcoin/historical-data
//! CSV until 03 06 2024
//! TODO: store everything in a DB, and fetch today's value from an API somewhere
use std::collections::HashMap;

use chrono::NaiveDate;
use serde::{Deserialize, Deserializer};

use crate::{bitcoin::BitcoinAmount, dollar::DollarAmount};

#[derive(Deserialize)]
//"Date","Price","Open","High","Low","Vol.","Change %"
struct CsvRecord {
    #[serde(deserialize_with = "deserialize_date::deserialize", rename = "Date")]
    date: NaiveDate,
    #[serde(deserialize_with = "deserialize_amount", rename = "Price")]
    price: DollarAmount,
    #[serde(deserialize_with = "deserialize_amount", rename = "Open")]
    _open: DollarAmount,
    #[serde(deserialize_with = "deserialize_amount", rename = "High")]
    _high: DollarAmount,
    #[serde(deserialize_with = "deserialize_amount", rename = "Low")]
    _low: DollarAmount,
    #[serde(rename = "Vol.")]
    _volume: String, // e.g. 203.36K
    #[serde(rename = "Change %")]
    _change_percent: String, // e.g. 5.03%
}

pub fn get_prices_from_csv(
    reader: impl std::io::Read,
) -> Result<HashMap<NaiveDate, (DollarAmount, BitcoinAmount)>, csv::Error> {
    let mut reader = csv::Reader::from_reader(reader);
    let mut records = HashMap::new();
    for record in reader.deserialize() {
        let record: CsvRecord = record?;
        records.insert(
            record.date,
            (DollarAmount::from(record.price), BitcoinAmount::one_btc()),
        );
    }

    Ok(records)
}

pub fn deserialize_amount<'de, D>(deserializer: D) -> Result<DollarAmount, D::Error>
where
    D: Deserializer<'de>,
{
    let s: String = Deserialize::deserialize(deserializer)?;
    s.replace(",", "")
        .parse::<f64>()
        .map_err(serde::de::Error::custom)
        .map(DollarAmount::from)
}

mod deserialize_date {
    use chrono::NaiveDate;
    use serde::{self, Deserialize, Deserializer};

    const FORMAT: &'static str = "%m/%d/%Y";

    pub fn deserialize<'de, D>(deserializer: D) -> Result<NaiveDate, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        NaiveDate::parse_from_str(&s, FORMAT).map_err(serde::de::Error::custom)
    }
}

#[test]
fn test_loading_csv() {
    let f = std::fs::File::open("data/price_history.csv").unwrap();
    let conversion_table = get_prices_from_csv(f).unwrap();
    assert!(!conversion_table.is_empty())
}
