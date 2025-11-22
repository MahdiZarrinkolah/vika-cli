# Configuration Reference

Complete reference for `.vika.json` configuration file.

## Configuration File Location

The configuration file `.vika.json` should be in your project root directory. Create it with:

```bash
vika-cli init
```

## Configuration Schema

```json
{
  "rootDir": "src",
  "schema": "openapi",
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
    "ignore": ["Auth"],
    "selected": []
  },
  "spec_path": "https://api.example.com/openapi.json"
}
```

## Field Reference

### `rootDir`

**Type**: `string`  
**Default**: `"src"`  
**Description**: Root directory for all generated files.

### `schema`

**Type**: `string`  
**Default**: `"openapi"`  
**Description**: OpenAPI specification version. Currently only `"openapi"` is supported.

### `schemas.output`

**Type**: `string`  
**Default**: `"src/schemas"`  
**Description**: Output directory for TypeScript types and Zod schemas.

### `schemas.naming`

**Type**: `"PascalCase" | "camelCase" | "snake_case"`  
**Default**: `"PascalCase"`  
**Description**: Naming convention for generated schema types.

Examples:
- `PascalCase`: `UserProfile`, `ProductDto`
- `camelCase`: `userProfile`, `productDto`
- `snake_case`: `user_profile`, `product_dto`

### `apis.output`

**Type**: `string`  
**Default**: `"src/apis"`  
**Description**: Output directory for API client functions.

### `apis.style`

**Type**: `string`  
**Default**: `"fetch"`  
**Description**: API client style. Currently only `"fetch"` is supported.

### `apis.baseUrl`

**Type**: `string | null`  
**Default**: `null`  
**Description**: Base URL prefix for API endpoints.

Supports environment variable substitution:
```json
{
  "apis": {
    "baseUrl": "${API_BASE_URL}/v1"
  }
}
```

### `apis.headerStrategy`

**Type**: `"bearerToken" | "fixed" | "consumerInjected"`  
**Default**: `"bearerToken"`  
**Description**: How authentication headers are generated.

- `bearerToken`: Adds `Authorization: Bearer ${token}` header (token injected by consumer)
- `fixed`: Adds fixed headers from config (not yet implemented)
- `consumerInjected`: Expects headers to be injected by consumer code

### `modules.ignore`

**Type**: `string[]`  
**Default**: `[]`  
**Description**: List of module tags to ignore during generation. These modules won't appear in the selection prompt.

### `modules.selected`

**Type**: `string[]`  
**Default**: `[]`  
**Description**: List of selected modules. Automatically populated after first generation.

### `spec_path`

**Type**: `string | null`  
**Default**: `null`  
**Description**: Path to OpenAPI specification. Can be a local file path or URL.

## Environment Variables

You can use environment variables in configuration values:

```json
{
  "apis": {
    "baseUrl": "${API_BASE_URL}/v1"
  }
}
```

## Validation

The configuration is validated on load. Common validation errors:

- Invalid output paths (must be relative, not absolute system paths)
- Unsupported API style
- Invalid naming convention

## Examples

### Minimal Configuration

```json
{
  "rootDir": "src"
}
```

### Full Configuration

```json
{
  "rootDir": "src",
  "schemas": {
    "output": "src/generated/schemas",
    "naming": "camelCase"
  },
  "apis": {
    "output": "src/generated/apis",
    "style": "fetch",
    "baseUrl": "/api/v2",
    "headerStrategy": "consumerInjected"
  },
  "modules": {
    "ignore": ["Internal", "Admin"]
  },
  "spec_path": "./openapi.json"
}
```

