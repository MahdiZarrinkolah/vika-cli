# 🚀 vika-cli

### A blazing-fast Rust-powered OpenAPI code generator that produces TypeScript types, Zod schemas, and Fetch clients — designed for real-world DX.

[![CI](https://github.com/MahdiZarrinkolah/vika-cli/workflows/CI/badge.svg)](https://github.com/MahdiZarrinkolah/vika-cli/actions)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)

`vika-cli` is a **blazing-fast**, **production-grade** code generator that transforms any Swagger/OpenAPI specification into fully typed:

- **TypeScript interfaces**
- **Zod validation schemas**
- **Fetch-based API client functions**
- **Module-organized output structure**

Built in Rust for exceptional performance and reliability.

---

# ✨ Features

- 🚀 Ultra-fast Rust engine with deep schema resolution
- 📦 Module-based generation (grouped by Swagger tags)
- 🧬 TypeScript interfaces from OpenAPI schemas
- 🛡️ Zod validation schemas with constraints
- 🔌 Strongly-typed Fetch API client generation
- 🎯 Interactive module selection
- 🎛 Config-driven outputs (`.vika.json`)
- 💾 Spec caching for fast regeneration
- 🔄 Backup system for generated files
- ⚠️ Conflict detection for user-modified files
- 🔍 Inspect command for analyzing specs
- 🎨 Progress indicators and verbose logging
- 🧠 Handles: oneOf, anyOf, allOf, enums, recursion, circular refs
- 🌐 Supports HEAD, OPTIONS, PATCH, all HTTP verbs
- 🎨 Customizable templates (Tera-based) with user overrides
- 🛠 Multi-platform installers + CI/CD automation
- 🔀 Multi-spec support for microservices architectures

---

# ⚖️ Comparison With Other Tools

| Tool                 | Types | Zod | Client      | Rust Speed | Module Selection | Inspect | Cache |
| -------------------- | ----- | --- | ----------- | ---------- | ---------------- | ------- | ----- |
| **vika-cli**         | ✅    | ✅  | Fetch       | ⚡⚡⚡     | ✅               | ✅      | ✅    |
| openapi-typescript   | ✅    | ❌  | ❌          | ❌         | ❌               | ❌      | ❌    |
| Orval                | ⚠️    | ⚠️  | Axios/Fetch | ❌         | ❌               | ❌      | ❌    |
| openapi-client-axios | ❌    | ❌  | Axios       | ❌         | ❌               | ❌      | ❌    |

**Why choose vika-cli?**

- **🚀 Rust-powered**: Blazing fast schema resolution and code generation
- **🛡️ Complete validation**: Native Zod schema generation with full constraint support
- **📦 Modular**: Interactive module selection for selective generation
- **🔍 Built-in inspection**: Analyze specs without generating code
- **💾 Smart caching**: Fast regeneration with intelligent spec caching
- **🎯 Developer experience**: Conflict detection, backups, and progress indicators

---

# 📦 Installation

### macOS & Linux (recommended)

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

### From Source

```bash
git clone https://github.com/MahdiZarrinkolah/vika-cli
cd vika-cli
cargo build --release
```

---

# 🚀 Quick Start

## 1️ Initialize a project

```bash
vika-cli init
```

Creates a `.vika.json` configuration file.

## 2️ Generate code from an OpenAPI specification

### Single Spec Mode

Remote:

```bash
vika-cli generate --spec https://example.com/openapi.json
```

Local:

```bash
vika-cli generate --spec ./swagger.yaml
```

### Multi-Spec Mode (Microservices)

When your `.vika.json` contains a `specs` array, you can generate from multiple specs:

```bash
# Generate all specs
vika-cli generate --all-specs

# Generate a specific spec by name
vika-cli generate --spec ecommerce

# Interactive selection (default)
vika-cli generate
```

Flags:

| Flag | Description |
| ----------- | ------------------------------- |
| `--spec <name>` | Generate specific spec (single or multi-spec mode) |
| `--all-specs` | Generate all specs (multi-spec mode only) |
| `--verbose` | Show detailed output |
| `--cache` | Use cached version of the spec |
| `--backup` | Backup files before overwriting |
| `--force` | Force overwrite conflicts |

The generator will:

1. Parse the spec(s)
2. Extract modules (tags)
3. Ask you which modules to generate
4. Produce TypeScript + Zod + Fetch clients
5. Show a detailed generation summary table

## 3️ Update previously generated modules

```bash
vika-cli update
```

## 4️ Inspect a specification (no generation)

### Single Spec Mode

```bash
vika-cli inspect --spec ./swagger.yaml
```

### Multi-Spec Mode

```bash
# Inspect all specs
vika-cli inspect --all-specs

# Inspect specific spec by name
vika-cli inspect --spec ecommerce
```

Examples:

```bash
# Single spec mode
vika-cli inspect --spec ./swagger.yaml --module products
vika-cli inspect --spec ./swagger.yaml --schemas
vika-cli inspect --spec ./swagger.yaml --json

# Multi-spec mode
vika-cli inspect --spec admin --schemas
vika-cli inspect --all-specs --json
```

---

# 🧠 How It Works

`vika-cli` uses a robust generation pipeline:

### 1. **Spec Parsing**

Reads OpenAPI 3.x JSON/YAML.

### 2. **Module Extraction**

Groups endpoints by Swagger tags.

### 3. **Schema Resolution**

Resolves:

- `$ref`
- Circular dependencies
- Recursive models
- oneOf / anyOf / allOf
- Enum values
- Nullable fields

### 4. **Code Generation**

Produces:

- TypeScript interfaces
- Zod schemas with constraints
- Fetch-based API clients

### 5. **Safe Writing**

- Writes only changed files
- Detects conflicts
- Optional backup mode
- Generates index/barrel files
- Optional Prettier/Biome post-formatting

---

# ⚙️ Configuration (`.vika.json`)

## Single Spec Mode

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
  },
  "spec_path": "https://example.com/openapi.json"
}
```

## Multi-Spec Mode (Microservices)

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
    "headerStrategy": "consumerInjected"
  },
  "modules": {
    "ignore": [],
    "selected": ["products", "orders", "users"]
  },
  "specs": [
    {
      "name": "ecommerce",
      "path": "http://localhost:3000/swagger-ecommerce.json"
    },
    {
      "name": "admin",
      "path": "http://localhost:3000/swagger-admin.json"
    },
    {
      "name": "public",
      "path": "http://localhost:3000/swagger-public.json"
    }
  ]
}
```

