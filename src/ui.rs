use std::{
    collections::HashMap,
    fs::File,
    sync::{
        mpsc::{self, Receiver, Sender},
        Arc, RwLock,
    },
    thread,
};

use chrono::{Days, NaiveDate, Utc};
use iced::{
    executor,
    widget::{button, text, Column},
    Application, Command, Element, Settings, Theme,
};

use crate::{
    bitcoin::BitcoinAmount, dollar::DollarAmount, historical_data::get_prices_from_csv,
    numeric_input::numeric_input,
};

pub struct WhatIf {
    amount: Option<DollarAmount>,
    start_date: Option<NaiveDate>,
    conversion_table: Arc<RwLock<HashMap<NaiveDate, (DollarAmount, BitcoinAmount)>>>,
    send_channel: Sender<NaiveDate>,
}

impl WhatIf {
    pub fn bitcoin_amount(&self) -> Option<BitcoinAmount> {
        let amount = self.amount?.dollars();
        let start_date = self.start_date?;
        let conversion_table = self.conversion_table.read().ok()?;
        println!(
            "Conversion table contains {} records.",
            conversion_table.len()
        );
        let (usd, sats) = conversion_table.get(&start_date)?;
        println!("Found the date's rates: {} = {}", usd, sats);

        amount
            .checked_mul(sats.sats())
            .and_then(|sats| sats.checked_div(usd.dollars()))
            .map(BitcoinAmount::from)
    }

    pub fn current_usd_value(&self) -> Option<DollarAmount> {
        let sats = self.bitcoin_amount()?.sats();
        let today = Utc::now().date_naive();
        let yesterday = today.checked_sub_days(Days::new(1))?;
        let conversion_table = self.conversion_table.read().ok()?;
        println!(
            "Conversion table contains {} records",
            conversion_table.len()
        );
        let (usd, sats_rate) = conversion_table.get(&yesterday)?;
        println!("Found yesterday's rates: {} = {}", usd, sats_rate);

        sats.checked_mul(usd.dollars())
            .and_then(|usd| usd.checked_div(sats_rate.sats()))
            .map(DollarAmount::from)
    }

    pub fn start() -> Result<(), iced::Error> {
        WhatIf::run(Settings::default())
    }
}

// USD amount
// Beginning date
// Get USD/BTC quote for the day

#[derive(Debug, Clone, Copy)]
pub enum Message {
    DateSelected(NaiveDate),
    AmountUpdated(Option<u64>),
}

impl Application for WhatIf {
    type Executor = executor::Default;
    type Flags = ();
    type Message = Message;
    type Theme = Theme;

    fn new(_flags: ()) -> (WhatIf, Command<Self::Message>) {
        let (send_channel, rx): (Sender<NaiveDate>, Receiver<NaiveDate>) = mpsc::channel();
        let (_tx, receive_channel): (
            Sender<(NaiveDate, DollarAmount, BitcoinAmount)>,
            Receiver<(NaiveDate, DollarAmount, BitcoinAmount)>,
        ) = mpsc::channel();

        let f = File::open("data/price_history.csv").unwrap();
        let conversion_table = get_prices_from_csv(f)
            .map_err(|err| {
                println!("Loading conversion table: {err:?}");
                err
            })
            .unwrap_or_default();
        println!(
            "Loaded conversion table with {} records.",
            conversion_table.len()
        );
        let conversion_table = Arc::new(RwLock::new(conversion_table));

        {
            let conversion_table = Arc::clone(&conversion_table);
            thread::spawn(move || loop {
                let Ok((date, usd, sats)) = receive_channel.recv() else {
                    break;
                };

                let Ok(mut table) = conversion_table.write() else {
                    break;
                };
                table.insert(date, (usd, sats));
            });
        }

        thread::spawn(move || loop {
            if let Ok(date) = rx.recv() {
                println!("Received date {date:?}");
            } else {
                break;
            }
        });

        (
            WhatIf {
                amount: None,
                start_date: None,
                conversion_table,
                send_channel,
            },
            Command::none(),
        )
    }

    fn title(&self) -> String {
        String::from("What if...")
    }

    fn update(&mut self, message: Self::Message) -> Command<Self::Message> {
        match message {
            Message::DateSelected(date) => {
                self.start_date = Some(date);
                self.send_channel
                    .send(date)
                    .map_err(|err| {
                        println!("{err:?}");
                        err
                    })
                    .unwrap(); // TODO handle unwrap?
            }
            Message::AmountUpdated(amount) => self.amount = amount.map(DollarAmount::from),
        }
        Command::none()
    }

    fn view(&self) -> Element<Self::Message> {
        Column::new()
            .max_width(600)
            .spacing(10)
            .padding(10)
            .align_items(iced::Alignment::Center)
            .push(button("Date").on_press(Message::DateSelected(
                // TODO date selector
                NaiveDate::from_ymd_opt(2019, 1, 1).unwrap_or_default(),
            )))
            .push(numeric_input(
                self.amount.map(DollarAmount::dollars),
                10_000,
                Message::AmountUpdated,
            ))
            .push_maybe(
                self.amount
                    .map(DollarAmount::from)
                    .map(text)
                    .map(|e| e.size(50)),
            )
            .push_maybe(self.bitcoin_amount().map(text).map(|e| e.size(50)))
            .push_maybe(self.current_usd_value().map(text).map(|e| e.size(100)))
            .into()
    }
}
