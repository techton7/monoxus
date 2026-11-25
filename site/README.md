# monoxus-site

Official documentation and interactive showcase website for [Monoxus](https://github.com/techton7/monoxus) headless Dioxus primitives.

## Local Development

### Prerequisites

- Rust (`wasm32-unknown-unknown` target)
- `dioxus-cli` (`0.8.0-alpha.3`)
- Node.js (for running edge worker tests)

### Repository Layout & Local `monoxus` Dependency

`monoxus-site` lives in `monoxus/site/` as an excluded independent Cargo package and depends directly on the parent crate via `monoxus = { path = ".." }` in `site/Cargo.toml`. No `.cargo/config.toml` patch override is needed to dogfood local changes.

Run all site commands from the `site/` directory.

### Run Dev Server

```sh
dx serve --platform web
```

### Run Tests

```sh
cargo test
node --test worker/redirects.test.ts
```

## Cloudflare Workers Deployment (`monoxus.com`)

The site builds as a static Dioxus WebAssembly SPA (`./dist`) fronted by a lightweight Cloudflare Worker (`worker/index.ts`) that handles `/docs` redirects and `Accept: text/markdown` content negotiation on `/docs/*`.

### CI/CD Secrets

Configure the following repository secrets in GitHub Actions for `.github/workflows/deploy-site.yml`:

- `CLOUDFLARE_API_TOKEN`
- `CLOUDFLARE_ACCOUNT_ID`

### Manual Build & Verify

```sh
dx build --release --platform web
rm -rf dist && mkdir -p dist
cp -R target/dx/monoxus-site/release/web/public/. dist/
cp -R public/. dist/
./scripts/check_export.sh
npx wrangler deploy
```

### Custom Domain Cutover (`monoxus.com`)

1. Register `monoxus.com` and onboard it to Cloudflare.
2. Uncomment the `routes` block for `monoxus.com` and `www.monoxus.com` in `wrangler.jsonc`.
3. Set `"workers_dev": false` in `wrangler.jsonc` once the custom domain routes are live.
4. Deploy via `npx wrangler deploy` or by pushing to `main`.
