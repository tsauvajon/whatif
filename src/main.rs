mod bitcoin;
mod dollar;
mod historical_data;
mod numeric_input;
mod ui;

pub fn main() -> iced::Result {
    ui::WhatIf::start()
}
