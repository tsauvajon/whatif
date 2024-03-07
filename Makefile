build:
	cargo build --target wasm32-unknown-unknown
	wasm-bindgen ./target/wasm32-unknown-unknown/debug/whatif.wasm --out-dir docs --web
	# Install with `cargo install miniserve`
	miniserve docs --index index.html

serve:
	trunk serve
