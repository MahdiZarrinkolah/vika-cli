# Template System

`vika-cli` uses **Tera templates** to generate TypeScript types, Zod schemas, and API client functions. You can customize these templates to change the output format without modifying the source code.

## Overview

The template system provides:

- **Built-in templates**: Default templates embedded in the binary
- **User override templates**: Custom templates in `.vika/templates/` that override built-in ones
- **Strongly-typed contexts**: Type-safe data structures passed to templates
- **Template resolution**: Automatic fallback from user templates to built-in templates

## Template Location

### Built-in Templates

Built-in templates are embedded in the `vika-cli` binary and located in `builtin/templates/`:

- `type-interface.tera` - TypeScript interface generation
- `type-enum.tera` - TypeScript enum type generation
- `type-alias.tera` - TypeScript type alias generation
- `zod-schema.tera` - Zod schema generation
- `zod-enum.tera` - Zod enum schema generation
- `api-client-fetch.tera` - Fetch-based API client function generation

### User Templates

User templates override built-in templates and are located in `.vika/templates/`:

```
.vika/
  templates/
    type-interface.tera    # Overrides built-in template
    zod-schema.tera        # Overrides built-in template
    api-client-fetch.tera  # Overrides built-in template
```

## Template Commands

### List Templates

View all available templates and which ones are overridden:

```bash
vika-cli templates list
```

**Output:**
```
Built-in templates:

  ✓ type-interface (overridden)
  - zod-schema
  - api-client-fetch
  ...

User overrides:

  - type-interface
```

### Initialize Templates

Copy all built-in templates to `.vika/templates/` for customization:

```bash
vika-cli templates init
```

This creates `.vika/templates/` and copies all built-in templates. Existing templates are skipped (not overwritten).

## Template Syntax

