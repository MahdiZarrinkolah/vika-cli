# Testing Approach

This document explains the testing strategies and methodologies used in `vika-cli`.

## Testing Philosophy

We use a **multi-layered testing approach** combining unit tests, integration tests, and property-based testing to ensure reliability and correctness.

## Testing Strategies

### 1. **Unit Testing** (Isolated Function Testing)

**Purpose:** Test individual functions in isolation with controlled inputs/outputs.

**Approach:**
- Tests live alongside code in `#[cfg(test)] mod tests` blocks
- Test pure functions with known inputs → expected outputs
- Test edge cases (empty strings, special characters, boundaries)

**Example from `src/generator/utils.rs`:**
```rust
#[test]
fn test_to_pascal_case_simple() {
    assert_eq!(to_pascal_case("hello"), "Hello");
    assert_eq!(to_pascal_case("world"), "World");
}

#[test]
fn test_to_pascal_case_empty() {
    assert_eq!(to_pascal_case(""), "");  // Edge case
}
```

**Characteristics:**
- ✅ Fast execution
- ✅ Tests specific behavior
- ✅ Easy to debug failures
- ✅ Can test private functions

### 2. **Property-Based Testing** (Multiple Input Variations)

**Purpose:** Test functions with various input combinations to find edge cases.

**Approach:**
- Test the same function with different input formats
- Verify consistent behavior across variations
- Discover unexpected edge cases

**Example from `src/generator/utils.rs`:**
```rust
// Same function, different input formats
test_to_pascal_case_with_underscore()  // "hello_world" → "HelloWorld"
test_to_pascal_case_with_hyphen()      // "hello-world" → "HelloWorld"
test_to_pascal_case_with_space()       // "hello world" → "HelloWorld"
test_to_pascal_case_mixed()            // "hello_world-test" → "HelloWorldTest"
```

**Benefits:**
- Finds bugs across input variations
- Ensures consistent behavior
- Documents expected behavior for different formats

### 3. **Integration Testing** (End-to-End Workflows)

**Purpose:** Test complete workflows from input to output, verifying components work together.

**Approach:**
- Test full user workflows (parse → generate → write)
- Use real-world scenarios
- Verify file system operations
- Test async operations

**Example from `tests/integration/generate_test.rs`:**
```rust
#[tokio::test]
async fn test_full_generation_workflow() {
    // 1. Setup: Create temp directory and spec file
    let temp_dir = setup_test_env();
    let spec_json = r#"{...}"#;
    fs::write(&spec_path, spec_json).unwrap();
    
    // 2. Parse: Convert spec to internal representation
    let parsed = fetch_and_parse_spec(spec_path).await.unwrap();
    
    // 3. Generate: Create TypeScript types, Zod schemas, API client
    let types = generate_typings(...).unwrap();
    let zod_schemas = generate_zod_schemas(...).unwrap();
    let api_functions = generate_api_client(...).unwrap();
    
    // 4. Write: Save files to disk
    write_schemas(...).unwrap();
    write_api_client(...).unwrap();
    
    // 5. Verify: Check files exist and are correct
    assert!(output_dir.join("users/types.ts").exists());
}
```

**Characteristics:**
- ✅ Tests real-world usage
- ✅ Catches integration bugs
- ✅ Validates file I/O
- ⚠️ Slower than unit tests
- ⚠️ More complex setup

### 4. **Fixture-Based Testing** (Test Data Builders)

**Purpose:** Create reusable test data to reduce duplication and improve maintainability.

**Approach:**
- Build test fixtures in `tests/common/fixtures.rs`
- Create factory functions for common test scenarios
- Build complex objects incrementally

**Example from `tests/common/fixtures.rs`:**
```rust
// Minimal fixture
pub fn create_minimal_openapi_spec() -> OpenAPI { ... }

// Complex fixture with dependencies
pub fn create_multi_module_spec() -> OpenAPI {
    let mut openapi = create_minimal_openapi_spec();  // Reuse base
    // Add modules, schemas, etc.
}

// Specialized fixtures
pub fn create_user_schema() -> Schema { ... }
pub fn create_enum_schema() -> Schema { ... }
pub fn create_nested_schema() -> Schema { ... }
```

