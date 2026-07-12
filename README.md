# Dept Tracker

Self-hosted debt dashboard for home servers (Proxmox, Umbrel, Unraid, Synology, …).
Track multiple loans, automatic regular payments, _Sonderzahlungen_ (extra payments),
interest, and payoff projections. Your financial data never leaves your machine.

## 🚀 Run it (Docker Compose)

The easiest way to add Dept Tracker to your home server — drop this service into your
`docker-compose.yml` and run `docker compose up -d`. No build required; it pulls a
prebuilt image.

```yaml
services:
  dept-tracker:
    image: ghcr.io/maxkeller321/dept-tracker:latest
    container_name: dept-tracker
    ports:
      - "8080:8080"                  # host:container — change the host port if 8080 is taken
    volumes:
      - ./dept-tracker-data:/data    # SQLite DB lives here — back this folder up
    environment:
      - RUST_LOG=info
      # Optional: bootstrap the first account on an EMPTY database.
      # Leave these unset to create your account in the UI on first visit.
      # - AUTH_USERNAME=admin
      # - AUTH_PASSWORD=change-me
    restart: unless-stopped
```

> Then browse to **`http://<server-ip>:8080`** and click **Create account** to set your
> username and password.

**Already have a compose stack?** Just paste the `dept-tracker:` block under your existing
`services:` and run `docker compose up -d dept-tracker`.

<details>
<summary><b>Prefer to build from source instead of pulling the image?</b></summary>

```bash
git clone https://github.com/maxkeller321/dept_tracker.git
cd dept_tracker
docker compose -f docker/compose.yml up -d --build
```

The backend Dockerfile uses [cargo-chef](https://github.com/LukeMathWalker/cargo-chef), so
code-only changes rebuild quickly without re-fetching crates. If the first build fails while
downloading Rust crates (slow/flaky network), just run the command again — dependency layers
are cached after the first successful cook step.
</details>

### First run & accounts

- On first visit with an empty database, the app shows **Create account** / _Konto erstellen_.
  Set a username and password there. **Sign in** is always available via the Login tab.
- One local account per installation. Docker does **not** auto-create a default password.
- To pre-seed credentials (skip manual registration on an empty DB), set `AUTH_USERNAME` /
  `AUTH_PASSWORD` in the compose `environment:` **before** the first run.
- To start over / register again: stop the container, delete `./dept-tracker-data/dept_tracker.db`
  (or the whole data folder), and start it again.

### Data & backups

Your data lives in the `/data` volume (`dept_tracker.db`). For portable backups use
**Settings → Export JSON**; **Import** does a **full replace** of all data.

> **🔒 Security:** Do not expose port 8080 directly to the internet. Put it behind a VPN
> (WireGuard/Tailscale) or a reverse proxy with TLS, and use a strong password.

## Features

- Multiple loans with an **interest rate (APR %)** each.
- Repayment as **prozentuale Tilgung %** or **Tilgung €**.
- **Regular installments are applied automatically**; only **Sonderzahlungen** (extra
  payments — immediate or scheduled) are entered manually.
- Interest paid / remaining, projected payoff dates, per-loan and household amortization.
- UI languages: **English, German, Spanish, French**.

More detail: [specs/001-debt-dashboard/quickstart.md](specs/001-debt-dashboard/quickstart.md)

## Development

```bash
# Backend + frontend dev servers (frontend proxies /api → :8080)
export AUTH_USERNAME=test AUTH_PASSWORD=test   # or use "Create account" on a fresh DB
cd backend && DATA_DIR=../data cargo run -p api
cd frontend && npm install && npm run dev
```

```bash
# Tests
cd backend && cargo test
cd frontend && npm test && npm run test:e2e
```

## Stack

Rust (Axum, sqlx, SQLite) · Svelte 5 + Vite · Docker

## License

Not set yet — pick one before publishing (MIT/Apache-2.0 are common for self-hosted apps)
and add a `LICENSE` file.
