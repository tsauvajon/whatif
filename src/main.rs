mod bitcoin;
mod dollar;
mod numeric_input;
mod ui;

pub fn main() -> iced::Result {
    ui::WhatIf::start()
}