**Benefits:**
- ✅ DRY (Don't Repeat Yourself)
- ✅ Consistent test data
- ✅ Easy to update when schemas change
- ✅ Clear test intent

### 5. **Error Testing** (Error Handling Validation)

**Purpose:** Ensure errors are properly created, converted, and displayed.

**Approach:**
- Test error creation with various inputs
- Verify error messages contain expected information
- Test error type conversions
- Validate error context preservation

**Example from `src/error.rs`:**
```rust
#[test]
fn test_schema_error_display() {
    let error = SchemaError::NotFound { name: "User".to_string() };
    let error_msg = error.to_string();
    assert!(error_msg.contains("User"));
    assert!(error_msg.contains("not found"));
}

#[test]
fn test_error_conversion() {
    let schema_error = SchemaError::NotFound { ... };
    let vika_error: VikaError = schema_error.into();
    // Verify conversion preserves information
}
```

**Benefits:**
- ✅ Ensures helpful error messages
- ✅ Validates error type system
- ✅ Tests error propagation

### 6. **State-Based Testing** (Testing State Changes)

**Purpose:** Verify that operations correctly modify state.

**Approach:**
- Test before/after state
- Verify state transitions
- Test state persistence

**Example from `tests/integration/common_schemas_test.rs`:**
```rust
#[test]
fn test_detect_common_schemas() {
    // Initial state: schemas in multiple modules
    let mut module_schemas = HashMap::new();
    module_schemas.insert("users", vec!["User", "CommonResponse"]);
    module_schemas.insert("products", vec!["Product", "CommonResponse"]);
    
    // Operation: Filter common schemas
    let (filtered, common_schemas) = filter_common_schemas(&module_schemas, &selected);
    
    // Verify state changes:
    // 1. Common schemas extracted
    assert!(common_schemas.contains("CommonResponse"));
    // 2. Removed from individual modules
    assert!(!filtered.get("users").contains("CommonResponse"));
    // 3. Other schemas preserved
    assert!(filtered.get("users").contains("User"));
}
```

### 7. **Boundary Testing** (Edge Cases)

**Purpose:** Test limits, empty inputs, and extreme values.

**Approach:**
- Test empty inputs
- Test single-item inputs
- Test maximum values
- Test invalid inputs

**Examples:**
```rust
test_to_pascal_case_empty()           // Empty string
test_no_common_schemas_single_module() // Single module (no commons possible)
test_get_cached_spec_miss()            // Cache miss scenario
```

### 8. **Contract Testing** (API Contract Validation)

**Purpose:** Ensure functions meet their documented contracts.

**Approach:**
- Test function signatures match expectations
- Verify return types
- Test error conditions
- Validate side effects

**Example:**
```rust
// Contract: generate_typings returns Result<Vec<TypeScriptType>>
let types = generate_typings(...).unwrap();  // Must not panic
assert!(!types.is_empty());                  // Must return items
```

## Test Organization Patterns

### **Arrange-Act-Assert (AAA) Pattern**

Most tests follow the AAA pattern:

```rust
#[test]
fn test_example() {
    // Arrange: Set up test data
    let input = "hello_world";
    
    // Act: Execute the function
    let result = to_pascal_case(input);
    
    // Assert: Verify the result
    assert_eq!(result, "HelloWorld");
}
```

### **Given-When-Then Pattern** (for integration tests)

```rust
#[tokio::test]
async fn test_full_generation_workflow() {
    // Given: A valid OpenAPI spec file
    let spec_json = r#"{...}"#;
    fs::write(&spec_path, spec_json).unwrap();
    
    // When: We run the full generation workflow
    let parsed = fetch_and_parse_spec(...).await.unwrap();
    let types = generate_typings(...).unwrap();
    write_schemas(...).unwrap();
    
    // Then: Files should be created correctly
    assert!(output_dir.join("users/types.ts").exists());
}
```

## Testing Tools & Utilities

### **Test Fixtures** (`tests/common/fixtures.rs`)
- Factory functions for creating test data
- Reusable OpenAPI spec builders
- Schema builders for different types

### **Test Helpers** (`tests/common/helpers.rs`)
- `setup_test_env()` - Create temporary directories
- `assert_file_contents()` - Verify file contents
- `create_mock_config()` - Create test configurations

### **External Libraries**
- `tempfile` - Temporary directories for file I/O tests
- `tokio-test` - Async test utilities
- `insta` - Snapshot testing (for generated code)

## Test Coverage Goals

1. **Unit Tests:** Cover all public and critical private functions
2. **Integration Tests:** Cover all major user workflows
3. **Error Tests:** Cover all error paths
4. **Edge Cases:** Test boundaries and empty inputs
5. **Property Tests:** Test functions with various input formats

## Running Tests

```bash
# All tests
cargo test

# Unit tests only
cargo test --lib

# Integration tests only  
cargo test --test '*'

# Specific test
cargo test test_to_pascal_case

# With output
cargo test -- --nocapture
```

## Best Practices

1. ✅ **One assertion per test** (when possible) - makes failures clear
2. ✅ **Descriptive test names** - `test_to_pascal_case_with_underscore` not `test1`
3. ✅ **Test edge cases** - empty strings, null values, boundaries
4. ✅ **Use fixtures** - Don't duplicate test data setup
5. ✅ **Test errors** - Verify error handling works
6. ✅ **Fast tests** - Unit tests should be fast (< 1ms)
7. ✅ **Isolated tests** - Tests shouldn't depend on each other
8. ✅ **Clear assertions** - Use `assert_eq!` with expected/actual

