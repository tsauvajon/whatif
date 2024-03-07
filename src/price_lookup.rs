//! First, basic version of a price database. It reads a CSV file containing
//! data until 06 March 2024, and is able to fetch today's price from the
//! coindesk.com API. Anything in between will be missing.
use std::{
    collections::HashMap,
    sync::{
        mpsc::{self, Receiver, Sender},
        Arc, RwLock,
    },
};

use chrono::{NaiveDate, Utc};
use serde::Deserialize;

use crate::{bitcoin::BitcoinAmount, dollar::DollarAmount, historical_data::get_prices_from_csv};

// Makes it work more easily on WASM + other platforms
const PRICE_HISTORY: &'static [u8] = include_bytes!("../data/price_history.csv");
const COINDESK_LATEST_BTC_PRICE: &str = "https://api.coindesk.com/v1/bpi/currentprice/USD.json";

/// Loads Bitcoin prices from different sources so we can look them up
pub struct PriceDatabase {
    pub data: Arc<RwLock<HashMap<NaiveDate, (DollarAmount, BitcoinAmount)>>>,
    updates_sender: Sender<NaiveDate>,
}

#[derive(Debug)]
pub enum Error {
    GetPricesFromCsv(csv::Error),
}

// Price in db: return it
// Not in DB: just fetch it and cache it!

impl PriceDatabase {
    /// First initial load of the database, and fetching of the price if not
    /// locally cached. The goal is to eventually load nothing at start, have a
    /// backend that stores the prices in a DB (+ cache in memory), and make
    /// this app query the backend. So we can gradually build a full history,
    /// use almost 0 request budget with API providers, and have a ready-to-use
    /// BTC price API for other purposes.
    pub fn start() -> Result<(Self, Receiver<NaiveDate>), Error> {
        let conversion_table = get_prices_from_csv(PRICE_HISTORY).map_err(|err| {
            println!("Loading conversion table: {err:?}");
            Error::GetPricesFromCsv(err)
        })?;
        println!(
            "Loaded conversion table with {} records.",
            conversion_table.len()
        );
        let conversion_table = Arc::new(RwLock::new(conversion_table));
        let (tx, rx): (Sender<NaiveDate>, Receiver<NaiveDate>) = mpsc::channel();
        let db = Self {
            data: conversion_table,
            updates_sender: tx,
        };

        // Cache today's rate, because we know we'll need it no matter what.
        let today = Utc::now().date_naive();
        db.get(today);

        Ok((db, rx))
    }

    pub fn get(&self, date: NaiveDate) -> Option<(DollarAmount, BitcoinAmount)> {
        println!("Received a price request for {date:?}");
        // Only get today's price from the API. A backend is needed
        // for more prices.

        if let Ok(Some(quote)) = self
            .data
            .read()
            .and_then(|data| Ok(data.get(&date).cloned()))
        {
            println!("We already have the price for this date!");
            return Some(quote);
        }

        if date != Utc::now().date_naive() {
            // This can be changed once we have our backend storing dates.
            println!("Not fetching data for past date");
            return None;
        }

        {
            let database = Arc::clone(&self.data);
            let tx = self.updates_sender.clone();

            let fut = async move {
                // We don't have today's price, let's fetch it!
                let resp = match reqwest::get(COINDESK_LATEST_BTC_PRICE).await {
                    Ok(resp) => resp,
                    Err(err) => {
                        println!("Fetching current BTC/USD quote: {err}");
                        return;
                    }
                };

                let resp = match resp.json::<CoinDeskBitcoinResponse>().await {
                    Ok(resp) => resp,
                    Err(err) => {
                        println!("Error parsing current BTC/USD response, good luck debugging the spaghetti code. {err}");
                        return;
                    }
                };

                let dollar_price = DollarAmount::from(resp.bpi.usd.rate_float);
                let bitcoin_price = BitcoinAmount::one_btc();

                match database.write() {
                    Ok(mut data) => {
                        data.insert(date, (dollar_price, bitcoin_price));
                        if let Err(err) = tx.send(date) {
                            println!("Send update upstream: {err}")
                        }
                    }
                    Err(err) => {
                        println!("Price database mutex is poisoned: {err}");
                    }
                };
            };

            // #[cfg(target_arch = "wasm32")]
            // iced::futures::executor::block_on(fut);
            #[cfg(not(target_arch = "wasm32"))]
            tokio::spawn(fut);
        }

        None
    }
}

#[derive(Deserialize)]
struct CoinDeskBitcoinResponse {
    bpi: Bpi,
}

#[derive(Deserialize)]
struct Bpi {
    #[serde(rename = "USD")]
    usd: Quote,
}

#[derive(Deserialize)]
struct Quote {
    rate_float: f64,
}
