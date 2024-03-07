mod bitcoin;
mod dollar;
mod historical_data;
mod numeric_input;
mod price_lookup;
mod ui;

// Currently disabled until I build a backend to hide the
// API key, that would get leaked in request headers.
//
// mod coinmarketcap;

#[cfg(target_arch = "wasm32")]
pub fn main() -> iced::Result {
    std::panic::set_hook(Box::new(console_error_panic_hook::hook));
    ui::WhatIf::start()
}

#[cfg(not(target_arch = "wasm32"))]
#[tokio::main]
pub async fn main() -> iced::Result {
    ui::WhatIf::start()
}
