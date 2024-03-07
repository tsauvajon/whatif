# What if you converted your entire net worth into BTC? 

This app, built with [Rust](https://www.rust-lang.org/) and
[iced](https://iced.rs/).

It uses WebAssembly to target the web, or can be built natively for desktops.

To run, just run `cargo run` - the only dependency is Rust
([install here](https://rustup.rs/)). I'm planning on hosting it at some point.

Since I couldn't find a free-tier API for historical data, I just downloaded a
CSV.
There will be gaps in data between 06 March 2024 and today. In a future version
I might store things in a database, but right now it's not a priority.


## Iced learning resources

- https://nikolish.in/gs-with-iced-2
- https://github.com/zupzup/rust-frontend-example-iced/blob/main/src/main.rs#L49
