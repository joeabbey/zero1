#!/bin/bash
# integrate-worktrees.sh
# Usage: ./integrate-worktrees.sh <base-name> <task1> <task2> ...

set -e

if [ $# -lt 2 ]; then
  echo "Usage: $0 <base-name> <task1> <task2> ..."
  echo "Example: $0 m4-phase1 stdlib-fs stdlib-crypto stdlib-env"
  exit 1
fi

BASE_NAME=$1
shift
TASKS=("$@")

echo "Integrating ${#TASKS[@]} feature branches..."

# Switch to main branch
git checkout main

for TASK in "${TASKS[@]}"; do
  BRANCH="feature/$TASK"

  echo ""
  echo "Integrating $TASK..."

  # Merge feature branch
  git merge --no-ff "$BRANCH" -m "feat: merge $TASK from parallel development"

  if [ $? -ne 0 ]; then
    echo "⚠️  Merge conflict in $TASK - resolve manually"
    exit 1
  fi

  echo "✓ Merged $BRANCH"
done

echo ""
echo "Running final verification..."
cargo test --workspace
cargo clippy --workspace --all-targets --all-features -- -D warnings

if [ $? -eq 0 ]; then
  echo ""
  echo "✓ All integrations successful!"
  echo ""
  echo "Cleanup worktrees with:"
  echo "  ./cleanup-worktrees.sh $BASE_NAME ${TASKS[*]}"
else
  echo "⚠️  Tests or clippy failed - fix before cleanup"
  exit 1
fi
