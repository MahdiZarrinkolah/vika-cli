# Testing Guide

This document covers the practical aspects of running and organizing tests in `vika-cli`.

> **For testing philosophy and strategies**, see [testing-approach.md](testing-approach.md).

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

**Location:** `tests/*.rs` (each file is a separate test binary)

**Run integration tests:**
```bash
# All integration tests
cargo test --tests

# Specific integration test file
cargo test --test generate_test
cargo test --test common_schemas_test
cargo test --test module_filtering_test
cargo test --test conflict_detection_test
```

**Test files:**
- `tests/generate_test.rs` - Full generation workflow (parse → generate → write)
- `tests/common_schemas_test.rs` - Common schema detection across modules
- `tests/module_filtering_test.rs` - Module filtering logic
- `tests/conflict_detection_test.rs` - File conflict detection and handling

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

#[tokio::test]
async fn my_test() {
    let spec = create_minimal_openapi_spec();  // From fixtures
    let temp_dir = setup_test_env();           // From helpers
}
```

## Running Tests

### Run all tests (unit + integration)
```bash
cargo test
```

### Run only unit tests
```bash
cargo test --lib
```

### Run only integration tests
```bash
# All integration tests
cargo test --tests

# Specific integration test file
cargo test --test generate_test
cargo test --test common_schemas_test
cargo test --test module_filtering_test
cargo test --test conflict_detection_test
```

### Run a specific test by name
```bash
# Searches across all tests (unit + integration)
cargo test test_to_pascal_case

# By module (unit tests only)
cargo test config::model::tests
```

### Run tests with output
```bash
cargo test -- --nocapture
```

### Run tests sequentially (for tests that change global state)
```bash
# Sequential execution
cargo test -- --test-threads=1

# Parallel execution (default)
cargo test
```

## Integration Test Examples

### Example: Run full generation workflow test
```bash
cargo test --test generate_test

# With verbose output
cargo test --test generate_test -- --nocapture
```

### Example: Run all integration tests
```bash
# Run all integration tests
cargo test --tests

# Run all integration tests sequentially
cargo test --tests -- --test-threads=1
```

### Example: Run specific integration test
```bash
# Test common schema detection
cargo test --test common_schemas_test

# Test module filtering
cargo test --test module_filtering_test

# Test conflict detection
cargo test --test conflict_detection_test
```

## Test Organization Benefits

1. **Unit tests** - Test individual functions in isolation, fast execution
2. **Integration tests** - Test full workflows end-to-end, verify file I/O
3. **Shared utilities** - Avoid code duplication, consistent test data
4. **Standard structure** - Follows Rust conventions

## Notes

- Integration tests in `tests/` are compiled as separate binaries
- Each `.rs` file in `tests/` becomes its own test binary
- Subdirectories in `tests/` are NOT compiled as test binaries (only files directly in `tests/`)
- Use `tests/common/` for shared utilities that other tests import with `mod common;`
- Cache tests use a mutex to prevent parallel execution issues (they change the working directory)
