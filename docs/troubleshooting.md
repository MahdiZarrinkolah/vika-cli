# Troubleshooting

Common issues and solutions when using `vika-cli`.

## Installation Issues

### "Command not found" after installation

**Problem**: `vika-cli` is not in your PATH.

**Solutions**:
- Add install directory to PATH:
  ```bash
  export PATH="$HOME/.local/bin:$PATH"  # Linux/macOS
  ```
- Restart your terminal
- Reinstall using the install script

### Installation script fails

**Problem**: Install script fails with permission or network errors.

**Solutions**:
- Check internet connection
- Verify GitHub releases are accessible
- Try manual installation from source
- Check script permissions: `chmod +x install.sh`

## Generation Issues

### "Spec path required" error

**Problem**: No spec path provided.

**Solutions**:
- Provide `--spec` flag: `vika-cli generate --spec ./swagger.yaml`
- Set `spec_path` in `.vika.json`
- Ensure `.vika.json` exists (run `vika-cli init`)

### "Failed to fetch spec" error

**Problem**: Cannot fetch remote OpenAPI spec.

**Solutions**:
- Check internet connection
- Verify URL is accessible
- Check if URL requires authentication
- Use local file instead: `vika-cli generate --spec ./local-spec.yaml`
- Try with `--cache` if you've fetched before

### Circular dependency warnings

**Problem**: Warnings about circular references in schemas.

**Solutions**:
- This is handled automatically using lazy references
- Warnings are informational, not errors
- Generated code will work correctly

### "No modules selected" error

**Problem**: No modules were selected during generation.

**Solutions**:
- Select at least one module in the interactive prompt
- Check if all modules are in `modules.ignore` in config
- Verify OpenAPI spec has tags defined

## Configuration Issues

### Invalid configuration error

**Problem**: `.vika.json` has invalid values.

**Solutions**:
- Run `vika-cli init` to regenerate config
- Check JSON syntax is valid
- Verify all required fields are present
- See [Configuration Reference](configuration.md)

### Output directory not found

**Problem**: Generated files don't appear in expected location.

**Solutions**:
- Check `rootDir` and output paths in `.vika.json`
- Ensure paths are relative, not absolute
- Verify directory permissions
- Check for typos in paths

## File Conflict Issues

### "File was modified" error

**Problem**: Generated file was modified by user, tool refuses to overwrite.

**Solutions**:
- Use `--force` to overwrite: `vika-cli generate --spec ./swagger.yaml --force`
- Use `--backup` to create backup first
- Manually restore from backup if needed
- Review changes before overwriting

### Backup not created

**Problem**: `--backup` flag doesn't create backups.

**Solutions**:
- Check write permissions in project directory
- Verify `.vika-backup/` directory can be created
- Check disk space
- Review error messages for details

## Code Generation Issues

### Generated types are incorrect

**Problem**: TypeScript types don't match OpenAPI spec.

**Solutions**:
- Verify OpenAPI spec is valid
- Check for unsupported OpenAPI features
- Report issue with spec example
- Check if schema uses advanced features not yet supported

### Missing imports in generated code

**Problem**: Generated code has missing imports.

**Solutions**:
- Ensure all required schemas are generated
- Check for circular dependencies
- Verify module structure
- Regenerate with `--force`

### Zod schemas don't validate

**Problem**: Generated Zod schemas fail validation.

**Solutions**:
- Check OpenAPI constraints are valid
- Verify Zod version compatibility
- Review generated schema code
- Report issue with example

## Performance Issues

### Generation is slow

**Problem**: Code generation takes too long.

**Solutions**:
- Use `--cache` for remote specs
- Generate only needed modules
- Check network speed for remote specs
- Consider using local spec file

### High memory usage

**Problem**: Tool uses too much memory.

**Solutions**:
- Generate modules in smaller batches
- Check for very large OpenAPI specs
- Report issue with spec size
- Consider splitting large specs

## Getting Help

If you can't resolve an issue:

1. Check this troubleshooting guide
2. Search existing [GitHub Issues](https://github.com/MahdiZarrinkolah/vika-cli/issues)
3. Open a new issue with:
   - Error message
   - Steps to reproduce
   - OpenAPI spec (if possible)
   - Configuration file (redact sensitive info)
   - System information

## Reporting Bugs

When reporting bugs, include:

- `vika-cli` version: `vika-cli --version`
- Operating system
- Rust version (if building from source)
- Error message and stack trace
- Minimal OpenAPI spec that reproduces the issue
- Configuration file (redact sensitive info)

