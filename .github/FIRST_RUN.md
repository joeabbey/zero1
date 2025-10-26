# First-Time CI/CD Setup and Verification

This document guides you through setting up and verifying the Zero1 CI/CD pipeline for the first time.

## Prerequisites

Before pushing these changes, ensure:

1. **Repository is on GitHub**: The workflows require GitHub Actions
2. **Git remote is configured**: `git remote -v` should show your GitHub repository
3. **You have push access**: Able to push to `main` branch

## Setup Steps

### 1. Push CI/CD Configuration

```bash
# Push the CI/CD configuration to GitHub
git push origin main
```

This will trigger the first CI workflow run automatically.

### 2. Enable GitHub Actions (if needed)

If Actions are not enabled:

1. Go to your repository on GitHub
2. Click **Settings** → **Actions** → **General**
3. Under "Actions permissions", select:
   - **Allow all actions and reusable workflows**
4. Click **Save**

### 3. Configure Branch Protection (Recommended)

Protect the `main` branch to require CI checks:

1. Go to **Settings** → **Branches**
2. Click **Add rule** for `main` branch
3. Enable:
   - ✓ Require status checks to pass before merging
   - ✓ Require branches to be up to date before merging
4. Select status checks (after first run):
   - ✓ `CI Success` (this gates all CI jobs)
5. Click **Create** or **Save changes**

### 4. Enable GitHub Pages (Optional)

To auto-deploy documentation:

1. Go to **Settings** → **Pages**
2. Under "Source", select:
   - **Deploy from a branch**
3. Under "Branch", select:
   - **gh-pages** (will be created after first successful CI run)
   - **/ (root)**
4. Click **Save**

**Note**: The `gh-pages` branch will be created automatically by the first successful CI run that pushes to `main`.

### 5. Add Repository Secrets (Optional - for Releases)

For automated releases and crates.io publishing:

