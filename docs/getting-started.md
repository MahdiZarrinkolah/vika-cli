# Getting Started with vika-cli

This guide will help you get started with `vika-cli` and generate your first TypeScript API client.

## Prerequisites

- Node.js and TypeScript project
- An OpenAPI/Swagger specification (JSON or YAML)
- Rust 1.70+ (if building from source)

## Installation

Choose your preferred installation method:

### Quick Install (Recommended)

**macOS/Linux:**
```bash
curl -fsSL https://github.com/MahdiZarrinkolah/vika-cli/releases/latest/download/install.sh | sh
```

**Windows (PowerShell):**
```powershell
irm https://github.com/MahdiZarrinkolah/vika-cli/releases/latest/download/install.ps1 | iex
```

### Cargo Install

```bash
cargo install vika-cli
```

## Quick Start

### Step 1: Initialize Your Project

```bash
vika-cli init
```

`init` collects the first spec (name, path, outputs) and writes `.vika.json`.  
To register additional specs later, run `vika-cli add` and answer the same prompts.

### Step 2: Generate Code

```bash
# Interactive selection (default)
vika-cli generate

# Force a specific spec
vika-cli generate --spec ecommerce

# Regenerate everything
vika-cli generate --all-specs
```

### Step 3: Select Modules

The tool will display available modules (grouped by OpenAPI tags) and let you select which ones to generate:

```
Available modules:
  [ ] users
  [ ] products
  [ ] orders
  [ ] auth

Select modules to generate (space to toggle, enter to confirm):
```

### Step 4: Use Generated Code

The generated code will be in your configured output directories (default: `src/schemas/` and `src/apis/`).

```typescript
import { getProduct } from './apis/products';
import { ProductDto } from './schemas/products';

const product: ProductDto = await getProduct('123');
```

## Common Workflows

### Updating Generated Code

After your API changes, regenerate code:

```bash
vika-cli update
```

This uses your saved configuration and regenerates all previously selected modules.

### Inspecting an API

Before generating, inspect what will be generated:

```bash
vika-cli inspect --spec ecommerce
```

### Using Caching

For remote specs, use caching for faster regeneration:

```bash
vika-cli generate --spec ecommerce --cache
```

### Creating Backups

Always create backups before regenerating:

```bash
vika-cli generate --spec ecommerce --backup
```

## Next Steps

- Read the [Configuration Guide](configuration.md) to customize output
- Learn about [Template Customization](templates.md)
- Check [Troubleshooting](troubleshooting.md) for common issues
- Explore the [Architecture](architecture.md) documentation

