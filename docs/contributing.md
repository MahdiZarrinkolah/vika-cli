# Contributing to vika-cli

Thank you for your interest in contributing to `vika-cli`! This document provides guidelines and instructions for contributing.

## Getting Started

1. Fork the repository
2. Clone your fork: `git clone https://github.com/YOUR_USERNAME/vika.git`
3. Create a branch: `git checkout -b feature/your-feature-name`
4. Make your changes
5. Run tests: `cargo test`
6. Format code: `cargo fmt`
7. Check linting: `cargo clippy --all-targets --all-features -- -D warnings`
8. Commit your changes: `git commit -m "Add feature: your feature"`
9. Push to your fork: `git push origin feature/your-feature-name`
10. Open a Pull Request

## Development Setup

### Prerequisites

- Rust 1.70+
- Cargo

### Building

```bash
cargo build
```

### Running Tests

```bash
# All tests
cargo test

# Unit tests only
cargo test --lib

# Integration tests only
cargo test --test '*'
```

### Code Quality

We enforce code quality through CI:

- **Formatting**: `cargo fmt --check`
- **Linting**: `cargo clippy --all-targets --all-features -- -D warnings`
- **Tests**: All tests must pass

## Code Style

### Rust Style

- Follow Rust standard formatting (`cargo fmt`)
- Follow clippy suggestions
- Use meaningful variable and function names
- Add documentation comments for public APIs

### Commit Messages

Use clear, descriptive commit messages:

```
Add feature: Support for oneOf schemas

- Implement oneOf schema resolution
- Add tests for oneOf generation
- Update documentation
```

## Testing Guidelines

### Unit Tests

- Test individual functions in isolation
- Use `#[cfg(test)]` modules
- Place tests near the code they test

### Integration Tests

- Test full workflows end-to-end
- Use `tests/` directory
- Use shared fixtures from `tests/common/`

### Snapshot Tests

- Use `insta` for generated code verification
- Update snapshots: `cargo insta review`

## Pull Request Process

1. **Update Documentation**: If your change affects user-facing behavior, update the README and docs
2. **Add Tests**: Include tests for new features
3. **Update CHANGELOG**: Add an entry describing your changes
4. **Ensure CI Passes**: All CI checks must pass
5. **Request Review**: Request review from maintainers

## Areas for Contribution

### High Priority

- Additional OpenAPI features (more schema types, more HTTP methods)
- Performance improvements
- Better error messages
- Documentation improvements

### Medium Priority

- Template customization system
- Plugin architecture
- Additional output formats
- CI/CD improvements

### Low Priority

- Additional language support (beyond TypeScript)
- GUI interface
- IDE integrations

## Questions?

- Open an issue for bug reports
- Open a discussion for questions
- Contact maintainers for security issues

## Code of Conduct

Be respectful and inclusive. We welcome contributions from everyone.

## License

By contributing, you agree that your contributions will be licensed under the MIT License.

