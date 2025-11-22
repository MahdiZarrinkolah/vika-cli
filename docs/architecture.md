# Architecture

High-level overview of `vika-cli`'s architecture and design decisions.

## Overview

`vika-cli` is built with a modular, layered architecture:

```
┌─────────────────────────────────────┐
│         CLI Layer                   │
│  (Commands, User Interaction)       │
└─────────────────────────────────────┘
              │
┌─────────────────────────────────────┐
│      Generator Core                 │
│  (Parsing, Code Generation)         │
└─────────────────────────────────────┘
              │
┌─────────────────────────────────────┐
│      Support Systems                │
│  (Config, Cache, Errors, Utils)     │
└─────────────────────────────────────┘
```

## Module Structure

### CLI Layer (`src/cli.rs`, `src/commands/`)

Handles user interaction and command parsing:

- **`cli.rs`**: Command-line argument parsing using `clap`
- **`commands/init.rs`**: Project initialization
- **`commands/generate.rs`**: Code generation workflow
- **`commands/update.rs`**: Regeneration workflow
- **`commands/inspect.rs`**: Spec inspection

### Generator Core (`src/generator/`)

Core code generation logic:

- **`swagger_parser.rs`**: OpenAPI spec parsing and module extraction
- **`schema_resolver.rs`**: Schema reference resolution and dependency tracking
- **`ts_typings.rs`**: TypeScript type generation
- **`zod_schema.rs`**: Zod schema generation
- **`api_client.rs`**: API client function generation
- **`writer.rs`**: File writing with conflict detection
- **`module_selector.rs`**: Interactive module selection
- **`utils.rs`**: Utility functions (naming, formatting)

### Support Systems

- **`config/`**: Configuration management (loading, validation)
- **`cache.rs`**: OpenAPI spec caching
- **`error.rs`**: Structured error handling
- **`progress.rs`**: Progress reporting
- **`formatter.rs`**: Code formatting utilities

## Data Flow

### Generation Workflow

```
1. Parse CLI arguments
   ↓
2. Load configuration (.vika.json)
   ↓
3. Fetch/parse OpenAPI spec
   ↓
4. Extract modules (from tags)
   ↓
5. Interactive module selection
   ↓
6. Resolve schema dependencies
   ↓
7. Generate TypeScript types
   ↓
8. Generate Zod schemas
   ↓
9. Generate API client functions
   ↓
10. Write files (with conflict detection)
   ↓
11. Save configuration (selected modules)
```

## Key Design Decisions

### 1. Module-Based Generation

Code is organized by OpenAPI tags, creating logical modules. This:
- Improves code organization
- Allows selective generation
- Reduces bundle size

### 2. Schema Resolution

Uses a dependency graph to:
- Handle circular references
- Resolve deep nesting
- Track schema usage across modules

### 3. Conflict Detection

Uses file metadata to detect user modifications:
- Prevents accidental overwrites
- Preserves user customizations
- Provides backup system

### 4. Caching

Caches remote OpenAPI specs:
- Faster regeneration
- Offline capability
- Reduced network usage

## Error Handling

Uses structured error types (`thiserror`):

- **`VikaError`**: Main error enum
- **`GenerationError`**: Generation-specific errors
- **`ConfigError`**: Configuration errors
- **`SchemaError`**: Schema resolution errors
- **`FileSystemError`**: File I/O errors
- **`NetworkError`**: Network-related errors

All errors implement `std::error::Error` and provide context.

## Testing Strategy

- **Unit tests**: Test individual functions in isolation
- **Integration tests**: Test full workflows end-to-end
- **Snapshot tests**: Verify generated code format using `insta`

See [testing.md](testing.md) for details.

## Performance Considerations

- **Async I/O**: Uses `tokio` for non-blocking operations
- **Caching**: Reduces redundant network requests
- **Lazy evaluation**: Only generates selected modules
- **Incremental writing**: Skips unchanged files

## Future Improvements

- Parallel code generation
- Incremental regeneration (only changed modules)
- Template system improvements
- Plugin architecture for custom generators