1. Go to **Settings** → **Secrets and variables** → **Actions**
2. Click **New repository secret**
3. Add `CARGO_REGISTRY_TOKEN`:
   - Get token from [crates.io/me](https://crates.io/me)
   - Name: `CARGO_REGISTRY_TOKEN`
   - Value: Your crates.io API token
4. Click **Add secret**

**Note**: This is only needed if you want to publish to crates.io automatically on releases.

## Verification

### Check First CI Run

1. Go to the **Actions** tab in your GitHub repository
2. Find the "CI" workflow run (should start automatically after push)
3. Click on the run to see details
4. Verify all jobs complete successfully:

   - ✓ **Test** (6 runs: Ubuntu/macOS/Windows × stable/nightly)
   - ✓ **Lint** (Clippy checks)
   - ✓ **Format** (rustfmt checks)
   - ✓ **Documentation** (rustdoc generation)
   - ✓ **Examples** (example compilation and parsing)
   - ✓ **Security** (cargo-audit, may fail - this is OK)
   - ✓ **CI Success** (summary gate)

### Expected First Run Results

**Success indicators**:
- All test jobs pass (327 tests across workspace)
- Clippy reports no warnings
- Format check passes
- Documentation builds without errors
- Examples parse and format correctly

**Acceptable warnings**:
- Security audit may report warnings (won't block CI)
- Nightly Rust may have different results (won't block CI)

**Typical first run time**:
- **Cold cache**: 20-30 minutes (downloading and compiling all dependencies)
- **Warm cache**: 10-15 minutes (subsequent runs)

### Verify Artifacts

After successful run:

1. Click on a completed workflow run
2. Scroll to **Artifacts** section at bottom
3. Verify presence of:
   - `test-results-{os}-{rust}` (test outputs)
   - `documentation` (generated docs)

### Verify GitHub Pages (if enabled)

After first successful run to `main`:

1. Wait 1-2 minutes for Pages deployment
2. Visit: `https://[your-username].github.io/zero1/`
3. Verify documentation is visible and navigable

## Troubleshooting First Run

### Workflow Not Starting

**Symptom**: No workflow run appears in Actions tab.

**Solutions**:
1. Check Actions are enabled (Settings → Actions → General)
2. Verify `.github/workflows/ci.yml` exists in repository
3. Check branch name matches trigger (`main`)
4. Look in Actions tab for disabled workflows

### Tests Fail

**Symptom**: Test jobs show red X.

**Common causes**:
1. **Platform-specific failures**: Check if only certain OS fails
2. **Dependency issues**: Check cargo output for missing deps
3. **Flaky tests**: Re-run workflow to verify

**Solutions**:
```bash
# Run tests locally to reproduce
cargo test --workspace --all-features --verbose

# Check specific platform
# (Use GitHub Actions logs to identify failing tests)
cargo test --workspace --all-features -- --nocapture [test-name]
```

### Clippy Fails

**Symptom**: Lint job shows errors.

**Solutions**:
```bash
# Run clippy locally
cargo clippy --workspace --all-targets --all-features -- -D warnings

# Fix warnings
cargo clippy --workspace --all-targets --all-features --fix

# Commit fixes
git add .
git commit -m "fix: resolve clippy warnings"
git push origin main
```

### Format Check Fails

**Symptom**: Format job shows differences.

**Solutions**:
```bash
# Check formatting locally
cargo fmt --all -- --check

# Fix formatting
cargo fmt --all

# Commit fixes
git add .
git commit -m "style: apply rustfmt"
git push origin main
```

### Documentation Build Fails

**Symptom**: Documentation job shows errors.

**Common causes**:
1. Broken doc links
2. Invalid doc comments
3. Missing `#[doc(hidden)]` for internal items

**Solutions**:
```bash
# Run locally with warnings as errors
RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps --all-features

# Fix warnings in source code doc comments
# Commit and push fixes
```

### Examples Fail to Parse

**Symptom**: Examples job shows parsing errors.

**Solutions**:
```bash
# Test parsing locally
find examples -name "*.z1c" -o -name "*.z1r" | while read -r file; do
  echo "Parsing: $file"
  cargo run -p z1-cli -- hash "$file"
done

# Fix any invalid Z1 syntax in examples
# Commit and push fixes
```

### Cache Issues

**Symptom**: Jobs take too long or fail with cache errors.

**Solutions**:
1. Wait for cache to populate (first run is always slow)
2. Check GitHub cache limits (10 GB per repository)
3. Manually clear cache:
   - Go to **Actions** → **Caches**
   - Delete old caches if needed

### Permission Errors

**Symptom**: Jobs fail with permission denied errors.

**Common causes**:
1. `GITHUB_TOKEN` permissions too restrictive
2. Branch protection rules blocking workflow

**Solutions**:
1. Go to **Settings** → **Actions** → **General**
2. Under "Workflow permissions", select:
   - **Read and write permissions**
3. Enable **Allow GitHub Actions to create and approve pull requests**
4. Click **Save**

## Testing Release Workflow

To test the release workflow without creating a real release:

### Option 1: Manual Dispatch

1. Go to **Actions** → **Release** workflow
2. Click **Run workflow**
3. Enter version: `v0.0.0-test`
4. Click **Run workflow**
5. Monitor execution
6. Delete test release from **Releases** page after verification

### Option 2: Test Tag

```bash
# Create and push test tag
git tag v0.0.0-test
git push origin v0.0.0-test

# Monitor release workflow in Actions tab

# Delete test tag and release after verification
git tag -d v0.0.0-test
git push origin :refs/tags/v0.0.0-test
# Also delete release from GitHub UI
```

## Next Steps

After successful first run:

1. **Configure branch protection** (if not already done)
2. **Add CI badge** to README (already included in this setup)
3. **Setup notifications**:
   - Settings → Notifications
   - Configure email/Slack/Discord for workflow failures
4. **Review Dependabot PRs**:
   - Should start appearing within a week
   - Review and merge dependency updates
5. **Plan first release**:
   - Update version in `Cargo.toml` files
   - Add entry to `CHANGELOG.md` (create if needed)
   - Tag release: `git tag v0.1.0 && git push origin v0.1.0`

## Ongoing Maintenance

### Weekly
- Review Dependabot PRs
- Check for failed workflow runs
- Monitor cache hit rates in workflow logs

### Per Release
- Tag version: `git tag vX.Y.Z`
- Push tag: `git push origin vX.Y.Z`
- Verify release workflow completes
- Test downloaded binaries
- Announce release

### As Needed
- Update action versions (Dependabot handles this)
- Adjust cache keys if dependencies change frequently
- Add new jobs for new checks

## Getting Help

If issues persist:

1. **Check workflow logs**: Detailed error messages in Actions tab
2. **Search GitHub issues**: Others may have similar problems
3. **Consult documentation**: See `docs/ci.md` for details
4. **Create issue**: Include workflow run link and error logs

## Success Checklist

After first run, verify:

- [ ] All CI jobs pass (green checkmarks)
- [ ] Documentation artifact uploaded
- [ ] Test result artifacts uploaded
- [ ] GitHub Pages deployed (if enabled)
- [ ] CI badge shows "passing" in README
- [ ] Branch protection configured
- [ ] Dependabot PRs appearing (within a week)
- [ ] Release workflow tested (optional)

## Reference

- **CI Workflow**: `.github/workflows/ci.yml`
- **Release Workflow**: `.github/workflows/release.yml`
- **Dependabot Config**: `.github/dependabot.yml`
- **Full Documentation**: `docs/ci.md`
- **Actions Dashboard**: `https://github.com/[username]/zero1/actions`
