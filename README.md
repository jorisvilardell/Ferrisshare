# Ferrisshare — P2P file transfer

Ferrisshare is a small Rust peer-to-peer file transfer toy used for a systems programming project.
It implements a tiny text-based protocol over TCP to send a single file from a sender (CLI) to a receiver (listener).

This README is intentionally short — it explains what the project is and how to run the listener and the CLI sender locally for development.

## What it is

- A minimal CLI sender (`cli` binary) and a listener/receiver service (`ferrisshare` binary).
- Protocol highlights: `HELLO` to announce a file, `YEET` to send block headers followed by the raw block bytes, `MISSION-ACCOMPLISHED` then `BYE-RIS` to finish.
- Storage: receiver writes to a temporary `*.ferrisshare` file then renames to the final filename.

## Quick run (local development)

Prerequisites:

- rust toolchain (stable) and cargo
- network access to localhost

Build the workspace:

```bash
cargo build --workspace
```

Run the listener (receiver) on port 9000 (default):

```bash
# In one terminal
cargo run --bin ferrisshare
```

Send a file with the CLI (sender). Example: send the repository `README.md` to localhost:9000

```bash
# In another terminal
cargo run --bin cli -- send --addr 127.0.0.1:9000 --file README.md --block-size 2048
```

**Recommended minimal block size**

We recommend using a minimal block size of 2048 bytes (as shown in the example above). Larger blocks reduce protocol overhead and typically improve throughput for local transfers. Be aware larger blocks use more memory and may be less forgiving on very unreliable networks — adjust down if you see timeouts or memory pressure.

Logs printed to both terminals show the protocol exchange (HELLO, OK, YEET blocks, OK-HOUSTEN responses, MISSION-ACCOMPLISHED, SUCCESS, BYE-RIS).

## Notes and troubleshooting

- The listener stores incoming data in `./<filename>.ferrisshare` during transfer and renames it to `./<filename>` after `MISSION-ACCOMPLISHED`.
- If you change block sizes on the sender, ensure they match the expected file split behavior.
- For debugging, run both binaries locally and watch logs.

## Development

- Tests: `cargo test`
- Formatting: `cargo fmt`
- Linting: `cargo clippy --all-targets --all-features -- -D warnings`

If you want more detailed usage or a packaged distribution, tell me which format you prefer and I'll add it.
