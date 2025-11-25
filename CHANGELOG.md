# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [1.0.0] - 2025-11-22

### Added

- Initial release of vika-cli
- Generate TypeScript interfaces from OpenAPI schemas
- Generate Zod validation schemas
- Generate Fetch-based HTTP client functions
- Module-based code generation (grouped by Swagger tags)
- Interactive module selection
- Configurable output directories
- Progress indicators and verbose mode
- Spec caching for faster regeneration
- Backup system for generated files
- Conflict detection for user-modified files
- Inspect command for analyzing OpenAPI specs
- Support for all HTTP methods (GET, POST, PUT, DELETE, PATCH, HEAD, OPTIONS)
- Formatted table output for module summaries
- Support for complex schemas (nested objects, arrays, unions, allOf, oneOf)
- Enum type generation
- Optional vs required field handling
- Nullable type support
- Circular dependency detection and handling
- Deep nesting support (unlimited depth)
- Environment variable substitution in configuration
- Multiple installation methods (install script, cargo)
- Comprehensive documentation
- CI/CD workflows for testing and releases
- Multi-platform binary releases (Linux, macOS Intel/ARM, Windows)

### Changed

- N/A (initial release)

### Deprecated

- N/A (initial release)

### Removed

- N/A (initial release)

### Fixed

- N/A (initial release)

### Security

- N/A (initial release)

## [1.0.2] - 2025-11-22

### Fixed

- Fixed release workflow to use correct binary name (`vika-cli` instead of platform-specific names)
- Fixed README.md with correct directory name for cloning (`cd vika-cli` instead of `cd vika`)

### Changed

- Removed redundant version-bump workflow (using cargo-release instead)
- Updated GitHub Actions workflows to explicitly set permissions for releases

## [1.0.1] - 2025-11-22

### Fixed

- Fixed GitHub Actions release workflow permissions
- Fixed release workflow binary path issue
- Initial multi-platform release workflow setup


## [1.0.3] - 2025-11-22

### Fixed

- Fixed release workflow to include install scripts (install.sh and install.ps1) in release assets

## [1.0.4] - 2025-11-24

### Fixed

- Fixed HTTP client body property to use `null` instead of `undefined` for TypeScript `exactOptionalPropertyTypes` compatibility
- Fixed GET request with body parameter type mismatch by adding proper RequestInit detection
- Fixed enum schema name resolution in Zod schemas when referenced via `$ref` (e.g., `CurrencySchema` -> `CurrencyEnumSchema`, `ProviderKeyEnumSchema`)
- Fixed enum schema generation for top-level enum schemas referenced in object properties and arrays
- Added HEAD and OPTIONS HTTP method support to HTTP client template
- Fixed CI test execution to run integration tests sequentially (`--test-threads=1`) to avoid directory conflicts in formatter tests

### Changed

- Enhanced HTTP client template to support GET requests with body parameters
- Improved enum schema reference resolution to check enum registry before falling back to schema name
- Enum schemas are now used directly (without `z.lazy()`) since they don't have circular dependencies
- Updated CI workflow to run integration tests sequentially for better reliability

### Added

- Comprehensive test suite with 141 passing tests across 15 test files
- Integration tests for CLI entry point (`tests/integration_main_test.rs`) - 16 tests covering all commands and flags
- Unit tests for progress reporter (`tests/progress_test.rs`) - 12 tests
- Unit tests for cache manager (`tests/cache_test.rs`) - 6 tests
- Unit tests for formatter (`tests/formatter_test.rs`) - 9 tests
- Unit tests for module selector (`tests/module_selector_test.rs`) - 3 tests
- Unit tests for swagger parser (`tests/swagger_parser_test.rs`) - 11 tests
- Unit tests for schema resolver (`tests/schema_resolver_test.rs`) - 12 tests covering dependency resolution, circular detection, and schema classification
- Enhanced config module tests with edge cases and validation scenarios
- Test infrastructure improvements: added `assert_cmd` and `predicates` crates for CLI testing


## [1.1.0] - 2025-11-24

### Added

- JSDoc comments generation for API client functions
  - Operation descriptions from OpenAPI `description` and `summary` fields
  - Parameter descriptions for path, query, and request body parameters
  - Request body descriptions
  - Properly formatted JSDoc blocks with `@param` tags

### Fixed

- Fixed function body extraction to preserve JSDoc comments that appear before `export const` declarations
- Fixed TypeScript code formatter to preserve indentation in generated files
- Fixed template rendering to correctly handle empty string descriptions

## [Unreleased]

### Changed

- **BREAKING**: Refactored configuration model to support detailed per-spec configuration
  - `SpecEntry` now requires `schemas`, `apis`, and `modules` fields (previously optional)
  - Removed top-level `spec_path` field from `Config` (use `specs` array instead)
  - `specs` field changed from `Option<Vec<SpecEntry>>` to `Vec<SpecEntry>` (always present, empty array if no specs)
  - Each spec entry now has its own `SchemasConfig`, `ApisConfig`, and `ModulesConfig` for fine-grained control
  - Improved error messages for configuration validation
  - All configuration fields now have `Default` implementations for easier initialization

### Fixed

- Fixed all Clippy warnings across the codebase
  - Removed unused imports and variables
  - Fixed unnecessary borrows for generic arguments (`&format!()` -> `format!()`)
  - Fixed unnecessary lazy evaluations (`ok_or_else(|| ...)` -> `ok_or(...)`)
  - Fixed useless comparisons (`cycles.len() >= 0` -> proper assertion)
  - Suppressed deprecation warnings for `assert_cmd::Command::cargo_bin` (to be migrated in future)
- Fixed test compilation errors after configuration model refactoring
- Updated all integration tests to use new configuration API
- Fixed snapshot tests to match new import ordering in generated code
- Fixed writer tests to correctly handle spec-specific output directories
- Fixed directory restoration in snapshot tests to prevent path resolution issues

### Refactored

- Simplified prompt formatting in command files (`add.rs`, `init.rs`)
- Removed unused constants (`SPEC_CACHE_FILE`, `SPEC_META_FILE`) and functions (`collect_ts_files`, `is_multi_spec_mode`)
- Improved code organization and readability across multiple files
- Updated test helpers to use new configuration structure with helper functions
- Enhanced test coverage for multi-spec scenarios
- Standardized test patterns for creating `SpecEntry` instances with default configs

### Planned

- Template customization system
- Plugin architecture
- Additional output formats
- Performance optimizations
- More OpenAPI features support

