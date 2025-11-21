# vika-cli

A Rust-based CLI tool that generates TypeScript typings, Zod schemas, and Fetch-based API clients from Swagger/OpenAPI specifications.

## Features

- 🚀 Generate TypeScript interfaces from OpenAPI schemas
- ✅ Generate Zod validation schemas
- 🔌 Generate Fetch-based HTTP client functions
- 📦 Module-based code generation (grouped by Swagger tags)
- 🎯 Interactive module selection
- ⚙️ Configurable output directories

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

The tool will:
1. Fetch and parse the Swagger/OpenAPI spec
2. Display available modules (tags)
3. Let you interactively select which modules to generate
4. Generate TypeScript types, Zod schemas, and API client functions

### 3. Update generated code

```bash
vika-cli update
```

Regenerates code for previously selected modules without interactive prompts.

## Configuration

The `.vika.json` configuration file:

```json
{
  "rootDir": "src",
  "schemas": {
    "output": "src/schemas"
  },
  "apis": {
    "output": "src/apis",
    "style": "fetch"
  },
  "modules": {
    "ignore": ["Auth"]
  }
}
```

### Configuration Options

- `rootDir`: Root directory for generated files
- `schemas.output`: Output directory for TypeScript types and Zod schemas
- `apis.output`: Output directory for API client functions
- `apis.style`: API client style (currently only "fetch" is supported)
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

## Requirements

- Rust 1.70+ (for building)
- Node.js/TypeScript project (for generated code)

## License

MIT

