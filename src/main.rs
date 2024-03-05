use calculator::{Message, WhatIf};
use iced::{executor, Application, Command, Element, Settings, Theme};

mod calculator;

pub fn main() -> iced::Result {
    WhatIf::run(Settings::default())
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
        String::from("A cool application")
    }

    fn update(&mut self, message: Self::Message) -> Command<Self::Message> {
        match message {
            Message::DateSelected(date) => self.start_date = Some(date),
            Message::AmountUpdated(amount) => self.amount = Some(amount),
        }
        Command::none()
    }

    fn view(&self) -> Element<Self::Message> {
        // We use a column: a simple vertical layout
        column![
            // The increment button. We tell it to produce an
            // `IncrementPressed` message when pressed
            button("+").on_press(Message::IncrementPressed),
            // We show the value of the counter here
            text(self.value).size(50),
            // The decrement button. We tell it to produce a
            // `DecrementPressed` message when pressed
            button("-").on_press(Message::DecrementPressed),
        ]
    }
}
