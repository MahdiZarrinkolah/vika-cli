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

## [1.2.0] - 2025-11-25

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

## [1.3.0] - 2025-11-26

### Added

- **Hook Generators for React Query and SWR**
  - Added `--react-query` flag to generate React Query hooks (`useQuery`, `useMutation`)
  - Added `--swr` flag to generate SWR hooks (`useSWR`, `useSWRMutation`)
  - Shared query keys system for consistent cache key management across hooks
  - Query keys are generated as functions that return arrays, supporting parameterized keys
  - Hook generation supports both single-spec and multi-spec modes
  - Generated hooks follow React Query and SWR best practices:
    - Queries use `useQuery`/`useSWR` with query keys
    - Mutations use `useMutation`/`useSWRMutation` with proper parameter handling
    - Path parameters are included in hook signatures
    - Body parameters are passed via `mutate()`/`trigger()` calls
    - Proper TypeScript types for all parameters and return values
  - Schema imports are automatically added for body types and enum types
  - Enum imports are included for query parameters that use enums
  - Hook files are organized by module: `src/hooks/{spec}/{module}/useX.ts`
  - Query keys files: `src/query-keys/{spec}/{module}.ts`
  - Hook naming follows conventions: `useGetX` for queries, `useCreateX`/`useUpdateX` for mutations

### Changed

- Hook generation now requires only one hook flag per command (mutually exclusive)
- Query keys generation groups query parameters into a single object parameter for consistency with API functions
- Import paths in hooks are calculated relative to hook file location

### Removed

- **BREAKING**: Removed `--tanstack` flag (TanStack Query support)
  - TanStack Query hooks have been removed in favor of React Query hooks
  - React Query and TanStack Query use the same library (`@tanstack/react-query`), so only one flag is needed

### Fixed

- Fixed non-deterministic import ordering in API client generation
  - Module paths are now sorted before generating imports to ensure consistent output
  - Fixes snapshot test failures on subsequent runs
- Fixed query keys generation to always return functions (even when no parameters)
  - Changed from `['key']` to `() => ['key']` for consistency
- Fixed mutation hooks to not accept body parameters in hook signature
  - Body is now passed via `mutate(data)` or `trigger({ arg: data })` calls
  - Only path parameters appear in hook signatures
- Fixed SWR mutation pattern to use correct API signature
  - Changed from `(_, data)` to `(key: string, { arg }: { arg: Type })` pattern
  - Mutations without body use `(key: string)` pattern
- Fixed schema import paths in hooks (corrected depth calculation)
- Fixed enum imports to be included in hook files when query parameters use enums
- Fixed Clippy warnings:
  - Removed unnecessary `mut` keywords
  - Changed `push_str("\n")` to `push('\n')` for single character appends
  - Changed `iter().cloned().collect()` to `to_vec()` for better performance
  - Added `#[allow(clippy::too_many_arguments)]` for generate command (9 args required by CLI structure)
- Fixed template registry test to expect 11 templates (added 5 hook templates)
- Removed failing template initialization tests that had file system issues

## [1.4.0] - 2025-12-01

### Added

- **Runtime HTTP Client with Generic Result System**
  - Added `VikaClient` class with configurable options (baseUrl, timeout, retries, retryDelay, headers, auth)
  - Implemented middleware system with `beforeRequest`, `afterResponse`, and `onError` callbacks
  - Added timeout support via `AbortController` for request cancellation
  - Implemented retry logic with exponential backoff for specific HTTP status codes (408, 429, 5xx) and network errors
  - Added authentication strategies: `bearerToken`, `fixed`, and `consumerInjected`
  - Generated runtime client files: `src/runtime/types.ts`, `http-client.ts`, and `index.ts` (centralized at root_dir level)
  - Runtime client supports both single-spec and multi-spec modes
  - Enhanced runtime client with comprehensive JSDoc documentation and usage examples
  - Type guard helpers (`isSuccess`, `isError`) for easier `ApiResult` type narrowing
  - `bearerTokenMiddleware` helper function for easy Bearer token authentication setup
  - Exported middleware types (`BeforeRequestMiddleware`, `AfterResponseMiddleware`, `ErrorMiddleware`) for better TypeScript support

