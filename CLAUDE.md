# spearmint

CLI tool to sync Roblox developer products and game passes from a local TOML config to the Roblox Open Cloud API.

## Build & Run

```bash
cargo build          # debug build
cargo build --release # release build (LTO + stripped)
cargo run -- <command>
```

Run from the project directory that contains `spearmint.toml` and `.env`:
```bash
spearmint sync
spearmint generate
spearmint list
spearmint init
```

## Architecture

```
src/
  main.rs       - Entry point, CLI routing
  cli.rs        - Clap subcommands: init, sync, generate, list
  config.rs     - spearmint.toml loading/saving, Product/Config types
  sync.rs       - Core sync logic, lock file (spearmint.lock.toml) management
  codegen.rs    - Lua + TypeScript code generation from mapping
  api/
    mod.rs          - HTTP client, API key auth
    dev_products.rs - Developer products endpoints (multipart/form-data)
    gamepasses.rs   - Game passes endpoints (multipart/form-data)
```

## Key Files (in consuming project)

- `spearmint.toml` - Product definitions (universe_id, output config, product entries)
- `spearmint.lock.toml` - Maps local keys to Roblox IDs + cached metadata (name, price, description, image SHA-256 hash) for change detection
- `.env` - Must contain `ROBLOX_PRODUCTS_API_KEY`

## Roblox API Details

All endpoints use `multipart/form-data` (not JSON) for both create and update:
- Create returns 200 with JSON response body
- Update (PATCH) returns 204 No Content
- Auth via `x-api-key` header
- Rate limits: 3 req/s for dev products, 5 req/s for game passes

## Dependencies

Rust edition 2021. Key crates: clap 4 (CLI), reqwest 0.12 (HTTP + multipart), serde/toml (config), tokio (async), anyhow (errors), dotenvy (.env), sha2/hex (image content hashing).
