# Release Process

CLOBster uses `cargo-release` and `git-cliff` for automated releases.

## Version Scheme

We follow [Semantic Versioning](https://semver.org/):

- **MAJOR**: Breaking API changes
- **MINOR**: New features, backward compatible
- **PATCH**: Bug fixes, backward compatible

## Release Workflow

### 1. Prepare Release

Ensure all changes are merged to `main` and CI passes:

```bash
git checkout main
git pull origin main
cargo test
cargo clippy
```

### 2. Update Changelog

Generate changelog with `git-cliff`:

```bash
git cliff --unreleased --prepend CHANGELOG.md
```

Review and edit if needed.

### 3. Create Release

Use `cargo-release` to automate the release:

```bash
# Dry run first
cargo release patch --dry-run

# Or for minor/major
cargo release minor --dry-run
cargo release major --dry-run

# Execute release
cargo release patch
```

This will:
1. Update version in `Cargo.toml`
2. Create a signed commit
3. Create a signed tag `v0.1.1`
4. Push to GitHub

### 4. GitHub Release

After pushing the tag, create a GitHub release:

1. Go to **Releases** → **Draft a new release**
2. Select the new tag
3. Copy changelog entries for this version
4. Publish release

## Release Configuration

### cargo-release (Cargo.toml)

```toml
[package.metadata.release]
publish = false              # Don't publish to crates.io
push = true                  # Push commits and tags
tag = true                   # Create git tags
tag-name = "v{{version}}"
tag-message = "Release v{{version}}"
sign-tag = true              # GPG sign tags
sign-commit = true           # GPG sign commits
allow-branch = ["main"]
pre-release-commit-message = "chore: release v{{version}}"
```

### git-cliff (cliff.toml)

```toml
[changelog]
header = "# Changelog"
body = """
{% for group, commits in commits | group_by(attribute="group") %}
### {{ group | upper_first }}
{% for commit in commits %}
- {{ commit.message | upper_first }}\
{% endfor %}
{% endfor %}
"""
trim = true

[git]
conventional_commits = true
commit_parsers = [
    { message = "^feat", group = "Features" },
    { message = "^fix", group = "Bug Fixes" },
    { message = "^doc", group = "Documentation" },
    { message = "^perf", group = "Performance" },
    { message = "^refactor", group = "Refactor" },
    { message = "^test", group = "Testing" },
    { message = "^chore", group = "Miscellaneous" },
]
```

## GPG Signing

Releases require GPG-signed commits and tags.

### Setup GPG

```bash
# Generate key (if needed)
gpg --full-generate-key

# List keys
gpg --list-secret-keys --keyid-format LONG

# Configure git
git config --global user.signingkey YOUR_KEY_ID
git config --global commit.gpgsign true
git config --global tag.gpgsign true
```

### Add Key to GitHub

1. Export public key:
   ```bash
   gpg --armor --export YOUR_KEY_ID
   ```
2. Go to GitHub → Settings → SSH and GPG keys → New GPG key
3. Paste the public key

## Hotfix Process

For urgent fixes to a released version:

```bash
# Create hotfix branch from tag
git checkout -b hotfix/issue-description v0.1.0

# Make fixes
git commit -m "fix: critical bug"

# Release patch
cargo release patch

# Merge back to main
git checkout main
git merge hotfix/issue-description
git push origin main
```

## Release Checklist

- [ ] All tests pass
- [ ] Changelog updated
- [ ] Version bumped in Cargo.toml
- [ ] Commit signed
- [ ] Tag created and signed
- [ ] Tag pushed to GitHub
- [ ] GitHub release created
- [ ] Documentation updated (if needed)
