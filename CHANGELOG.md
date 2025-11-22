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

## [Unreleased]

### Planned

- Template customization system
- Plugin architecture
- Additional output formats
- Performance optimizations
- More OpenAPI features support

