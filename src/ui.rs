use std::{
    collections::HashMap,
    fs::File,
    sync::{
        mpsc::{self, Receiver, Sender},
        Arc, RwLock,
    },
    thread,
};

use chrono::{NaiveDate, Utc};
use iced::{
    executor,
    widget::{text, Button, Column, Container, Text},
    Application, Command, Element, Length, Settings, Theme,
};
use iced_aw::date_picker::Date;

use crate::{
    bitcoin::BitcoinAmount, dollar::DollarAmount, historical_data::get_prices_from_csv,
    numeric_input::numeric_input,
};

pub struct WhatIf {
    amount: Option<DollarAmount>,
    show_date_picker: bool,
    start_date: Option<NaiveDate>,
    conversion_table: Arc<RwLock<HashMap<NaiveDate, (DollarAmount, BitcoinAmount)>>>,
    send_channel: Sender<NaiveDate>,
}

impl WhatIf {
    pub fn bitcoin_amount(&self) -> Option<BitcoinAmount> {
        let amount = self.amount?.dollars();
        let start_date = self.start_date?;
        let conversion_table = self.conversion_table.read().ok()?;
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
        let conversion_table = self.conversion_table.read().ok()?;
        let (usd, sats_rate) = conversion_table.get(&today)?;
        println!("Found today's rates: {} = {}", usd, sats_rate);

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
    ToggleDatePicker(bool),
    DateSelected(Date),
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
                show_date_picker: false,
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
            Message::AmountUpdated(amount) => self.amount = amount.map(DollarAmount::from),
            Message::DateSelected(date) => {
                let date = NaiveDate::from(date);
                self.start_date = Some(date);
                self.show_date_picker = false;
                self.send_channel
                    .send(date)
                    .map_err(|err| {
                        println!("{err:?}");
                        err
                    })
                    .unwrap(); // TODO handle unwrap?
            }
            Message::ToggleDatePicker(toggle) => self.show_date_picker = toggle,
        }
        Command::none()
    }

    fn view(&self) -> Element<Self::Message> {
        let col = Column::new()
            .max_width(600)
            .spacing(10)
            .padding(10)
            .align_items(iced::Alignment::Center)
            .push(iced_aw::date_picker(
                self.show_date_picker,
                NaiveDate::from_ymd_opt(2019, 1, 1).unwrap(),
                Button::new(Text::new("Date")).on_press(Message::ToggleDatePicker(true)),
                Message::ToggleDatePicker(false),
                Message::DateSelected,
            ))
            .push(numeric_input(
                self.amount.map(DollarAmount::dollars),
                10_000,
                Message::AmountUpdated,
            ))
            .push_maybe(
                self.amount
                    .and_then(|amt| {
                        self.start_date.and_then(|date| {
                            self.bitcoin_amount().map(|btc| {
                                format!(
                                    "If you converted your entire net worth of {amt} into {btc} on {date}"
                                )
                            })
                        })
                    })
                    .map(text)
                    .map(|e| e.size(30)),
            )
            // .push_maybe(
            //     self.bitcoin_amount()
            //         .map(|amt| format!("You would have {amt}"))
            //         .map(text)
            //         .map(|e| e.size(30)),
            // )
            // .push(text(
            //     "(Historical data from https://www.investing.com/crypto/bitcoin/historical-data)",
            // ))
            .push_maybe(
                self.current_usd_value()
                    .map(|amt| format!("Your net worth today would be {amt}"))
                    .map(text)
                    .map(|e| e.size(50)),
            );

        Container::new(col)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x()
            .center_y()
            .into()
    }
}
