# Release Process

This document describes the complete release process for vika-cli, from version bumping to publishing binaries.

## Table of Contents

- [Prerequisites](#prerequisites)
- [Release Methods](#release-methods)
- [Step-by-Step Release Process](#step-by-step-release-process)
- [Post-Release Checklist](#post-release-checklist)
- [Troubleshooting](#troubleshooting)
- [Quick Reference](#quick-reference)
- [Support](#support)

## Prerequisites

### Required Tools

1. **cargo-release** (recommended):
   ```bash
   cargo install cargo-release
   ```

2. **Git** with proper authentication:
   - SSH keys configured for GitHub
   - Or GitHub CLI (`gh`) authenticated

3. **GitHub Access**:
   - Push access to repository
   - Permission to create tags and releases

### Pre-Release Checks

Before starting a release, ensure:

- [ ] All tests pass: `cargo test`
- [ ] Code is formatted: `cargo fmt --check`
- [ ] No clippy warnings: `cargo clippy --all-targets --all-features -- -D warnings`
- [ ] CHANGELOG.md is up to date with all changes
- [ ] Documentation is current
- [ ] No uncommitted changes (or use `--allow-dirty` if needed)

## Release Methods

### Method 1: cargo-release (Recommended)

**Full automation** - handles everything from version bump to tag creation.

#### Usage

```bash
# Preview what will change (always do this first!)
cargo release --dry-run patch   # or minor, major

# Execute the release
cargo release patch              # 1.0.0 -> 1.0.1
cargo release minor              # 1.0.0 -> 1.1.0
cargo release major              # 1.0.0 -> 2.0.0
```

#### What cargo-release Does

1. **Pre-release checks**:
   - Runs `cargo test --all-features`
   - Runs `cargo clippy --all-targets --all-features -- -D warnings`
   - Checks `cargo fmt --check`

2. **Version bumping**:
   - Updates `version` in `Cargo.toml`
   - Updates `CHANGELOG.md` (if configured)

3. **Committing**:
   - Creates commit with message: `chore: release vika-cli <version>`

4. **Tagging**:
   - Creates tag: `v<version>`
   - Tag message: `Release vika-cli <version>`

5. **Pushing**:
   - Pushes commits and tags to remote

6. **GitHub Actions**:
   - Automatically triggered by tag push
   - Builds binaries for all platforms
   - Creates GitHub Release

#### Advanced Options

```bash
# Skip pre-release checks (not recommended)
cargo release patch --no-verify

# Allow dirty working directory
cargo release patch --allow-dirty

# Don't push (just create tag locally)
cargo release patch --no-push

# Custom version
cargo release 1.2.3

# Preview only
cargo release --dry-run patch
```

### Method 2: Manual Process

For complete manual control over the process.

1. **Update version in Cargo.toml**:
   ```toml
   version = "1.0.1"
   ```

2. **Update CHANGELOG.md**:
   - Move items from `[Unreleased]` to new version section
   - Add actual changes
   - Add release date

3. **Commit changes**:
   ```bash
   git add Cargo.toml CHANGELOG.md
   git commit -m "chore: release v1.0.1"
   ```

4. **Create and push tag**:
   ```bash
   git tag -a v1.0.1 -m "Release v1.0.1"
   git push origin main
   git push origin v1.0.1
   ```

4. **GitHub Actions** will automatically:
   - Build binaries
   - Create GitHub Release

### Method 3: Fully Manual

For complete control.

1. **Update version in Cargo.toml**:
   ```toml
   version = "1.0.1"
   ```

2. **Update CHANGELOG.md**:
   ```markdown
   ## [1.0.1] - 2025-01-22
   
   ### Fixed
   - Fixed issue with circular dependency detection
   - Improved error messages for invalid schemas
   ```

3. **Run pre-release checks**:
   ```bash
   cargo test
   cargo clippy --all-targets --all-features -- -D warnings
   cargo fmt --check
   ```

4. **Commit and tag**:
   ```bash
   git add Cargo.toml CHANGELOG.md
   git commit -m "chore: release v1.0.1"
   git tag -a v1.0.1 -m "Release v1.0.1"
   git push origin main
   git push origin v1.0.1
   ```

## Step-by-Step Release Process

### Standard Release Workflow

1. **Prepare for release**:
   ```bash
   # Ensure you're on main branch
   git checkout main
   git pull origin main
   
   # Run all checks
   cargo test
   cargo clippy --all-targets --all-features -- -D warnings
   cargo fmt --check
   ```

2. **Determine version bump**:
   - **Patch** (1.0.0 → 1.0.1): Bug fixes, small improvements
   - **Minor** (1.0.0 → 1.1.0): New features, backward-compatible
   - **Major** (1.0.0 → 2.0.0): Breaking changes

3. **Update CHANGELOG.md** (if using manual method):
   - Move items from `[Unreleased]` to new version section
   - Ensure all changes are documented
   - Add release date

4. **Execute release**:
   ```bash
   # Using cargo-release (recommended)
   cargo release --dry-run patch  # Preview first!
   cargo release patch
   ```

5. **Verify release**:
   - Check GitHub Actions workflow completed
   - Verify binaries are uploaded
   - Check GitHub Release was created
   - Test installation from release

6. **Post-release tasks** (see below)

### Pre-Release Checklist

- [ ] All features are complete and tested
- [ ] All tests pass (`cargo test`)
- [ ] No clippy warnings (`cargo clippy --all-targets --all-features -- -D warnings`)
- [ ] Code is formatted (`cargo fmt --check`)
- [ ] CHANGELOG.md is updated with all changes
- [ ] README.md is up to date
- [ ] Documentation is current
- [ ] Version number is correct
- [ ] Git working directory is clean (or use `--allow-dirty`)

### Version Bump Guidelines

#### Patch Release (1.0.0 → 1.0.1)

Use for:
- Bug fixes
- Performance improvements
- Documentation updates
- Dependency updates
- Small internal improvements

Example:
```bash
cargo release patch
```

#### Minor Release (1.0.0 → 1.1.0)

Use for:
- New features
- New generation options
- Backward-compatible API additions
- New configuration options

Example:
```bash
cargo release minor
```

#### Major Release (1.0.0 → 2.0.0)

Use for:
- Breaking changes to generated code format
- CLI interface changes
- Breaking configuration changes
- Removal of features

Example:
```bash
cargo release major
```

## Post-Release Checklist

After the release is complete:

- [ ] Verify GitHub Release was created
- [ ] Check all platform binaries are present:
  - [ ] Linux (x86_64)
  - [ ] macOS Intel (x86_64)
  - [ ] macOS ARM (arm64)
  - [ ] Windows (x86_64)
- [ ] Verify checksums are included
- [ ] Test installation from release:
  ```bash
  # Test install script
  curl -fsSL https://github.com/MahdiZarrinkolah/vika-cli/releases/latest/download/install.sh | sh
  ```
- [ ] Announce release (if desired):
  - GitHub Discussions
  - Twitter/X
  - Project website
- [ ] Monitor for issues
- [ ] Update any external documentation

## Automated Release Workflow

### GitHub Actions Workflow

When you push a tag matching `v*`:

1. **Release Workflow** (`.github/workflows/release.yml`):
   - Builds binaries for all platforms
   - Generates checksums
   - Creates GitHub Release
   - Uploads all artifacts

2. **Version Bump Workflow** (`.github/workflows/version-bump.yml`):
   - Detects version changes in `Cargo.toml`
   - Automatically creates and pushes tag
   - Triggers release workflow

### Workflow Diagram

```
Developer
   │
   ├─> cargo release patch
   │   │
   │   ├─> Run tests & checks
   │   ├─> Bump version
   │   ├─> Update changelog
   │   ├─> Create commit
   │   ├─> Create tag
   │   └─> Push to GitHub
   │
   └─> GitHub Actions
       │
       ├─> Build binaries (all platforms)
       ├─> Generate checksums
       ├─> Create GitHub Release
       └─> Upload artifacts
```

## Troubleshooting

### Common Issues

#### Release Fails: Tests Don't Pass

```bash
# Fix tests first
cargo test
# Fix any failing tests
# Then retry release
cargo release patch
```

#### Release Fails: Clippy Warnings

```bash
# Fix clippy warnings
cargo clippy --all-targets --all-features -- -D warnings
# Fix warnings
# Then retry release
cargo release patch
```

#### Release Fails: Uncommitted Changes

```bash
# Option 1: Commit changes
git add .
git commit -m "fix: ..."
cargo release patch

# Option 2: Allow dirty (not recommended)
cargo release patch --allow-dirty
```

#### Tag Already Exists

```bash
# Delete local tag
git tag -d v1.0.1

# Delete remote tag
git push origin :refs/tags/v1.0.1

# Retry release
cargo release patch
```

#### GitHub Actions Fails

1. Check workflow logs in GitHub Actions
2. Common issues:
   - Build failures (check Rust version compatibility)
   - Missing secrets (check repository settings)
   - Network issues (retry workflow)

#### Version Mismatch

Ensure `Cargo.toml` version matches Git tag:
- `Cargo.toml`: `version = "1.0.1"`
- Git tag: `v1.0.1`

If mismatch:
```bash
# Fix version in Cargo.toml
# Then create tag manually
git tag -a v1.0.1 -m "Release v1.0.1"
git push origin v1.0.1
```

## Publishing to Crates.io

When ready to publish to crates.io:

1. **Update `.cargo-release.toml`**:
   ```toml
   publish = true
   ```

2. **Ensure you have crates.io access**:
   ```bash
   cargo login <your-token>
   ```

3. **Release**:
   ```bash
   cargo release patch
   ```

   This will:
   - Bump version
   - Create tag
   - Publish to crates.io
   - Push to GitHub

## Release Notes Template

When creating a GitHub Release, use this template:

```markdown
## Changes

See [CHANGELOG.md](https://github.com/MahdiZarrinkolah/vika-cli/blob/main/CHANGELOG.md) for details.

## Installation

### macOS / Linux
```bash
curl -fsSL https://github.com/MahdiZarrinkolah/vika-cli/releases/latest/download/install.sh | sh
```

### Windows (PowerShell)
```powershell
irm https://github.com/MahdiZarrinkolah/vika-cli/releases/latest/download/install.ps1 | iex
```

### Cargo
```bash
cargo install vika-cli
```

## What's Changed

- Feature 1
- Feature 2
- Bug fix 1
- Bug fix 2
```

## Quick Reference

### Common Commands

```bash
# Preview release
cargo release --dry-run patch

# Release patch version
cargo release patch

# Release minor version
cargo release minor

# Release major version
cargo release major

# Custom version
cargo release 1.2.3

# Skip checks (not recommended)
cargo release patch --no-verify

# Allow dirty working directory
cargo release patch --allow-dirty

# Don't push (local only)
cargo release patch --no-push
```

## Quick Reference

For a quick overview of the release process:

### Using cargo-release

```bash
# Install cargo-release
cargo install cargo-release

# Preview what will change
cargo release --dry-run patch

# Release (bumps version, updates changelog, creates tag)
cargo release patch
```

### Manual Process

1. **Update version in Cargo.toml**:
   ```toml
   version = "1.1.0"
   ```

2. **Update CHANGELOG.md**:
   - Move items from `[Unreleased]` to new version section
   - Add release date

3. **Commit and tag**:
   ```bash
   git add Cargo.toml CHANGELOG.md
   git commit -m "chore: release v1.1.0"
   git tag v1.1.0
   git push origin main --tags
   ```

### Automated Release Workflow

When you push a tag matching `v*`:
1. GitHub Actions automatically builds binaries for all platforms
2. Creates a GitHub Release with binaries and checksums
3. Release notes are generated from CHANGELOG.md

### Conventional Commits (Optional)

For even more automation, you can use conventional commits:
- `feat:` → minor version bump
- `fix:` → patch version bump  
- `BREAKING CHANGE:` → major version bump

Then use `cargo-release` with `auto-version = "auto"` to automatically determine the version bump from commit messages.

### GitHub Actions Automation

The `.github/workflows/version-bump.yml` workflow:
- Detects when version changes in Cargo.toml
- Automatically creates and pushes a Git tag
- Triggers the release workflow

This means you can just update the version and push, and the tag will be created automatically!

## Support

For issues or questions about the release process:
- Open an issue on GitHub
- Check [docs/development/release-setup.md](docs/development/release-setup.md) for detailed setup instructions
- Review [docs/development/release-quick-start.md](docs/development/release-quick-start.md) for a quick reference