### Configuration Options

| Key | Description |
| --------------------- | ------------------------------------------ |
| `spec_path` | Single spec path (URL or file path) - **mutually exclusive with `specs`** |
| `specs` | Array of spec entries for multi-spec mode - **mutually exclusive with `spec_path`** |
| `specs[].name` | Spec identifier (kebab-case recommended, e.g., "ecommerce", "admin") |
| `specs[].path` | Spec path (URL or file path) |
| `schemas.output` | Directory for types + Zod schemas |
| `schemas.naming` | Naming convention for generated types |
| `apis.output` | Directory for API clients |
| `apis.baseUrl` | Base URL prefix for client requests |
| `apis.headerStrategy` | `bearerToken`, `fixed`, `consumerInjected` |
| `modules.ignore` | Skip tagged modules |
| `modules.selected` | Only generate these modules (if specified) |

**Important**: `spec_path` and `specs` are mutually exclusive. Use `spec_path` for single-spec projects, or `specs` for multi-spec microservices architectures.

Full reference: [`docs/configuration.md`](docs/configuration.md)

---

# 🎨 Customizing Templates

`vika-cli` uses **Tera templates** for code generation. You can customize the output format by overriding built-in templates.

## Quick Start

1. **Initialize templates** (copies built-in templates to `.vika/templates/`):
   ```bash
   vika-cli templates init
   ```

2. **List available templates**:
   ```bash
   vika-cli templates list
   ```

3. **Customize templates** in `.vika/templates/`:
   ```bash
   # Edit .vika/templates/type-interface.tera
   # Edit .vika/templates/api-client-fetch.tera
   # etc.
   ```

4. **Regenerate code** - your custom templates will be used automatically:
   ```bash
   vika-cli generate --spec your-spec.yaml
   ```

## Template Files

- `type-interface.tera` - TypeScript interface generation
- `type-enum.tera` - TypeScript enum generation
- `zod-schema.tera` - Zod schema generation
- `api-client-fetch.tera` - API client function generation

**Template Resolution**: User templates in `.vika/templates/` override built-in templates automatically.

Full documentation: [`docs/templates.md`](docs/templates.md)

---

# 🧱 Output Structure

## Single Spec Mode

```
📁 src/
│
├── 📁 schemas/
│   │
│   ├── 📁 products/
│   │   ├── 📄 types.ts          # TypeScript interfaces
│   │   ├── 📄 schemas.ts        # Zod validation schemas
│   │   └── 📄 index.ts          # Barrel exports
│   │
│   ├── 📁 users/
│   │   ├── 📄 types.ts
│   │   ├── 📄 schemas.ts
│   │   └── 📄 index.ts
│   │
│   └── 📁 orders/
│       ├── 📄 types.ts
│       ├── 📄 schemas.ts
│       └── 📄 index.ts
│
└── 📁 apis/
    │
    ├── 📄 http.ts                # HTTP client utility
    │
    ├── 📁 products/
    │   └── 📄 index.ts           # API client functions
    │
    ├── 📁 users/
    │   └── 📄 index.ts
    │
    └── 📁 orders/
        └── 📄 index.ts
```

## Multi-Spec Mode (Microservices)

When using `specs` array, output is organized by spec name:

