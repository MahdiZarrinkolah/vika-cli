# Template System

`vika-cli` uses templates to generate code. You can customize these templates to change the output format.

## Template Location

Templates are located in the `src/templates/` directory of the `vika-cli` source code:

- `http_client.ts`: HTTP client utility template
- `index.ts`: Barrel export template

## Template Variables

Templates support variable substitution using Rust's string formatting.

### HTTP Client Template

The HTTP client template (`http_client.ts`) generates the base HTTP utility used by all API functions.

**Variables:**
- None (static template)

**Example Output:**
```typescript
export const http = {
  get: async <T>(url: string): Promise<T> => {
    // Implementation
  },
  // ...
};
```

### Index Template

The index template (`index.ts`) generates barrel exports for modules.

**Variables:**
- None (static template)

**Example Output:**
```typescript
export * from './types';
export * from './schemas';
```

## Customizing Templates

To customize templates:

1. Fork the `vika-cli` repository
2. Modify templates in `src/templates/`
3. Rebuild the tool

**Note**: Template customization requires rebuilding from source. This is a planned feature for future releases.

## Future Enhancements

Planned features:
- Template configuration in `.vika.json`
- Custom template paths
- Template variable expansion
- Multiple template sets

## Contributing Templates

If you create useful templates, consider contributing them back to the project!

