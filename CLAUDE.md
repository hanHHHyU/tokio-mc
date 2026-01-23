# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

`tokio-mc` is a pure Rust library for Mitsubishi Communication (MC) protocol, built on tokio for async communication with Mitsubishi and Keyence PLCs using the 3E frame protocol.

## Build and Development Commands

```bash
# Check compilation
cargo check

# Run tests
cargo test

# Build the library
cargo build

# Build with specific features
cargo build --features "3e-async"  # For async usage
cargo build --features "3e-sync"   # For sync usage

# Build examples
cargo build --examples --features "3e-async"
cargo build --examples --features "3e-sync"
cargo build --examples --features "tcp,server"  # For test server

# Run examples
cargo run --example 3e-client --features "3e-async"
cargo run --example 3e-client-sync --features "3e-sync"
cargo run --example mitsubishi-mc-test-server --features "tcp,server"
```

## Architecture Overview

### Core Module Structure

- **`src/client/`**: Client implementations for MC protocol communication
  - `mod.rs`: Core async traits (`Client`, `Reader`, `Writer`) and `Context` wrapper that handles PLC model-specific address conversions
  - `tcp.rs`: Async TCP client implementation
  - `sync/`: Synchronous client implementations mirroring async APIs

- **`src/frame/`**: Protocol frame handling and data structures
  - `mod.rs`: Request/Response enums, FunctionCode definitions
  - `types.rs`: Core types (Address, Quantity, Model enum for Mitsubishi/Keyence)
  - `kv/`: Keyence-specific address conversion logic
  - `map.rs`, `regex.rs`: Address parsing and instruction code mapping

- **`src/codec/`**: Encoding/decoding logic for MC protocol
  - `mod.rs`: `ClientEncoder` and decoders for protocol messages
  - `tcp.rs`: TCP-specific codec implementations
  - Handles conversion between Rust types and MC protocol byte format

- **`src/server/`**: Server implementation (feature-gated)
  - Service trait and TCP server for testing/simulation

### Key Design Patterns

1. **Address Abstraction**: The `Context` wrapper automatically converts between Keyence and Mitsubishi address formats based on the configured PLC model.

2. **Type Conversion**: All typed read/write operations (u16, i32, f32, etc.) are built on top of base u8 operations with little-endian byte conversion.

3. **Feature Gates**: 
   - `3e-async`: Enables async client functionality
   - `3e-sync`: Enables sync client functionality  
   - `server`: Enables test server implementation

4. **Protocol Implementation**: Uses 3E frame format with command/subcommand codes for different operations (ReadU8s, WriteU8s, ReadBits, WriteBits).

## Testing

The codebase includes comprehensive unit tests for:
- Address conversion and parsing
- Protocol encoding/decoding
- Bit manipulation operations
- Frame construction

Run `cargo test` to execute all tests.