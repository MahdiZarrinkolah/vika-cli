# Configuration Reference

`vika-cli` now standardizes on a multi-spec configuration model. Even single-spec projects use the same structure, which keeps migrations simple and enables `vika-cli add` to append additional specs at any time.

## Location

`.vika.json` lives in your project root. Create it with:

```bash
vika-cli init
```

## Example

```json
{
  "$schema": "https://raw.githubusercontent.com/vikarno/vika-cli/main/schema/vika-config.schema.json",
  "root_dir": "src",
  "generation": {
    "enable_cache": true,
    "enable_backup": false,
    "conflict_strategy": "ask"
  },
  "specs": [
    {
      "name": "ecommerce",
      "path": "http://localhost:3000/swagger-ecommerce.json",
      "schemas": {
        "output": "src/schemas/ecommerce",
        "naming": "PascalCase"
      },
      "apis": {
        "output": "src/apis/ecommerce",
        "style": "fetch",
        "base_url": "https://api.example.com",
        "header_strategy": "consumerInjected"
      },
      "modules": {
        "ignore": [],
        "selected": ["orders", "payments", "users"]
      }
    }
  ]
}
```

## Global fields

| Field | Type | Description |
| --- | --- | --- |
| `$schema` | `string` | Points to the published JSON schema; updated automatically by `init`. |
| `root_dir` | `string` (default `src`) | Base directory for generated assets. |
| `generation.enable_cache` | `boolean` (default `true`) | Cache parsed specs under `.vika-cache` for faster reruns. |
| `generation.enable_backup` | `boolean` (default `false`) | Create timestamped backups before overwriting files. |
| `generation.conflict_strategy` | `ask | force | skip` (default `ask`) | How the writer reacts when existing files differ from newly generated content. |
| `specs` | `array` | Required list of spec entries. `vika-cli init` creates one entry; `vika-cli add` appends more. |

## Spec entries

Each spec fully controls its own outputs.

| Field | Type | Description |
| --- | --- | --- |
| `name` | `string` | Unique identifier (kebab-case recommended). Used in directory paths and CLI selectors. |
| `path` | `string` | Local path or URL to the OpenAPI document. Remote URLs are cached using this name/path combo. |
| `schemas.output` | `string` | Destination folder for this spec’s TypeScript types and Zod schemas. |
| `schemas.naming` | `PascalCase | camelCase | snake_case | kebab-case` (default `PascalCase`) | Controls casing for generated type names. |
| `apis.output` | `string` | Destination folder for Fetch clients. |
| `apis.style` | `string` (currently only `fetch`) | API client template to use. |
| `apis.base_url` | `string?` | Optional base URL baked into generated clients. Environment variable placeholders (`${API_BASE_URL}`) are supported. |
| `apis.header_strategy` | `consumerInjected | bearerToken | fixed` (default `consumerInjected`) | Determines how request headers are wired up. |
| `modules.ignore` | `string[]` | Tags to exclude from prompts and generation. |
| `modules.selected` | `string[]` | Tags to include. Empty arrays trigger interactive selection; once you pick modules, `generate`/`update` writes them back so future runs can skip the prompt. |

## Environment variables

Any string field accepts `${VAR_NAME}` placeholders. The CLI substitutes them with the process environment at runtime. Example:

```json
{
  "specs": [
    {
      "name": "admin",
      "path": "https://internal.example.com/openapi.json",
      "apis": {
        "output": "src/apis/admin",
        "style": "fetch",
        "base_url": "${ADMIN_API}/v1",
        "header_strategy": "bearerToken"
      },
      "schemas": {
        "output": "src/schemas/admin",
        "naming": "PascalCase"
      },
      "modules": {
        "ignore": [],
        "selected": []
      }
    }
  ]
}
```

## Validation rules

`vika-cli` validates the config on every command:

- At least one spec must exist.
- Spec names must be unique and match `^[A-Za-z0-9_-]+$`.
- `path` cannot be empty. URLs and local paths are accepted.
- Output directories can be relative or absolute, but absolute paths are checked to avoid system directories like `/etc`.
- Only the `fetch` API style is currently supported.
- `modules.selected` is allowed to be empty; the CLI will prompt and then persist your selection.

## Managing specs

- `vika-cli init` boots your project and captures the first spec.
- `vika-cli add` appends additional specs interactively (same questions as `init`).
- `vika-cli inspect`, `generate`, and `update` all operate on the `specs` array. Use `--spec <name>` or `--all-specs` to control scope.

Keeping everything inside `specs` removes the old “single vs multi” divergence and ensures every spec has independent schema/API outputs, module filters, and common files.

