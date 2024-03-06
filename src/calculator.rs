use chrono::NaiveDate;
use iced::{
    executor,
    widget::{button, text, Column},
    Application, Command, Element, Settings, Theme,
};

use crate::{bitcoin::BitcoinAmount, numeric_input::numeric_input};

pub struct WhatIf {
    pub amount: Option<u32>,
    pub start_date: Option<NaiveDate>,
}

impl WhatIf {
    pub fn bitcoin_amount(&self) -> Option<BitcoinAmount> {
        let amount = self.amount? as u64;
        let _start_date = self.start_date?;
        let btc_price_then = 1234;

        amount.checked_mul(btc_price_then).map(BitcoinAmount::from)
    }

    pub fn current_usd_value(&self) -> Option<u64> {
        let sats = self.bitcoin_amount()?.sats();
        let btc_price_today = 100;

        sats.checked_mul(btc_price_today)
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
    AmountUpdated(Option<u32>),
}

impl Application for WhatIf {
    type Executor = executor::Default;
    type Flags = ();
    type Message = Message;
    type Theme = Theme;

    fn new(_flags: ()) -> (WhatIf, Command<Self::Message>) {
        (
            WhatIf {
                amount: None,
                start_date: None,
            },
            Command::none(),
        )
    }

    fn title(&self) -> String {
        String::from("What if...")
    }

    fn update(&mut self, message: Self::Message) -> Command<Self::Message> {
        match message {
            Message::DateSelected(date) => self.start_date = Some(date),
            Message::AmountUpdated(amount) => self.amount = amount,
        }
        Command::none()
    }

    fn view(&self) -> Element<Self::Message> {
        let col = Column::new()
            .max_width(600)
            .spacing(10)
            .padding(10)
            .align_items(iced::Alignment::Center)
            .push(numeric_input(self.amount, 10_000, Message::AmountUpdated))
            .push_maybe(
                self.amount
                    // TODO: proper formatting
                    .map(|amt| format!("${amt}"))
                    .map(text)
                    .map(|e| e.size(50)),
            )
            .push_maybe(self.bitcoin_amount().map(text).map(|e| e.size(50)))
            .push_maybe(
                self.current_usd_value()
                    // TODO: proper formatting
                    .map(|amt| format!("${amt}"))
                    .map(text)
                    .map(|e| e.size(100)),
            )
            .push(button("Date").on_press(Message::DateSelected(NaiveDate::default())));

        col.into()
    }
}
