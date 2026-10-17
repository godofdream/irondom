# Irondom

Minimalistic but featureful Home Assistant alternative. Rust, API-first, responsive web UI, low memory (<25MB RSS).

## Features
- HTTP API (axum) for devices, entities, automations, scenes, integrations
- SQLite storage (rusqlite)
- Responsive web interface (served at `/`)
- Automation engine with time-of-day triggers
- Home Assistant YAML automation import (`POST /api/automations/import`)
- Integrations: ESPHome, Tesla Fleet, Tesla Wall Connector, Ziggurat (Zigbee), Tuya

## Build
Requires Rust toolchain, mold, sccache.

```sh
cargo build --release
```

Binary: `target/release/irondom`

## Run
```sh
./irondom  # listens on 0.0.0.0:8080
```
Config via `irondom.toml`:
```toml
data_dir = "./data"
http_port = 8080
log_level = "info"
```

## Memory
Release build ~2.5MB binary, ~5MB RSS.

## License
MIT
