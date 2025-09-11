#!/bin/bash
# scripts/phase-status.sh - Phase Orchestrator: Show development progress across all phases

set -euo pipefail

# Ensure we're in the right directory
if [[ ! -f "Cargo.toml" ]]; then
    echo "❌ Must run from dbfast project root (Cargo.toml not found)"
    exit 1
fi

echo "📊 DBFast Development Phase Status"
echo "=================================="
echo ""

# Check current branch and status
CURRENT_BRANCH=$(git branch --show-current)
echo "🌿 Current branch: $CURRENT_BRANCH"

# Check for uncommitted changes
if ! git diff-index --quiet HEAD --; then
    echo "⚠️  Uncommitted changes detected:"
    git status --short | head -5
    if [[ $(git diff-index --name-only HEAD -- | wc -l) -gt 5 ]]; then
        echo "   ... and $(($(git diff-index --name-only HEAD -- | wc -l) - 5)) more files"
    fi
    echo ""
fi

# Show recent activity
echo "📈 Recent Activity (last 5 commits):"
git log --oneline -5 --pretty=format:"   %C(yellow)%h%C(reset) %C(green)%cr%C(reset) %s" || echo "   No commits found"
echo ""
echo ""

# Phase definitions
PHASES=("2A" "2B" "2C" "2D" "3B" "4A" "4B")
declare -A PHASE_NAMES=(
    ["2A"]="SQL File Execution"
    ["2B"]="Template Creation"
    ["2C"]="Database Cloning"
    ["2D"]="Auto-Rebuild Integration"
    ["3B"]="Environment Commands"
    ["4A"]="Remote Configuration"
    ["4B"]="Backup Management"
)

declare -A PHASE_DESCRIPTIONS=(
    ["2A"]="Foundation: Replace placeholder with real SQL execution"
    ["2B"]="Core: CREATE DATABASE WITH TEMPLATE functionality"
    ["2C"]="Performance: <200ms database cloning from templates"
    ["2D"]="Intelligence: Auto-rebuild templates on file changes"
    ["3B"]="CLI: Environment management commands"
    ["4A"]="Remote: Configuration and connection validation"
    ["4B"]="Safety: Backup/restore with pg_dump integration"
)

echo "🎯 Phase Progress Overview:"
echo ""

COMPLETED_COUNT=0
IN_PROGRESS_PHASE=""
NEXT_PHASE=""

