# Noughts and Crosses

Multiplayer browser-based tech demo built on:

- **Rust+WASM**: Demonstrating shared client-server code.
- **Bevy**: Data-driven game engine
- **Cloudflare Workers**: Serverless runtime
- **Cloudflare Durable Storage**: Realtime strongly consistent scalable storage via WebSockets

## Running

This project is not deployed to CloudFlare. It can be run locally using Wrangler and Trunk:

- Install Rust: https://rustup.rs/
- Install Trunk: https://trunkrs.dev/
- Install Wrangler: https://developers.cloudflare.com/workers/wrangler/install-and-update/

Start wrangler in a terminal:

    wrangler dev

In another terminal start Trunk:

    trunk serve

Open your web browser:

    open http://localhost:8080/
