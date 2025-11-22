# vika-cli

A Rust-based CLI tool that generates TypeScript typings, Zod schemas, and Fetch-based API clients from Swagger/OpenAPI specifications.

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

```bash
cargo install --path .
```

Or build from source:

```bash
cargo build --release
```

## Usage

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

## Requirements

- Rust 1.70+ (for building)
- Node.js/TypeScript project (for generated code)

## License

MIT

