# Datanium

Datanium is a modular, self-hosted database management system written in Rust.

## Goals

- Modular database architecture
- Pluggable storage engines
- Structured data and relational models
- Query execution and optimization
- Network-accessible database server
- Extensible computation engine

## Workspace

| Crate | Description |
|---|---|
| datanium-core | Shared types and interfaces |
| datanium-storage | Persistent storage (planned) |
| datanium-query | Query processing |
| datanium-server | Database server |
| datanium-client | Rust client SDK |

## Development

Check the workspace:

    cargo check --workspace

Run tests:

    cargo test --workspace

Run the server:

    cargo run -p datanium-server

## Status

Early development. Not ready for production use.

See [Contributing to Datanium](CONTRIBUTING.md) for the PR workflow, local
checks and data integrity rules.
