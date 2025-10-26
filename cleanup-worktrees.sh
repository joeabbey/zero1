#!/bin/bash
# cleanup-worktrees.sh
# Usage: ./cleanup-worktrees.sh <base-name> <task1> <task2> ...

set -e

if [ $# -lt 2 ]; then
  echo "Usage: $0 <base-name> <task1> <task2> ..."
  echo "Example: $0 m4-phase1 stdlib-fs stdlib-crypto stdlib-env"
  exit 1
fi

BASE_NAME=$1
shift
TASKS=("$@")

echo "Cleaning up ${#TASKS[@]} worktrees..."

for TASK in "${TASKS[@]}"; do
  BRANCH="feature/$TASK"
  WORKTREE="../${BASE_NAME}-${TASK}"

  echo ""
  echo "Removing worktree: $WORKTREE"
  git worktree remove "$WORKTREE" || true

  echo "Deleting branch: $BRANCH"
  git branch -d "$BRANCH" || true

  echo "✓ Cleaned up $TASK"
done

echo ""
echo "All worktrees cleaned up!"