for phase in "${PHASES[@]}"; do
    phase_lower=$(echo "$phase" | tr '[:upper:]' '[:lower:]')

    # Check if phase is tagged as complete
    if git tag -l | grep -q "phase-$phase_lower-complete"; then
        status="✅ Complete"
        icon="✅"
        ((COMPLETED_COUNT++))
    # Check if there's an open PR for this phase
    elif command -v gh &>/dev/null && gh pr list --state open --search "Phase $phase" --json number 2>/dev/null | grep -q "number"; then
        status="🔄 In Review"
        icon="🔄"
        if [[ -z "$IN_PROGRESS_PHASE" ]]; then
            IN_PROGRESS_PHASE="$phase (PR Review)"
        fi
    # Check if phase branch exists locally
    elif git branch -a | grep -q "phase-$phase_lower"; then
        status="🚧 In Progress"
        icon="🚧"
        if [[ -z "$IN_PROGRESS_PHASE" ]]; then
            IN_PROGRESS_PHASE="$phase (Development)"
        fi
    # Check if phase plan exists
    elif [[ -f "PHASING/PHASE_$phase.md" ]]; then
        status="📋 Planned"
        icon="📋"
        if [[ -z "$NEXT_PHASE" && $COMPLETED_COUNT -eq ${#PHASES[@]} ]]; then
            NEXT_PHASE="$phase"
        fi
    else
        status="⏳ Pending"
        icon="⏳"
        if [[ -z "$NEXT_PHASE" ]]; then
            NEXT_PHASE="$phase"
        fi
    fi

    # Format output
    printf "  %s Phase %s: %-25s %s\n" "$icon" "$phase" "${PHASE_NAMES[$phase]}" "$status"
    printf "     %s\n" "${PHASE_DESCRIPTIONS[$phase]}"
    echo ""
done

# Progress summary
TOTAL_PHASES=${#PHASES[@]}
PROGRESS_PERCENT=$(( COMPLETED_COUNT * 100 / TOTAL_PHASES ))

echo "📊 Overall Progress: $COMPLETED_COUNT/$TOTAL_PHASES phases complete ($PROGRESS_PERCENT%)"

# Progress bar
echo -n "   ["
for i in $(seq 1 $TOTAL_PHASES); do
    if [[ $i -le $COMPLETED_COUNT ]]; then
        echo -n "█"
    else
        echo -n "░"
    fi
done
echo "]"
echo ""

# Current status and next actions
if [[ -n "$IN_PROGRESS_PHASE" ]]; then
    echo "🚧 Currently Working On: $IN_PROGRESS_PHASE"
    if [[ "$CURRENT_BRANCH" =~ phase- ]]; then
        echo "   Branch: $CURRENT_BRANCH"

        # Show TDD cycle guidance
        TDD_COMMITS=$(git log --grep="🔴\|🟢" --oneline dev..HEAD 2>/dev/null | wc -l || echo "0")
        REFACTOR_COMMITS=$(git log --grep="🔧" --oneline dev..HEAD 2>/dev/null | wc -l || echo "0")

        if [[ $TDD_COMMITS -gt 0 ]]; then
            echo "   TDD Commits: $TDD_COMMITS (🔴 RED/🟢 GREEN phases)"
        fi
        if [[ $REFACTOR_COMMITS -gt 0 ]]; then
            echo "   Refactor Commits: $REFACTOR_COMMITS (🔧 REFACTOR phases)"
        fi

        echo ""
        echo "💡 TDD Workflow Commands:"
        echo "   make commit-tdd      # For RED/GREEN phases"
        echo "   make commit-clean    # For REFACTOR phases"
        echo "   make check-all       # Quality verification"

        # Extract current phase from branch name
        if [[ "$CURRENT_BRANCH" =~ phase-([0-9][a-z]?)- ]]; then
            CURRENT_PHASE_CODE=$(echo "${BASH_REMATCH[1]}" | tr '[:lower:]' '[:upper:]')
            echo "   ./scripts/complete-phase.sh $CURRENT_PHASE_CODE  # When ready for PR"
        fi
    fi
elif [[ -n "$NEXT_PHASE" ]]; then
    echo "🎯 Next Phase to Start: $NEXT_PHASE (${PHASE_NAMES[$NEXT_PHASE]})"
    echo "   ./scripts/run-phase.sh $NEXT_PHASE"

    # Check if phase plan exists
    if [[ ! -f "PHASING/PHASE_$NEXT_PHASE.md" ]]; then
        echo "   ⚠️  Phase plan missing - create PHASING/PHASE_$NEXT_PHASE.md first"
    fi
else
    echo "🎉 All phases complete! Ready for release preparation."
fi

echo ""

# Quality status
echo "🔍 Quality Status:"
if make check-all &>/dev/null; then
    echo "   ✅ All quality checks passing"
else
    echo "   ❌ Quality issues detected - run 'make check-all' for details"
fi

# Branch status
LOCAL_BRANCHES=$(git branch | grep "phase-" | wc -l)
if [[ $LOCAL_BRANCHES -gt 0 ]]; then
    echo "   🌿 $LOCAL_BRANCHES phase branches exist locally"
fi

# Remote status
if command -v gh &>/dev/null && gh auth status &>/dev/null; then
    OPEN_PRS=$(gh pr list --state open --search "Phase" --json number 2>/dev/null | jq length 2>/dev/null || echo "0")
    if [[ $OPEN_PRS -gt 0 ]]; then
        echo "   🔄 $OPEN_PRS open phase PRs"
    fi
fi

echo ""
echo "💡 Useful Commands:"
echo "   ./scripts/run-phase.sh [PHASE]     # Start new phase"
echo "   ./scripts/complete-phase.sh [PHASE] # Complete current phase"
echo "   make check-all                     # Run all quality checks"
echo "   git log --oneline -10              # Review recent commits"

if command -v gh &>/dev/null; then
    echo "   gh pr list                         # View open PRs"
fi
