use chrono::NaiveDate;

pub struct WhatIf {
    pub amount: Option<u64>,
    pub start_date: Option<NaiveDate>,
}

// USD amount
// Beginning date
// Get USD/BTC quote for the day

#[derive(Debug, Clone, Copy)]
pub enum Message {
    DateSelected(NaiveDate),
    AmountUpdated(u64),
}
