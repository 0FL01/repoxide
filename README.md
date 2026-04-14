## Repoxide: Oxidizing Codebases Since Yesterday

Welcome to my lovingly hand-oxidized Rust monorepo, Repoxide—where dreams of simplicity oxidize into convoluted magnificence. Imagine GitHub, but after leaving it out in the rain overnight—yep, that's Repoxide.

## Folder Architecture (or: How to Confuse Future Me Efficiently)

* `crates/repoxide` — The beating, rusted heart of a CLI/library that occasionally works (terms & conditions apply).
* `apps/web/server` — A rust-powered web API, still alive for now. (Web-scale guaranteed by the law of wishful thinking.)
* `apps/web/client` — A former VitePress/Vue frontend. Retired, but haunting the repo because deleting code is emotional.

## Quick-ish Start (a.k.a. Please Don't File Issues Yet)

```bash
# For rusty help, type from the root:
cargo run -- --help  # If stuck, yell at terminal louder

# If you're feeling confident and optimistic (I'm not):
cargo build --release  # "Optimized" means bugs run faster

# Spin up my Docker-powered tech circus:
docker compose up --build  # "But it builds on my machine..."
```

## Web Application (Actively Rusty, Officially Frontend-Free™)

* The current web-stack™️ (buzzword-compliant) lives bravely at `apps/web/server`.
* The original frontend (`apps/web/client`) is now a relic, useful only as a warning to future generations.
* Runs locally on `127.0.0.1:93` because 92 was too mainstream.
