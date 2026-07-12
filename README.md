# Dept Tracker

Self-hosted debt dashboard for home servers (Proxmox, Umbrel, etc.). Track multiple loans, automatic regular payments, Sonderzahlungen (extra payments), interest, and payoff projections.

## Quick start

```bash
docker compose -f docker/compose.yml up -d --build
```

Open http://localhost:8080 — use **Create account** / **Konto erstellen** on the auth screen (or the Register tab) to set up username and password on first visit. One local account per installation; **Sign in** is always available via the Login tab.

If the image build fails while downloading Rust crates (slow or flaky network), run `docker compose -f docker/compose.yml build` again — dependency layers are cached after the first successful cook step, so retries are usually faster. The backend Dockerfile uses [cargo-chef](https://github.com/LukeMathWalker/cargo-chef) so code-only changes rebuild quickly without re-fetching crates.

Optional: pre-set credentials via `.env` (skips manual registration if the database has no account yet):

```bash
cp .env.example .env   # set AUTH_USERNAME and AUTH_PASSWORD
docker compose -f docker/compose.yml up -d --build
```

Without `.env` credentials, Docker does **not** auto-create a default password — fresh `./data` shows the registration flow.

To register again: delete `./data/dept_tracker.db` (or the whole `./data` folder) and restart, **or** set `AUTH_USERNAME` / `AUTH_PASSWORD` in `.env` before first run with an empty database.

Data lives in `./data/dept_tracker.db` (Docker volume). For portable backups use **Settings → Export JSON**; import does a **full replace** of all data.

Each loan requires an **interest rate (APR %)**. Repayment is either **prozentuale Tilgung %** or **Tilgung €**. **Regular installments are applied automatically**; only **Sonderzahlungen** are entered manually. UI languages: **English, German, Spanish, French**.

> **Security:** Do not expose port 8080 to the internet without a VPN or reverse proxy. Use a strong password on the setup screen.

More detail: [quickstart.md](specs/001-debt-dashboard/quickstart.md)

## Development

```bash
# Fresh DB: open app and use “Create account”, or:
export AUTH_USERNAME=test
export AUTH_PASSWORD=test
cd backend && DATA_DIR=../data cargo run -p api
cd frontend && npm install && npm run dev   # proxies /api → :8080
```

```bash
cd backend && cargo test
cd frontend && npm test && npm run test:e2e
```

## Stack

Rust (Axum, sqlx, SQLite) · Svelte 5 + Vite · Docker
