use std::collections::HashMap;

use reqwest::{
    header::{HeaderMap, HeaderValue},
    Client,
};
use serde::Deserialize;

pub const PROD_DOMAIN: &str = "pro-api.coinmarketcap.com";
const ENDPOINT_URL: &str = "v2/cryptocurrency/quotes/latest?symbol=BTC";
const AUTH_HEADER: &str = "X-CMC_PRO_API_KEY";
const BTC_SYMBOL: &str = "BTC";
const USD_SYMBOL: &str = "USD";

pub struct CoinMarketCapClient {
    domain: String,
    api_key: String,
}

impl CoinMarketCapClient {
    pub fn new(domain: &str, api_key: &str) -> Self {
        Self {
            domain: domain.to_string(),
            api_key: api_key.to_string(),
        }
    }

    pub async fn get_bitcoin_usd_price(&self) -> Result<u64, Error> {
        let client = Client::new();
        let mut headers = HeaderMap::new();
        headers.insert(
            AUTH_HEADER,
            HeaderValue::from_str(&self.api_key).map_err(Error::InvalidHeader)?,
        );

        let response = client
            .get(format!("https://{}/{ENDPOINT_URL}", self.domain))
            .headers(headers)
            .send()
            .await
            .map_err(Error::Http)?
            .json::<Response>()
            .await
            .map_err(Error::Parse)?;

        if response.status.error_code != 0 {
            return Err(Error::CoinMarketCapError(response.status));
        }

        let data = response.data.ok_or(Error::MissingData);
        let data = data?;
        let bitcoin_info = data.get(BTC_SYMBOL).ok_or(Error::MissingBitcoinResponse)?;

        let btc_quotes = bitcoin_info
            .iter()
            .find(|data| data.symbol == BTC_SYMBOL)
            .ok_or(Error::MissingBitcoinData)?;
        let btc_usd_price = btc_quotes
            .quote
            .get(USD_SYMBOL)
            .ok_or(Error::MissingUsdQuote)?
            .price;

        Ok(btc_usd_price as u64)
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
struct Response {
    status: Status,
    data: Option<HashMap<String, Vec<Data>>>,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct Status {
    error_code: u64,           // 0 = OK
    credit_count: Option<u64>, // cost, 10k/month allowed
    pub error_message: Option<String>,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
struct Data {
    symbol: String,
    quote: HashMap<String, Quote>,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
struct Quote {
    price: f64,
}

#[derive(Debug)]
pub enum Error {
    InvalidHeader(reqwest::header::InvalidHeaderValue),
    Http(reqwest::Error),
    Parse(reqwest::Error),
    CoinMarketCapError(Status),
    MissingData,
    MissingBitcoinResponse,
    MissingBitcoinData,
    MissingUsdQuote,
}

#[tokio::test]
async fn test_get_usd_price() {
    let sandbox_domain = "sandbox-api.coinmarketcap.com";
    let sandbox_api_key = "b54bcf4d-1bca-4e8e-9a24-22ff2c3d462c";

    let client = CoinMarketCapClient::new(sandbox_domain, sandbox_api_key);

    let price = client.get_bitcoin_usd_price().await.unwrap();
    assert_eq!(0, price);
}
