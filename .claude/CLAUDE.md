# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Rust gRPC implementation of the Senzing SDK traits (`sz-sdk-rust-grpc`). This crate implements the traits defined in [`sz-sdk`](https://github.com/docktermj/sz-sdk-rust) by communicating with a remote Senzing gRPC server (e.g., `senzing/serve-grpc`). Licensed under Apache 2.0.

## Build Commands

- `cargo build` — compile the project
- `cargo test` — run all tests
- `cargo test <test_name>` — run a single test
- `cargo clippy -- -D warnings` — lint (treat warnings as errors)
- `cargo fmt` — format code
- `make setup` — start a Senzing gRPC server via Docker on port 8261 (required for integration tests)
- `make clean` — stop the Docker container

## Architecture

**Trait-based design:** The `sz-sdk` crate (git dependency) defines traits (`SzEngine`, `SzConfig`, `SzConfigManager`, `SzDiagnostic`, `SzProduct`, `SzAbstractFactory`) and error types (`SzError`). This crate provides gRPC implementations of each trait.

**Key pattern — sync wrapper over async gRPC:** Trait methods are synchronous. Each struct holds a cloned `tonic` gRPC client and calls `runtime().block_on(async { ... })` to invoke async gRPC methods. A shared global tokio runtime is created via `OnceLock` in `src/runtime.rs`.

**Proto files** in `proto/` are compiled by `build.rs` using `tonic-build`. Generated code is included via `tonic::include_proto!()` in `src/lib.rs` as `pb_szconfig`, `pb_szconfigmanager`, `pb_szdiagnostic`, `pb_szengine`, `pb_szproduct`.

**Factory pattern:** `SzAbstractFactoryGrpc` holds a `tonic::transport::Channel` (cheaply cloneable) and creates service clients from it. Mirrors the Go SDK pattern in `senzing-garage/sz-sdk-go-grpc`.

**SzConfig statefulness:** The proto API is stateless (config_definition is a parameter), but the Rust `SzConfig` trait is stateful. `SzConfigGrpc` stores the config_definition and passes it on each RPC, updating it when mutations occur.

## File Layout

- `src/lib.rs` — module declarations, proto includes, re-exports
- `src/szabstractfactory.rs` — `SzAbstractFactoryGrpc` (entry point)
- `src/szengine.rs` — `SzEngineGrpc` (~25 methods)
- `src/szconfig.rs` — `SzConfigGrpc` (stateful config wrapper)
- `src/szconfigmanager.rs` — `SzConfigManagerGrpc` (creates SzConfigGrpc instances)
- `src/szdiagnostic.rs` — `SzDiagnosticGrpc`
- `src/szproduct.rs` — `SzProductGrpc`
- `src/runtime.rs` — shared tokio runtime via OnceLock
- `proto/*.proto` — protobuf service definitions (from `senzing-garage/sz-sdk-proto`)
- `build.rs` — proto compilation with tonic-build
