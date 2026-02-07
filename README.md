# spearmint

Sync Roblox developer products and game passes from a TOML config to the Open Cloud API.

## Install

```bash
cargo install spearmint                # Cargo
mise use github:UrbanIndigo/spearmint  # mise
aftman add UrbanIndigo/spearmint       # Aftman
```

## Setup

1. Create an API key at [create.roblox.com/dashboard/credentials](https://create.roblox.com/dashboard/credentials) with **Developer Product** and **Game Pass** write permissions.

2. Add your key to `.env`:
   ```
   ROBLOX_PRODUCTS_API_KEY=your_api_key_here
   ```

3. Run `spearmint init` and edit the generated `spearmint.toml`.

## Config

```toml
universe_id = 123456789

[output]
path = "src/shared/modules/Products.luau"
typescript = true

[products.coins_100]
type = "dev_product"
name = "100 Coins"
price = 99
description = "Get 100 coins"
image = "assets/products/coins.png"

[products.vip]
type = "gamepass"
name = "VIP"
price = 499
```

## Commands

| Command | Description |
|---------|-------------|
| `spearmint sync` | Sync products to Roblox and generate output files |
| `spearmint generate` | Generate Lua/TypeScript files without API calls |
| `spearmint list` | List products and sync status |
| `spearmint init` | Create a default config template |

## License

MIT
