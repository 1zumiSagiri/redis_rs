# redis_rs
Implementation of [Redis](https://redis.io/) in Rust.

## Build
```bash
cargo run --bin server
cargo run --bin client
cargo run --example hello_world_redis
```

## Features
- Crate [tokio](https://docs.rs/tokio/latest/tokio/) for async I/O
- Crate [tracing](https://docs.rs/tracing/latest/tracing/) for logging and journaling