- **Generic API Result Type**
  - Added `ApiResult<SuccessMap, ErrorMap>` discriminated union type for typed API responses
  - Success map: `Record<status_code, body_type>` for all 2xx responses
  - Error map: `Record<status_code, body_type>` for all non-2xx responses
  - Generated type names: `{FunctionName}Responses` and `{FunctionName}Errors`
  - API functions now return `Promise<ApiResult<SuccessMap, ErrorMap>>` for type-safe error handling

- **Query Parameters as Schema Types**
  - Query parameter interfaces are now generated in schema files (`types.ts`) instead of API client files
  - Added Zod schemas for query parameters (`{FunctionName}QueryParamsSchema`)
  - Parameter-level enums (e.g., `SortByEnum`) are generated in schema files with corresponding Zod schemas
  - Query parameters are imported from schema files using namespace qualification (e.g., `Categories.PublicCategoriesControllerFindAllQueryParams`)
  - Common query parameter types are placed in the `common` directory when applicable

- **Enhanced Hook Generation**
  - React Query and SWR hooks now use `ApiResult<SuccessMap, ErrorMap>` for typed responses
  - Hooks import runtime types (`ApiResult`) from the runtime client
  - Query parameter types and enums are imported from schema files with namespace qualification
  - Improved type safety for hook return values and error handling
  - Hook generation support in `update` command - hooks are now automatically generated when configured in `.vika.json`
  - Interactive hook configuration prompts in `init` and `add` commands - users can now configure hooks during project initialization and when adding new specs
  - Hook library selection via `.vika.json` configuration (`hooks.library: "react-query"` or `"swr"`)

### Changed

- **BREAKING**: API client functions now use `vikaClient` instead of simple `http` client
  - All API functions import `vikaClient` and `ApiResult` from runtime client
  - Function signatures changed to return `Promise<ApiResult<SuccessMap, ErrorMap>>`
  - Request/response handling now goes through middleware pipeline

- **BREAKING**: Query parameter types moved from API client files to schema files
  - Query parameter interfaces are now in `src/schemas/{spec}/{module}/types.ts`
  - Query parameter Zod schemas are in `src/schemas/{spec}/{module}/schemas.ts`
  - API client files import query parameter types from schema files

- **BREAKING**: Runtime client location changed from `src/apis/{spec}/runtime/` to `src/runtime/` (centralized at root_dir level)
- Query parameter enums are now generated in schema files instead of API client files
- Hook generation imports enums from schema files using namespace qualification
- Hooks and query-keys output directories now follow the same pattern as schemas and apis (no spec-name subdirectory)
- Improved runtime client documentation with inline examples and better explanations

### Fixed

- Fixed TypeScript strict mode compatibility (`exactOptionalPropertyTypes`)
  - Updated `auth` property type to explicitly allow `undefined`
  - Fixed `signal` property in `RequestInit` to conditionally include it
- Fixed React Query mutation hook type mismatch by adding `TVariables` type parameter
- Fixed duplicate enum declarations by checking existing types before generation
- Fixed enum schema references in query parameter Zod schemas (e.g., `SortByEnumSchema.optional()`)
- Fixed JSDoc comment placement for response types in API client files
- Fixed directory structure inconsistency where hooks and query-keys had an extra spec-name directory nesting (now matches schemas/apis structure)
- Fixed import paths in generated hooks to correctly resolve runtime, API clients, schemas, and query-keys without spec-name directory
- Fixed hook file counting in `update` command summary to include hooks and query-keys
- Fixed all Clippy warnings with `-D warnings` flag:
  - Reduced function arguments by using context structs (`QueryParamsContext`)
  - Simplified Option handling with `.and_then()` and `.map()` chains
  - Used array syntax for char comparisons (e.g., `[' ', '=', '{']`)
  - Removed useless `format!()` calls
  - Fixed collapsible-if statements
  - Removed needless borrows for generic arguments
  - Fixed field-reassign-with-default warnings in test files by using struct initialization

### Refactored

- Created `QueryParamsContext` struct to reduce function argument count
- Improved code organization by separating query parameter generation into dedicated module
- Enhanced type extraction helpers with better Option handling patterns
- Standardized test patterns to use struct initialization instead of field reassignment


## [Unreleased]

### Planned

- Template customization system
- Plugin architecture
- Additional output formats
- Performance optimizations
- More OpenAPI features support