```
📁 src/
│
├── 📁 schemas/
│   │
│   ├── 📁 ecommerce/             # From ecommerce spec
│   │   ├── 📁 products/
│   │   │   ├── 📄 types.ts
│   │   │   ├── 📄 schemas.ts
│   │   │   └── 📄 index.ts
│   │   └── 📁 orders/
│   │       └── ...
│   │
│   ├── 📁 admin/                 # From admin spec
│   │   ├── 📁 users/
│   │   └── 📁 permissions/
│   │
│   └── 📁 public/                # From public spec
│       └── 📁 catalog/
│
└── 📁 apis/
    │
    ├── 📄 http.ts
    │
    ├── 📁 ecommerce/             # From ecommerce spec
    │   ├── 📁 products/
    │   │   └── 📄 index.ts
    │   └── 📁 orders/
    │       └── 📄 index.ts
    │
    ├── 📁 admin/                 # From admin spec
    │   └── 📁 users/
    │       └── 📄 index.ts
    │
    └── 📁 public/                # From public spec
        └── 📁 catalog/
            └── 📄 index.ts
```

This structure ensures:
- ✅ **Isolation**: Each spec's generated code is separated
- ✅ **No conflicts**: Different specs can have modules with the same name
- ✅ **Clear organization**: Easy to identify which service generated which code

**File types:**

- 🟦 **types.ts** - TypeScript type definitions (`ProductDto`, `UserProfile`, etc.)
- 🟨 **schemas.ts** - Zod validation schemas (`ProductDtoSchema`, etc.)
- 🟩 **index.ts** - Barrel exports and API client functions
- 🟧 **http.ts** - Core HTTP client with fetch wrapper

---

# 📘 Example Output

### TypeScript Types

```ts
export interface ProductDto {
  id: string;
  price: number;
  title: string;
}
```

### Zod Schema

```ts
export const ProductDtoSchema = z.object({
  id: z.string(),
  price: z.number(),
  title: z.string(),
});
```

### Fetch API Client

```ts
export const getProduct = async (id: string): Promise<ProductDto> =>
  http.get(`/products/${id}`);
```

---

# 🧩 Advanced Features

## Multi-Spec Support (Microservices)

`vika-cli` supports generating code from multiple OpenAPI specifications in a single project. This is ideal for microservices architectures where different services expose separate APIs.

### Benefits

- **🔀 Multiple Services**: Generate code for all your microservices in one command
- **📦 Isolated Output**: Each spec's code is organized in separate directories
- **🎯 Selective Generation**: Generate all specs or target specific ones
- **🔄 Unified Workflow**: Single config file manages all specs

### Usage

1. **Configure multiple specs** in `.vika.json`:
   ```json
   {
     "specs": [
       { "name": "ecommerce", "path": "http://localhost:3000/swagger-ecommerce.json" },
       { "name": "admin", "path": "http://localhost:3000/swagger-admin.json" },
       { "name": "public", "path": "http://localhost:3000/swagger-public.json" }
     ]
   }
   ```

2. **Generate all specs**:
   ```bash
   vika-cli generate --all-specs
   ```

3. **Generate specific spec**:
   ```bash
   vika-cli generate --spec ecommerce
   ```

4. **Interactive selection** (default):
   ```bash
   vika-cli generate
   # Prompts: "Which spec do you want to generate?"
   ```

### Spec Naming Rules

- Use **kebab-case** (e.g., `ecommerce`, `order-service`, `user-api`)
- Names must be **unique** within the `specs` array
- Names are used in directory paths: `apis/{spec_name}/{module}/`

### ⚡ Caching

```bash
vika-cli generate --cache
```

Each spec is cached independently using its name as part of the cache key.

### 🛡 Backup Mode

```bash
vika-cli generate --backup
```

Backups stored in:

```
.vika-backup/<timestamp>/
```

### ⚠ Conflict Detection

Warns if manually modified files would be overwritten.

### 🧪 Snapshot Testing

Using `insta` for generator correctness.

---

# 🧱 Architecture Overview

- **Commands**: init, generate, update, inspect
- **Generator Core**: TS/Zod/API client generation
- **Schema Resolver**: Handles refs, unions, recursion
- **Writer System**: Diffs, backups, conflict detection
- **Config System**: Load & validate `.vika.json`
- **Error System**: Structured typed errors
- **Utilities**: caching, formatting, progress indicators

Details: [`docs/architecture.md`](docs/architecture.md)

---

# 🔄 Development & Release

### CI includes:

- `cargo fmt`
- `cargo clippy`
- Unit tests
- Snapshot tests
- Multi-platform builds

### Releases

```bash
cargo release patch
```

Semantic versioning applies:

- **MAJOR** = breaking changes
- **MINOR** = new features
- **PATCH** = fixes

See: `CHANGELOG.md`

---

# 🤝 Contributing

1. Fork
2. Create a feature branch
3. Make changes with tests
4. Submit a PR

Guide: [`docs/contributing.md`](docs/contributing.md)

---

# 📜 License

MIT

---

# 🔗 Links

- 🔗 **GitHub**: [https://github.com/MahdiZarrinkolah/vika-cli](https://github.com/MahdiZarrinkolah/vika-cli)
- 📚 **Documentation**: /docs
- 🐞 **Issue Tracker**: [https://github.com/MahdiZarrinkolah/vika-cli/issues](https://github.com/MahdiZarrinkolah/vika-cli/issues)

---

# 🎉 Thank You

`vika-cli` is now a fully production-grade OpenAPI codegen tool.

Enjoy building! 🚀
