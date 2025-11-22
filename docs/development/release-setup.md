# Setting Up Automated Versioning

This guide explains how to set up automated version bumping and changelog management.

## Recommended: cargo-release

### Setup

1. **Install cargo-release**:
   ```bash
   cargo install cargo-release
   ```

2. **Use cargo-release**:
   ```bash
   # Preview changes
   cargo release --dry-run patch
   
   # Release (bumps version, updates changelog, creates tag, pushes)
   cargo release patch  # or minor, major
   ```

3. **For automatic version detection from conventional commits**:
   Edit `.cargo-release.toml` and set:
   ```toml
   auto-version = "auto"
   ```

## Alternative: Manual Process

1. **Update version in Cargo.toml**:
   ```toml
   version = "1.0.1"
   ```

2. **Update CHANGELOG.md**:
   - Move items from `[Unreleased]` to new version section
   - Add actual changes
   - Add release date

3. **Commit and tag**:
   ```bash
   git add Cargo.toml CHANGELOG.md
   git commit -m "chore: release v1.0.1"
   git tag v1.0.1
   git push origin main --tags
   ```

## GitHub Actions (Fully Automated)

The `.github/workflows/version-bump.yml` workflow:
- Detects version changes in `Cargo.toml`
- Automatically creates and pushes Git tag
- Triggers release workflow

**Workflow**:
1. Update version in `Cargo.toml`
2. Update `CHANGELOG.md`
3. Commit and push to main
4. GitHub Actions creates tag automatically
5. Release workflow builds and publishes

## Recommended Workflow

1. **Use cargo-release** for releases:
   ```bash
   cargo release patch
   ```

2. **GitHub Actions** automatically handles tagging and releases when you push tags

