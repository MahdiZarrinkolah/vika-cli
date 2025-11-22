# Testing Guide

## Test Structure

This project follows Rust's standard test organization:

### Unit Tests (in `src/` files)
Unit tests live alongside the code they test, inside `#[cfg(test)] mod tests` blocks.

**Location:** `src/**/*.rs`

**Run unit tests:**
```bash
cargo test --lib
```

**Examples:**
- `src/generator/utils.rs` - Tests for `to_pascal_case`, `to_camel_case`
- `src/error.rs` - Tests for error types and conversions
- `src/cache.rs` - Tests for caching functionality
- `src/config/model.rs` - Tests for config serialization/deserialization
- `src/config/loader.rs` - Tests for config loading/saving
- `src/config/validator.rs` - Tests for config validation

### Integration Tests (in `tests/` directory)
Integration tests test the public API and how components work together.

**Location:** `tests/integration/*.rs`

**Run integration tests:**
```bash
# All integration tests
cargo test --test '*'

# Specific integration test
cargo test --test integration::generate_test
```

**Test files:**
- `tests/integration/generate_test.rs` - Full generation workflow
- `tests/integration/common_schemas_test.rs` - Common schema detection
- `tests/integration/module_filtering_test.rs` - Module filtering logic
- `tests/integration/conflict_detection_test.rs` - File conflict detection

### Shared Test Utilities (in `tests/common/`)
Shared utilities for integration tests.

**Location:** `tests/common/`

**Structure:**
- `tests/common/mod.rs` - Module exports
- `tests/common/fixtures.rs` - Test data fixtures (OpenAPI specs, schemas)
- `tests/common/helpers.rs` - Helper functions (temp dirs, assertions)

**Usage in integration tests:**
```rust
mod common;
use common::*;  // Imports fixtures and helpers

#[test]
fn my_test() {
    let spec = create_minimal_openapi_spec();  // From fixtures
    let temp_dir = setup_test_env();           // From helpers
}
```

## Running Tests

### Run all tests
```bash
cargo test
```

### Run only unit tests
```bash
cargo test --lib
```

### Run only integration tests
```bash
cargo test --test '*'
```

### Run a specific test
```bash
# By name
cargo test test_to_pascal_case

# By module
cargo test config::model::tests

# Specific integration test file
cargo test --test integration::generate_test
```

### Run tests with output
```bash
cargo test -- --nocapture
```

### Run tests in parallel (default) or sequentially
```bash
# Parallel (default)
cargo test

# Sequential
cargo test -- --test-threads=1
```

## Test Organization Benefits

1. **Unit tests** test individual functions in isolation
2. **Integration tests** test the full workflow end-to-end
3. **Shared utilities** avoid code duplication across tests
4. **Standard structure** follows Rust conventions, making it familiar to Rust developers

