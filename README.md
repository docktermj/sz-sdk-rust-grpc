# sz-sdk-rust-grpc

Rust gRPC implementation of the [Senzing SDK](https://senzing.com/) traits.

This crate provides gRPC-based implementations of the traits defined in
[`sz-sdk`](https://github.com/docktermj/sz-sdk-rust) by communicating with a
remote Senzing gRPC server (e.g., [`senzing/serve-grpc`](https://hub.docker.com/r/senzing/serve-grpc)).

## Prerequisites

A running Senzing gRPC server. The quickest way to start one locally:

```sh
docker run --rm -d -p 8261:8261 senzing/serve-grpc
```

## Usage

Add to your `Cargo.toml`:

```toml
[dependencies]
sz-sdk-rust-grpc = { git = "https://github.com/docktermj/sz-sdk-rust-grpc.git" }
sz-sdk = { git = "https://github.com/docktermj/sz-sdk-rust.git" }
```

### Quick start

```rust,no_run
use sz_sdk::{SzAbstractFactory, SzEngine, SzProduct};
use sz_sdk_rust_grpc::SzAbstractFactoryGrpc;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let factory = SzAbstractFactoryGrpc::new_from_url("http://localhost:8261")?;
    factory.check_health()?;

    let product = factory.create_product()?;
    println!("{}", product.get_version()?);

    let mut engine = factory.create_engine()?;
    let record = r#"{"NAME_FULL": "Jane Smith", "ADDR_FULL": "100 Main St"}"#;
    engine.add_record("CUSTOMERS", "1001", record, 0)?;

    Ok(())
}
```

### Builder pattern

```rust,no_run
use std::time::Duration;
use sz_sdk_rust_grpc::SzAbstractFactoryGrpc;

let factory = SzAbstractFactoryGrpc::builder()
    .url("http://localhost:8261")
    .connect_timeout(Duration::from_secs(5))
    .rpc_timeout(Duration::from_secs(60))
    .build()?;
# Ok::<(), sz_sdk::SzError>(())
```

## Features

| Feature   | Default | Description |
|-----------|---------|-------------|
| `tracing` | off     | Emits [`tracing`](https://docs.rs/tracing) spans for every gRPC call |

Enable via Cargo:

```toml
sz-sdk-rust-grpc = { git = "...", features = ["tracing"] }
```

## Architecture

- All trait methods are **synchronous**. Internally they use `block_on` on a
  shared tokio runtime to execute async gRPC calls.
- All types are `Clone + Send + Sync` — safe to share across threads.
- A single `tonic::transport::Channel` is shared across all service clients,
  multiplexing RPCs over a pool of HTTP/2 connections.

## License

[Apache 2.0](LICENSE)
