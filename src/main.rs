use calculator::WhatIf;

mod bitcoin;
mod calculator;
mod numeric_input;

pub fn main() -> iced::Result {
    WhatIf::start()
}
