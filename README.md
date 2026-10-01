# Maqai

A guided rooting tool for Meta Quest, built with Tauri and Leptos.

## Development

Install the [Tauri prerequisites](https://v2.tauri.app/start/prerequisites/), then:

```sh
cargo install tauri-cli --version '^2' --locked
cargo install trunk --locked
rustup target add wasm32-unknown-unknown
cargo tauri dev
```

Build with `cargo tauri build`.

## License

GPL-3.0-or-later. Based on the [Tauri Leptos template](https://github.com/tauri-apps/create-tauri-app), licensed under MIT OR Apache-2.0. Original notices are in [licenses](licenses/).
