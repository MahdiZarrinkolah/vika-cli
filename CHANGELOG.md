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

### Changed

- Enhanced HTTP client template to support GET requests with body parameters
- Improved enum schema reference resolution to check enum registry before falling back to schema name
- Enum schemas are now used directly (without `z.lazy()`) since they don't have circular dependencies


## [Unreleased]

### Planned

- Template customization system
- Plugin architecture
- Additional output formats
- Performance optimizations
- More OpenAPI features support

