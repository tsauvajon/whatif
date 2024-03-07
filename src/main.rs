mod bitcoin;
mod dollar;
mod historical_data;
mod numeric_input;
mod price_lookup;
mod ui;

// Currently disabled until I build a backend to hide the
// API key, that would get leaked in request headers.
//
// mod coinmarketcap;

#[tokio::main]
pub async fn main() -> iced::Result {
    ui::WhatIf::start()
}
