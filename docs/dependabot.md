# Dependabot Configuration

This document explains how Dependabot is configured in the Zero1 repository and how to work with dependency updates.

## Configuration Overview

Dependabot is configured in `.github/dependabot.yml` to monitor:
- **Cargo dependencies** - Rust crates (weekly on Mondays)
- **GitHub Actions** - Workflow actions (weekly on Mondays)

## Auto-Merge Policy

The repository uses GitHub's native auto-merge feature (not the deprecated `@dependabot merge` command) via the `dependabot-auto-merge.yml` workflow.

### Auto-Merge Rules

**Automatically merged:**
- Patch updates (e.g., 1.2.3 → 1.2.4)
- Minor updates (e.g., 1.2.0 → 1.3.0)

**Requires manual review:**
- Major updates (e.g., 1.0.0 → 2.0.0)
- Updates that fail CI tests

### How It Works

1. Dependabot creates a PR for dependency updates
2. CI pipeline runs automatically (tests, clippy, formatting)
3. If CI passes:
   - **Patch/Minor**: PR is auto-approved and auto-merged
   - **Major**: Comment added requesting manual review
4. If CI fails: PR remains open for investigation

## Grouping

Dependencies are grouped to reduce PR noise:

**Rust Dependencies:**
- Minor and patch updates grouped into single PR
- Major updates remain separate for careful review

**GitHub Actions:**
- All action updates grouped together
- Easier to review and test workflow changes

## Manual Operations

### Using GitHub UI

Instead of the deprecated `@dependabot` comment commands, use GitHub's native controls:

- **Merge PR**: Use the "Merge pull request" button
- **Close PR**: Use the "Close pull request" button
- **Reopen PR**: Use the "Reopen pull request" button
- **Rebase PR**: Comment `@dependabot rebase` (still works for now)

### Using GitHub CLI

```bash
# Approve PR
gh pr review <number> --approve

# Merge PR
gh pr merge <number> --squash

# Close PR
gh pr close <number>

# Reopen PR
gh pr reopen <number>
```

### Using REST API

See [GitHub's Pull Request API documentation](https://docs.github.com/en/rest/pulls).

## Deprecation Timeline

**January 27, 2026**: The following Dependabot comment commands will stop working:
- `@dependabot merge`
- `@dependabot squash and merge`
- `@dependabot rebase and merge`

**Migration**: Already complete! This repository uses GitHub's native auto-merge feature.

## Testing Dependabot PRs

When reviewing Dependabot PRs:

1. Check the CI status (all jobs must pass)
2. Review the changelog/release notes for breaking changes
3. For major updates, test locally:
   ```bash
   gh pr checkout <number>
   cargo test --workspace
   cargo clippy --workspace --all-targets --all-features
   ```
4. Merge using GitHub UI or enable auto-merge

## Troubleshooting

### Auto-merge not triggering

- Check that CI passed successfully
- Verify the update type (major updates aren't auto-merged)
- Check workflow permissions in repository settings

### Dependabot PR stuck

- Re-run failed CI jobs if transient failure
- Comment `@dependabot recreate` to rebuild the PR
- Comment `@dependabot rebase` to rebase on latest main

### Conflicts with main

Dependabot will automatically rebase when main is updated. If conflicts persist:
1. Comment `@dependabot rebase`
2. Or manually resolve conflicts and push

## Best Practices

1. **Enable auto-merge** for all repos to reduce maintenance burden
2. **Group updates** to minimize PR spam
3. **Test major updates** thoroughly before merging
4. **Review changelogs** for all dependency updates
5. **Keep CI green** so auto-merge works smoothly

## References

- [GitHub Dependabot Documentation](https://docs.github.com/en/code-security/dependabot)
- [Dependabot Configuration Options](https://docs.github.com/en/code-security/dependabot/dependabot-version-updates/configuration-options-for-the-dependabot.yml-file)
- [Auto-merge Documentation](https://docs.github.com/en/pull-requests/collaborating-with-pull-requests/incorporating-changes-from-a-pull-request/automatically-merging-a-pull-request)
- [Deprecation Announcement](https://github.blog/changelog/2025-10-06-upcoming-changes-to-github-dependabot-pull-request-comment-commands/)
