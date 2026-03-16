# Code0-Flow - Flow Management Library

[![Crate](https://img.shields.io/crates/v/code0-flow.svg)](https://crates.io/crates/code0-flow)
[![Documentation](https://docs.rs/code0-flow/badge.svg)](https://docs.rs/code0-flow)

Internal Rust library for CodeZero execution services (Aquila, Draco, Taurus). It manages flow definitions, config helpers, and health checks.

## Features

- `flow_definition` read flow definitions from a filesystem layout (per-feature folders with `data_type/`, `flow_type/`, `runtime_definition/` JSONs) with optional feature/version filtering
- `flow_service` push definitions to Aquila via gRPC (depends on `flow_definition`)
- `flow_config` env helpers and `.env` loading
- `flow_health` tonic health service with NATS readiness

