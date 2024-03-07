//! Join things together in an iced UI.

use std::sync::{mpsc::Receiver, Arc, Mutex};

use chrono::{NaiveDate, Utc};
use iced::{
    executor,
    widget::{text, Button, Column, Container, Text},
    Application, Command, Element, Length, Settings, Subscription, Theme,
};
use iced_aw::date_picker::Date;

use crate::{
    bitcoin::BitcoinAmount, dollar::DollarAmount, numeric_input::numeric_input,
    price_lookup::PriceDatabase,
};

pub struct WhatIf {
    amount: Option<DollarAmount>,
    show_date_picker: bool,
    start_date: Option<NaiveDate>,
    price_database: PriceDatabase,
    updates_receiver: Arc<Mutex<Receiver<NaiveDate>>>,
}

impl WhatIf {
    pub fn bitcoin_amount(&self) -> Option<BitcoinAmount> {
        let amount = self.amount?.dollars();
        let start_date = self.start_date?;
        let (usd, sats) = self.price_database.get(start_date)?;
        println!("Found the date's rates: {} = {}", usd, sats);

        amount
            .checked_mul(sats.sats())
            .and_then(|sats| sats.checked_div(usd.dollars()))
            .map(BitcoinAmount::from)
    }

    pub fn current_usd_value(&self) -> Option<DollarAmount> {
        let sats = self.bitcoin_amount()?.sats();
        let today = Utc::now().date_naive();
        let (usd, sats_rate) = self.price_database.get(today)?;
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
    PriceDatabaseUpdated(NaiveDate),
    AmountUpdated(Option<u64>),
}

impl Application for WhatIf {
    type Executor = executor::Default;
    type Flags = ();
    type Message = Message;
    type Theme = Theme;

    fn new(_flags: ()) -> (WhatIf, Command<Self::Message>) {
        let (price_database, updates_receiver) = PriceDatabase::start().unwrap();

        (
            WhatIf {
                amount: None,
                show_date_picker: false,
                start_date: None,
                price_database,
                updates_receiver: Arc::new(Mutex::new(updates_receiver)),
            },
            Command::none(),
        )
    }

    fn title(&self) -> String {
        String::from("What if...")
    }

    // https://stackoverflow.com/a/75689667
    fn subscription(&self) -> Subscription<Message> {
        iced::subscription::unfold(
            "price database updated",
            self.updates_receiver.clone(),
            move |receiver| async move {
                let date = receiver.lock().unwrap().recv().unwrap();
                (Message::PriceDatabaseUpdated(date), receiver)
            },
        )
    }

    fn update(&mut self, message: Self::Message) -> Command<Self::Message> {
        match message {
            Message::AmountUpdated(amount) => self.amount = amount.map(DollarAmount::from),
            Message::DateSelected(date) => {
                self.show_date_picker = false;
                self.start_date = Some(NaiveDate::from(date));
            }
            Message::PriceDatabaseUpdated(_) => (), // Just trigger an update but nothing else to do
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

    fn theme(&self) -> Theme {
        Theme::Dark
    }
}
