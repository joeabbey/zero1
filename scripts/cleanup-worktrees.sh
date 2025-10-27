#!/bin/bash
# cleanup-worktrees.sh
# Usage: ./scripts/cleanup-worktrees.sh <base-name> <task1> <task2> ...

set -e

if [ $# -lt 2 ]; then
  echo "Usage: $0 <base-name> <task1> <task2> ..."
  echo "Example: $0 m5-phase1 stdlib-fs stdlib-crypto stdlib-env"
  exit 1
fi

BASE_NAME=$1
shift
TASKS=("$@")

echo "Cleaning up ${#TASKS[@]} worktrees..."

for TASK in "${TASKS[@]}"; do
  BRANCH="feature/$BASE_NAME-$TASK"
  WORKTREE="../${BASE_NAME}-${TASK}"

  echo ""
  echo "Removing worktree: $WORKTREE"
  git worktree remove "$WORKTREE" 2>/dev/null || echo "Worktree already removed"

  echo "Deleting branch: $BRANCH"
  git branch -d "$BRANCH" 2>/dev/null || echo "Branch already deleted"

  echo "✓ Cleaned up $TASK"
done

echo ""
echo "All worktrees cleaned up!"
