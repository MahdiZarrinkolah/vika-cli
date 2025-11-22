# vika-cli

A production-grade Rust CLI tool that generates TypeScript typings, Zod schemas, and Fetch-based API clients from Swagger/OpenAPI specifications.

[![CI](https://github.com/MahdiZarrinkolah/vika-cli/workflows/CI/badge.svg)](https://github.com/MahdiZarrinkolah/vika-cli/actions)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

## Features

- 🚀 Generate TypeScript interfaces from OpenAPI schemas
- ✅ Generate Zod validation schemas
- 🔌 Generate Fetch-based HTTP client functions
- 📦 Module-based code generation (grouped by Swagger tags)
- 🎯 Interactive module selection
- ⚙️ Configurable output directories
- 🎨 Progress indicators and verbose mode
- 💾 Spec caching for faster regeneration
- 🔄 Backup system for generated files
- 🛡️ Conflict detection for user-modified files
- 🔍 Inspect command for analyzing OpenAPI specs
- 🎯 Support for HEAD, OPTIONS HTTP methods
- 📊 Formatted table output for module summaries

## Installation

### Quick Install (macOS/Linux)

```bash
curl -fsSL https://github.com/MahdiZarrinkolah/vika-cli/releases/latest/download/install.sh | sh
```

### Windows (PowerShell)

```powershell
irm https://github.com/MahdiZarrinkolah/vika-cli/releases/latest/download/install.ps1 | iex
```

### Cargo

```bash
cargo install vika-cli
```

### Build from Source

```bash
git clone https://github.com/MahdiZarrinkolah/vika-cli.git
cd vika-cli
cargo build --release
```

## Getting Started

### 1. Initialize a project

```bash
vika-cli init
```

This creates a `.vika.json` configuration file and sets up the project structure.

### 2. Generate code from Swagger spec

```bash
vika-cli generate --spec https://example.com/swagger.json
```

Or from a local file:

```bash
vika-cli generate --spec ./swagger.yaml
```

**Available flags:**
- `--verbose`: Enable verbose output with detailed progress information
- `--cache`: Use cached spec if available (faster for remote specs)
- `--backup`: Create backup of existing files before writing
- `--force`: Force overwrite user-modified files

The tool will:
1. Fetch and parse the Swagger/OpenAPI spec (with optional caching)
2. Display available modules (tags)
3. Let you interactively select which modules to generate
4. Generate TypeScript types, Zod schemas, and API client functions
5. Show formatted table summary of generated files

### 3. Update generated code

```bash
vika-cli update
```

Regenerates code for previously selected modules without interactive prompts.

### 4. Inspect OpenAPI spec

```bash
vika-cli inspect --spec https://example.com/swagger.json
```

Analyze an OpenAPI spec without generating code:

```bash
# Show all modules
vika-cli inspect --spec ./swagger.yaml

# Show specific module details
vika-cli inspect --spec ./swagger.yaml --module products

# Show schema details
vika-cli inspect --spec ./swagger.yaml --module products --schemas

# JSON output
vika-cli inspect --spec ./swagger.yaml --json
```

## How Generation Works

`vika-cli` follows a multi-stage generation pipeline:

1. **Spec Parsing**: Fetches and parses OpenAPI/Swagger specifications (supports JSON and YAML)
2. **Module Extraction**: Groups endpoints by tags, creating logical modules
3. **Schema Resolution**: Resolves `$ref` references and builds dependency graphs
4. **Type Generation**: Converts OpenAPI schemas to TypeScript interfaces
5. **Zod Generation**: Creates Zod validation schemas from OpenAPI constraints
6. **API Client Generation**: Generates type-safe Fetch-based API functions
7. **File Writing**: Writes generated code with conflict detection and backup support

The generator handles:
- Circular dependencies (detected and handled gracefully)
- Deep nesting (unlimited depth)
- Union types (`oneOf`, `anyOf`)
- AllOf composition
- Enums and string constraints
- Optional vs required fields
- Nullable types

## Configuration

The `.vika.json` configuration file:

```json
{
  "rootDir": "src",
  "schemas": {
    "output": "src/schemas",
    "naming": "PascalCase"
  },
  "apis": {
    "output": "src/apis",
    "style": "fetch",
    "baseUrl": "/api/v1",
    "headerStrategy": "bearerToken"
  },
  "modules": {
    "ignore": ["Auth"]
  }
}
```

### Configuration Options

- `rootDir`: Root directory for generated files
- `schemas.output`: Output directory for TypeScript types and Zod schemas
- `schemas.naming`: Naming convention for schemas (`PascalCase`, `camelCase`, `snake_case`)
- `apis.output`: Output directory for API client functions
- `apis.style`: API client style (currently only "fetch" is supported)
- `apis.baseUrl`: Base URL prefix for API endpoints (supports `${ENV_VAR}` substitution)
- `apis.headerStrategy`: Header generation strategy (`bearerToken`, `fixed`, `consumerInjected`)
- `modules.ignore`: List of module tags to ignore during generation

See [docs/configuration.md](docs/configuration.md) for complete configuration reference.

## Generated Code Structure

```
src/
├── schemas/
│   └── <module>/
│       ├── types.ts      # TypeScript interfaces
│       ├── schemas.ts    # Zod validation schemas
│       └── index.ts      # Barrel exports
└── apis/
    ├── http.ts           # HTTP client utility
    └── <module>/
        └── index.ts      # API client functions
```

## Example Generated Code

### TypeScript Types

```typescript
export interface ProductDto {
  id: string;
  price: number;
  title: string;
}
```

### Zod Schemas

```typescript
import { z } from "zod";

export const ProductDtoSchema = z.object({
  id: z.string(),
  price: z.number(),
  title: z.string(),
});
```

### API Client Functions

```typescript
import { http } from "../http";

export const getProduct = async (id: string): Promise<ProductDto> => {
  const url = `/products/${id}`;
  return http.get<ProductDto>(url);
};
```

## Customizing Output

### Template System

`vika-cli` uses templates for code generation. Templates are located in `src/templates/`:

- `http_client.ts`: HTTP client utility template
- `index.ts`: Barrel export template

To customize output, you can modify these templates. See [docs/templates.md](docs/templates.md) for details.

### Naming Conventions

Control how schemas are named using the `schemas.naming` config option:

- `PascalCase`: `UserProfile` (default)
- `camelCase`: `userProfile`
- `snake_case`: `user_profile`

### Header Strategies

Configure how authentication headers are generated:

- `bearerToken`: Adds `Authorization: Bearer ${token}` header
- `fixed`: Adds fixed headers from config
- `consumerInjected`: Expects headers to be injected by consumer

## Advanced Features

### Caching

The tool caches downloaded OpenAPI specs in `.vika-cache/` for faster regeneration:

```bash
vika-cli generate --spec https://api.example.com/openapi.json --cache
```

### Backup System

Create backups before overwriting files:

```bash
vika-cli generate --spec ./swagger.yaml --backup
```

Backups are stored in `.vika-backup/TIMESTAMP/` with preserved directory structure.

### Conflict Detection

The tool detects if generated files were modified by the user and warns before overwriting. Use `--force` to override:

```bash
vika-cli generate --spec ./swagger.yaml --force
```

### Error Handling

All errors are structured and provide clear messages. Common error types:
- Schema errors (missing references, circular dependencies)
- Config errors (invalid paths, missing fields)
- Network errors (fetch failures, invalid URLs)
- File system errors (permission denied, disk full)

## Examples

### Real-World Usage

```bash
# Generate from remote API
vika-cli generate --spec https://api.example.com/openapi.json --cache

# Generate specific modules only
vika-cli generate --spec ./swagger.yaml
# Then select modules interactively

# Update after API changes
vika-cli update --force

# Inspect before generating
vika-cli inspect --spec ./swagger.yaml --json
```

See [docs/getting-started.md](docs/getting-started.md) for more examples.

## Troubleshooting

### Common Issues

**Problem**: "Spec path required" error
- **Solution**: Ensure you provide `--spec` flag or set `spec_path` in `.vika.json`

**Problem**: Circular dependency warnings
- **Solution**: This is handled automatically. The generator uses lazy references for circular deps.

**Problem**: Generated files conflict with user modifications
- **Solution**: Use `--force` to overwrite, or `--backup` to create backups first

**Problem**: Network errors when fetching remote specs
- **Solution**: Check your internet connection, or use `--cache` with a previously cached spec

See [docs/troubleshooting.md](docs/troubleshooting.md) for more solutions.

## Architecture

`vika-cli` is built with a modular architecture:

- **CLI Layer**: Command parsing and user interaction (`src/cli.rs`, `src/commands/`)
- **Generator Core**: Code generation logic (`src/generator/`)
- **Config System**: Configuration management (`src/config/`)
- **Error Handling**: Structured error types (`src/error.rs`)
- **Utilities**: Caching, progress reporting, formatting (`src/cache.rs`, `src/progress.rs`, `src/formatter.rs`)

See [docs/architecture.md](docs/architecture.md) for detailed architecture documentation.

## Versioning

`vika-cli` follows [Semantic Versioning](https://semver.org/):

- **Major (x.0.0)**: Breaking changes to generated code format, CLI interface changes
- **Minor (0.x.0)**: New features, new generation options, backward-compatible additions
- **Patch (0.0.x)**: Bug fixes, performance improvements, documentation updates

### Automated Versioning

Version bumping and changelog updates are automated:

```bash
# Using cargo-release (recommended)
cargo release patch   # or minor, major
```

**📖 See [RELEASE.md](RELEASE.md) for complete release process documentation.**

Quick references:
- [docs/development/release-quick-start.md](docs/development/release-quick-start.md) - Quick start guide
- [docs/development/release-setup.md](docs/development/release-setup.md) - Setup instructions

See [CHANGELOG.md](CHANGELOG.md) for version history.

## Contributing

Contributions are welcome! Please see [docs/contributing.md](docs/contributing.md) for guidelines.

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests
5. Submit a pull request

## Requirements

- Rust 1.70+ (for building)
- Node.js/TypeScript project (for generated code)

## License

MIT

## Links

- [GitHub Repository](https://github.com/MahdiZarrinkolah/vika-cli)
- [Documentation](docs/)
- [Issue Tracker](https://github.com/MahdiZarrinkolah/vika-cli/issues)
