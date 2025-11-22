# Quick Start: Automated Versioning

## Fastest Way: cargo-release

```bash
# Preview what will change
cargo release --dry-run patch

# Release (bumps version, updates changelog, creates tag, pushes)
cargo release patch   # 1.0.0 -> 1.0.1
cargo release minor   # 1.0.0 -> 1.1.0
cargo release major   # 1.0.0 -> 2.0.0
```

That's it! cargo-release will:
1. ✅ Run tests and clippy
2. ✅ Bump version in Cargo.toml
3. ✅ Update CHANGELOG.md
4. ✅ Create Git tag
5. ✅ Push to remote
6. ✅ GitHub Actions builds and releases binaries

## Workflow Examples

### Simple Release
```bash
cargo release patch
```

### Manual Control
```bash
# Update version in Cargo.toml
# Update CHANGELOG.md
git add Cargo.toml CHANGELOG.md
git commit -m "chore: release v1.1.0"
git tag v1.1.0
git push origin main --tags
```

See [release-setup.md](release-setup.md) for detailed setup instructions.

