#!/bin/bash
# setup-parallel-worktrees.sh
# Usage: ./scripts/setup-parallel-worktrees.sh <base-name> <task1> <task2> ...
# Example: ./scripts/setup-parallel-worktrees.sh m5 feature-a feature-b feature-c

set -e

if [ $# -lt 2 ]; then
  echo "Usage: $0 <base-name> <task1> <task2> ..."
  echo "Example: $0 m5-phase1 stdlib-fs stdlib-crypto stdlib-env"
  exit 1
fi

BASE_NAME=$1
shift
TASKS=("$@")

echo "Setting up ${#TASKS[@]} parallel worktrees for $BASE_NAME..."

for TASK in "${TASKS[@]}"; do
  BRANCH="feature/$BASE_NAME-$TASK"
  WORKTREE="../${BASE_NAME}-${TASK}"

  echo ""
  echo "Creating worktree for $TASK:"
  echo "  Branch: $BRANCH"
  echo "  Path: $WORKTREE"

  # Create feature branch from current HEAD
  git branch "$BRANCH"

  # Create worktree
  git worktree add "$WORKTREE" "$BRANCH"

  echo "✓ Ready: cd $WORKTREE && claude"
done

echo ""
echo "All worktrees created! Next steps:"
echo ""
echo "1. Launch agents in separate terminals:"
for TASK in "${TASKS[@]}"; do
  WORKTREE="../${BASE_NAME}-${TASK}"
  echo "   cd $WORKTREE && claude"
done
echo ""
echo "2. After all agents complete, integrate:"
echo "   cd $(basename $(pwd))"
echo "   ./scripts/integrate-worktrees.sh $BASE_NAME ${TASKS[*]}"