Templates use [Tera](https://keats.github.io/tera/) syntax, which is similar to Jinja2/Django templates.

### Basic Syntax

```tera
{# Comments #}
{{ variable_name }}
{% if condition %}...{% endif %}
{% for item in items %}...{% endfor %}
```

### Example: Type Interface Template

```tera
{% if description %}/**
 * {{ description }}
 */
{% endif %}
export interface {{ type_name }} {
{% for field in fields %}
{% if field.description %}  /**
   * {{ field.description }}
   */
{% endif %}  {{ field.name }}{% if field.optional %}?{% endif %}: {{ field.type_name }};
{% endfor %}
}
```

## Template Contexts

Each template receives a strongly-typed context object with the data needed for generation.

### TypeContext

Used for TypeScript interface/enum/alias generation:

```rust
pub struct TypeContext {
    pub type_name: String,
    pub fields: Vec<Field>,
    pub is_enum: bool,
    pub enum_values: Option<Vec<String>>,
    pub is_alias: bool,
    pub alias_target: Option<String>,
    pub description: Option<String>,
}

pub struct Field {
    pub name: String,
    pub type_name: String,
    pub optional: bool,
    pub description: Option<String>,
}
```

**Available in templates:**
- `type_name` - Name of the type (e.g., "User")
- `fields` - Array of field objects (for interfaces)
- `fields[].name` - Field name
- `fields[].type_name` - TypeScript type name
- `fields[].optional` - Boolean indicating if field is optional
- `fields[].description` - Optional field description
- `is_enum` - Boolean indicating if this is an enum
- `enum_values` - Array of enum values (for enums)
- `is_alias` - Boolean indicating if this is a type alias
- `alias_target` - Target type for aliases (e.g., "Record<string, any>")
- `description` - Optional type description

### ZodContext

Used for Zod schema generation:

```rust
pub struct ZodContext {
    pub schema_name: String,
    pub zod_expr: String,      // Pre-built Zod expression
    pub is_enum: bool,
    pub enum_values: Option<Vec<String>>,
    pub description: Option<String>,
}
```

**Available in templates:**
- `schema_name` - Name of the schema (e.g., "UserSchema")
- `zod_expr` - Pre-built Zod expression string
- `is_enum` - Boolean indicating if this is an enum schema
- `enum_values` - Array of enum values (for enum schemas)
- `description` - Optional schema description

### ApiContext

Used for API client function generation:

```rust
pub struct ApiContext {
    pub function_name: String,
    pub operation_id: Option<String>,
    pub http_method: String,
    pub path: String,
    pub path_params: Vec<Parameter>,
    pub query_params: Vec<Parameter>,
    pub request_body: Option<RequestBody>,
    pub responses: Vec<Response>,
    pub type_imports: String,
    pub http_import: String,
    pub return_type: String,
    pub function_body: String,
    pub module_name: String,
    pub params: String,
    pub description: String,
}

pub struct Parameter {
    pub name: String,
    pub param_type: String,
    pub optional: bool,
    pub description: Option<String>,
}
```

**Available in templates:**
- `function_name` - Generated function name (e.g., "getUser")
- `operation_id` - OpenAPI operationId (if available)
- `http_method` - HTTP method (e.g., "get", "post")
- `path` - API path (e.g., "/users/{id}")
- `path_params` - Array of path parameters
- `query_params` - Array of query parameters
- `request_body` - Optional request body information
- `responses` - Array of response objects
- `type_imports` - Pre-formatted import statements
- `http_import` - Relative path to runtime client (e.g., `./runtime` or `../runtime`)
- `return_type` - TypeScript return type (e.g., ": Promise<User>")
- `function_body` - Pre-formatted function body
- `module_name` - Module name (e.g., "users")
- `params` - Pre-formatted parameter string
- `description` - Operation description/summary

## Customization Examples

### Example 1: Add JSDoc Comments to Interfaces

Create `.vika/templates/type-interface.tera`:

```tera
{% if description %}/**
 * {{ description }}
 */
{% endif %}
export interface {{ type_name }} {
{% for field in fields %}
{% if field.description %}  /**
   * {{ field.description }}
   */
{% endif %}  {{ field.name }}{% if field.optional %}?{% endif %}: {{ field.type_name }};
{% endfor %}
}
```

### Example 2: Custom Zod Schema Format

Create `.vika/templates/zod-schema.tera`:

```tera
{% if description %}/**
 * {{ description }}
 */
{% endif %}
export const {{ schema_name }}Schema: z.ZodType<any> = {{ zod_expr }};
```

### Example 3: Custom API Function Format

Create `.vika/templates/api-client-fetch.tera`:

```tera
import { http } from "{{ http_import }}";
{{ type_imports }}
{% if description %}/**
 * {{ description }}
 */
{% endif %}
export async function {{ function_name }}({{ params }}){{ return_type }} {
{{ function_body }}
}
```

## Template Resolution Priority

When generating code, templates are resolved in this order:

1. **User template** (`.vika/templates/{name}.tera`) - Highest priority
2. **Built-in template** (`builtin/templates/{name}.tera`) - Fallback

If a user template exists, it will be used. Otherwise, the built-in template is used.

## Best Practices

### 1. Start with Built-in Templates

Always initialize templates first:

```bash
vika-cli templates init
```

This gives you a copy of all built-in templates to modify.

### 2. Test Your Changes

After modifying templates, test with a small spec:

```bash
vika-cli generate --spec your-spec.yaml
```

### 3. Version Control

Commit your custom templates to version control:

```bash
git add .vika/templates/
git commit -m "Add custom templates"
```

### 4. Keep Templates Simple

Templates should focus on formatting. Complex logic should remain in the Rust code.

### 5. Document Custom Templates

Add comments to your templates explaining any customizations:

```tera
{# Custom template: Adds JSDoc comments to all interfaces #}
{% if description %}/**
 * {{ description }}
 */
{% endif %}
export interface {{ type_name }} {
  ...
}
```

## Troubleshooting

### Template Not Found

If you see "Template not found" errors:

1. Verify the template file exists in `.vika/templates/`
2. Check the filename matches exactly (case-sensitive)
3. Ensure the file has `.tera` extension

### Template Syntax Errors

If templates fail to parse:

1. Check Tera syntax documentation: https://keats.github.io/tera/docs/
2. Validate template syntax with a simple test
3. Check for unclosed `{% %}` or `{{ }}` blocks

### Template Not Applied

If your custom template isn't being used:

1. Run `vika-cli templates list` to verify override status
2. Check template file permissions
3. Ensure template filename matches exactly

## Template Variables Reference

### Common Variables

All templates support:
- String interpolation: `{{ variable_name }}`
- Conditionals: `{% if condition %}...{% endif %}`
- Loops: `{% for item in items %}...{% endfor %}`
- Filters: `{{ value | filter }}`

### Tera Built-in Filters

- `upper` - Convert to uppercase
- `lower` - Convert to lowercase
- `trim` - Remove whitespace
- `replace` - Replace substring
- `length` - Get string/array length

See [Tera documentation](https://keats.github.io/tera/docs/) for full filter list.

## Future Enhancements

Planned features:

- Template inheritance
- Template includes/partials
- Custom template variables
- Template validation
- Template preview mode

## Contributing Templates

If you create useful templates, consider contributing them back to the project! Templates that provide value to the community may be included as built-in alternatives.

## See Also

- [Tera Template Documentation](https://keats.github.io/tera/docs/)
- [Configuration Guide](./configuration.md)
- [Architecture Documentation](./architecture.md)
