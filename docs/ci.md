# Zero1 CI/CD Pipeline Documentation

This document describes the continuous integration and deployment infrastructure for the Zero1 project.

## Overview

Zero1 uses GitHub Actions for automated testing, linting, documentation generation, and releases. The CI/CD pipeline ensures code quality and consistency across all contributions.

## Workflows

### Main CI Workflow (`.github/workflows/ci.yml`)

The primary CI workflow runs on:
- Every push to `main` branch
- Every pull request targeting `main`
- Manual trigger via workflow dispatch

#### Jobs

**1. Test (`test`)**

Tests the codebase across multiple platforms and Rust versions:

- **Platforms**: Ubuntu, macOS, Windows
- **Rust versions**: stable, nightly (nightly allowed to fail)
- **Matrix strategy**: 6 combinations (3 OS × 2 Rust versions)
- **Timeout**: 30 minutes per job
- **Caching**: Cargo registry, index, and build artifacts
- **Command**: `cargo test --workspace --all-features --verbose`
- **Artifacts**: Test results uploaded for 7 days

**2. Lint (`lint`)**

Enforces code quality with Clippy:

- **Platform**: Ubuntu latest
- **Rust version**: stable
- **Timeout**: 30 minutes
- **Caching**: Cargo registry, index, and build artifacts
- **Command**: `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- **Failure**: Any Clippy warning causes job to fail

**3. Format Check (`format`)**

Verifies code formatting:

- **Platform**: Ubuntu latest
- **Rust version**: stable
- **Timeout**: 10 minutes
- **Command**: `cargo fmt --all -- --check`
- **Failure**: Any formatting deviation causes job to fail

**4. Documentation (`documentation`)**

Generates and validates documentation:

- **Platform**: Ubuntu latest
- **Rust version**: stable
- **Timeout**: 30 minutes
- **Caching**: Cargo registry, index, and build artifacts
- **Command**: `cargo doc --workspace --no-deps --all-features --verbose`
- **Environment**: `RUSTDOCFLAGS=-D warnings` (warnings treated as errors)
- **Artifacts**: Documentation uploaded for 30 days
- **GitHub Pages**: Automatically deploys to `gh-pages` branch on `main` push

**5. Examples (`examples`)**

Verifies example code compiles and parses correctly:

- **Platform**: Ubuntu latest
- **Rust version**: stable
- **Timeout**: 30 minutes
- **Caching**: Cargo registry, index, and build artifacts
- **Commands**:
  - `cargo check --examples --verbose` - Verify compilation
  - Parse all `.z1c` and `.z1r` files in `examples/` directory
  - Format-check all `.z1c` files in `examples/` directory

**6. Security Audit (`security`)**

Checks for security vulnerabilities in dependencies:

- **Platform**: Ubuntu latest
- **Rust version**: stable
- **Timeout**: 15 minutes
- **Tool**: `cargo-audit`
- **Command**: `cargo audit --deny warnings`
- **Failure mode**: `continue-on-error: true` (won't block other jobs)

**7. CI Success (`ci-success`)**

Summary job that requires all critical jobs to pass:

- **Dependencies**: test, lint, format, documentation, examples
- **Purpose**: Provides single gate for required status checks
- **Failure**: If any dependent job fails, this job fails

#### Caching Strategy

The CI pipeline uses efficient caching to reduce build times:

**Cache Keys**:
- Registry: `{os}-{rust_version}-cargo-registry-{Cargo.lock_hash}`
- Index: `{os}-{rust_version}-cargo-index-{Cargo.lock_hash}`
- Build: `{os}-{rust_version}-{job_type}-target-{Cargo.lock_hash}`

**Restore Keys** (fallback hierarchy):
- `{os}-{rust_version}-cargo-registry-` (latest for OS/Rust combo)

**Cache Locations**:
- `~/.cargo/registry` - Downloaded dependency sources
- `~/.cargo/git` - Git dependencies
- `target/` - Compiled artifacts (per-job to avoid conflicts)

### Release Workflow (`.github/workflows/release.yml`)

Automated release process triggered by version tags or manual dispatch.

#### Triggers

- **Tag push**: Any tag matching `v*.*.*` (e.g., `v0.1.0`, `v1.2.3`)
- **Manual dispatch**: Allows specifying version manually

#### Jobs

**1. Create Release (`create-release`)**

- Extracts version from tag or input
- Parses `CHANGELOG.md` for release notes (if exists)
- Creates GitHub release (draft=false)
- Marks as prerelease if version contains `alpha`, `beta`, or `rc`

**2. Build Release Binaries (`build-release`)**

Builds native binaries for multiple platforms:

| Platform | Target | Artifact Name |
|----------|--------|---------------|
| Linux x86_64 | `x86_64-unknown-linux-gnu` | `z1-linux-x86_64.tar.gz` |
| Linux ARM64 | `aarch64-unknown-linux-gnu` | `z1-linux-aarch64.tar.gz` |
| macOS x86_64 | `x86_64-apple-darwin` | `z1-macos-x86_64.tar.gz` |
| macOS ARM64 | `aarch64-apple-darwin` | `z1-macos-aarch64.tar.gz` |
| Windows x86_64 | `x86_64-pc-windows-msvc` | `z1-windows-x86_64.exe.zip` |

**Build Process**:
1. Checkout code
2. Setup Rust toolchain with target
3. Install cross-compilation tools (if needed)
4. Build: `cargo build --release --target {target} -p z1-cli`
5. Strip binary (Unix platforms)
6. Package: `.tar.gz` (Unix) or `.zip` (Windows)
7. Upload to GitHub release

**3. Publish to crates.io (`publish-crates`)**

Publishes all crates to crates.io in dependency order:

- **Requirement**: `CARGO_REGISTRY_TOKEN` secret must be set
- **Order**: Respects crate dependencies
- **Flags**: `--allow-dirty` (release tags may have uncommitted changes)
- **Error handling**: `continue-on-error: true` (won't block release)

**Publish order**:
1. `z1-ast`
2. `z1-lex`
3. `z1-parse`
4. `z1-hash`
5. `z1-fmt`
6. `z1-typeck`, `z1-effects`, `z1-ctx`
7. `z1-prov`, `z1-policy`, `z1-test`
8. `z1-codegen-ts`, `z1-codegen-wasm`
9. `z1-cli`

### Dependabot Configuration (`.github/dependabot.yml`)

Automated dependency updates:

**Cargo Dependencies**:
- **Frequency**: Weekly (Mondays, 09:00 UTC)
- **Limit**: Up to 10 open PRs
- **Labels**: `dependencies`, `rust`
- **Commit prefix**: `deps:` or `deps-dev:`
- **Review**: Assigned to `joeabbey`

**GitHub Actions**:
- **Frequency**: Weekly (Mondays, 09:00 UTC)
- **Limit**: Up to 5 open PRs
- **Labels**: `dependencies`, `ci`
- **Commit prefix**: `ci:`
- **Review**: Assigned to `joeabbey`

## Setup Instructions

### First-Time Setup

1. **Enable GitHub Actions**:
   - Go to repository Settings → Actions → General
   - Allow all actions and reusable workflows

2. **Configure Branch Protection** (recommended):
   - Go to Settings → Branches → Add rule for `main`
   - Enable "Require status checks to pass before merging"
   - Select: `CI Success` (this gates all CI jobs)
   - Enable "Require branches to be up to date before merging"

3. **Optional: Enable GitHub Pages** (for documentation):
   - Go to Settings → Pages
   - Source: Deploy from a branch
   - Branch: `gh-pages`, folder: `/` (root)
   - Wait for first push to `main` to create `gh-pages` branch

4. **Optional: Setup Releases**:
   - Add repository secret `CARGO_REGISTRY_TOKEN` (from crates.io)
   - Go to Settings → Secrets and variables → Actions → New repository secret

### Verifying CI Works

After first push to `main`:

1. Go to repository Actions tab
2. Find the "CI" workflow run
3. Verify all jobs complete successfully (green checkmarks)
4. Check artifacts are uploaded (test results, documentation)

**Expected results**:
- Test job: 6 runs (3 OS × 2 Rust versions), ~327 tests passing
- Lint job: No warnings
- Format job: No formatting issues
- Documentation job: Docs generated and uploaded
- Examples job: All examples parse and format correctly
- Security job: No vulnerabilities (or allowed to fail)

### Troubleshooting

#### Tests Fail on Specific Platform

**Symptom**: Tests pass on some platforms but fail on others.

**Common causes**:
- Path separator differences (Windows uses `\`, Unix uses `/`)
- Line ending differences (CRLF vs LF)
- File permissions (Unix-specific)

**Solutions**:
- Use `std::path::PathBuf` and `Path::join()` for paths
- Configure `.gitattributes` for consistent line endings
- Avoid platform-specific assumptions

#### Cache Not Working

**Symptom**: CI runs take too long, rebuilding everything each time.

**Check**:
1. Verify `Cargo.lock` is committed to repository
2. Check cache key includes `hashFiles('**/Cargo.lock')`
3. Review cache hit/miss in job logs

**Solutions**:
- Ensure `Cargo.lock` is not in `.gitignore`
- Check cache storage limits (10 GB per repository)

#### Documentation Deployment Fails

**Symptom**: GitHub Pages not updating or 404 errors.

**Check**:
1. Verify `gh-pages` branch exists
2. Check Pages settings point to `gh-pages` branch
3. Review documentation job logs for errors

**Solutions**:
- Manually create `gh-pages` branch if needed
- Ensure `GITHUB_TOKEN` has workflow permissions
- Check for `rustdoc` warnings (set `RUSTDOCFLAGS=-D warnings`)

#### Release Build Fails

**Symptom**: Release workflow fails during binary build.

**Common causes**:
- Cross-compilation toolchain not installed
- Target not supported by dependencies
- Platform-specific code issues

**Solutions**:
- Review build logs for specific errors
- Test cross-compilation locally: `cargo build --target {target}`
- Add platform-specific conditional compilation if needed

#### Dependabot PRs Conflict

**Symptom**: Multiple Dependabot PRs updating the same dependencies.

**Solutions**:
- Configure `open-pull-requests-limit` lower (currently 10)
- Merge PRs more frequently
- Use `@dependabot squash and merge` comment

## CI Badge

Add to `README.md`:

```markdown
[![CI](https://github.com/joeabbey/zero1/workflows/CI/badge.svg)](https://github.com/joeabbey/zero1/actions)
```

This displays the current CI status and links to workflow runs.

## Local Development

### Running CI Checks Locally

Before pushing, run the same checks locally:

```bash
# Format check
cargo fmt --all -- --check

# Lint check
cargo clippy --workspace --all-targets --all-features -- -D warnings

# Run tests
cargo test --workspace --all-features

# Build documentation
cargo doc --workspace --no-deps --all-features

# Check examples
cargo check --examples
```

### Reproducing CI Environment

For debugging CI-specific issues:

```bash
# Use same Rust version as CI
rustup install stable
rustup default stable

# Clear caches (simulates fresh CI environment)
cargo clean
rm -rf ~/.cargo/registry/cache
rm -rf ~/.cargo/git/db

# Run tests
cargo test --workspace --all-features --verbose
```

## Performance Optimization

### Reducing CI Time

Current optimizations:
- **Caching**: Reduces dependency download/build time by ~70%
- **Parallel jobs**: Test matrix runs in parallel
- **Incremental compilation**: Enabled by default
- **Shared cache keys**: Multiple jobs share cache when possible

**Typical CI times** (with warm cache):
- Test (per platform): 5-10 minutes
- Lint: 3-5 minutes
- Format: <1 minute
- Documentation: 5-8 minutes
- Examples: 3-5 minutes
- Security: 1-2 minutes

**Total**: ~10-15 minutes (jobs run in parallel)

### Future Optimizations

Potential improvements:
- **sccache**: Distributed compilation caching
- **nextest**: Faster test runner
- **cargo-hakari**: Workspace hack for dependency unification
- **Conditional jobs**: Skip jobs based on changed files

## Security Considerations

### Secrets Management

Never commit secrets to repository. Use GitHub Secrets:

- `CARGO_REGISTRY_TOKEN` - For publishing to crates.io
- `GITHUB_TOKEN` - Automatically provided by GitHub Actions

### Dependency Auditing

Security audit runs weekly via Dependabot and on every CI run:

- `cargo audit` checks for known vulnerabilities
- Dependabot creates PRs for vulnerable dependencies
- Both tools reference RustSec Advisory Database

### Supply Chain Security

- All actions pinned to major versions (`@v3`, `@v4`)
- Official actions used where possible (`actions/*`, `dtolnay/*`)
- Third-party actions reviewed before use

## Monitoring and Notifications

### Status Checks

Branch protection can require:
- `CI Success` job to pass (gates all other jobs)
- Specific jobs if fine-grained control needed

### Notifications

Configure in repository Settings → Notifications:
- Email on workflow failures
- Slack/Discord webhooks for team notifications
- GitHub mobile app push notifications

## Maintenance

### Regular Tasks

**Weekly**:
- Review Dependabot PRs
- Check for failed workflow runs
- Monitor cache hit rates

**Monthly**:
- Update action versions (Dependabot handles this)
- Review security audit results
- Clean up old artifacts (auto-deleted after retention period)

**Per Release**:
- Tag release: `git tag v0.1.0 && git push origin v0.1.0`
- Verify release workflow completes
- Test downloaded binaries on each platform
- Update `CHANGELOG.md` for next release

## References

- [GitHub Actions Documentation](https://docs.github.com/en/actions)
- [Rust CI Best Practices](https://matklad.github.io/2021/09/04/fast-rust-builds.html)
- [cargo-audit](https://github.com/rustsec/rustsec/tree/main/cargo-audit)
- [dtolnay/rust-toolchain](https://github.com/dtolnay/rust-toolchain)
- [actions/cache](https://github.com/actions/cache)

## Questions and Support

For CI/CD issues:
1. Check workflow run logs in Actions tab
2. Search existing GitHub issues
3. Create new issue with "ci" label
4. Include workflow run link and error logs
