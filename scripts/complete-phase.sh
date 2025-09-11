#!/bin/bash
# scripts/complete-phase.sh - Phase Orchestrator: Complete and prepare phase for PR

set -euo pipefail

PHASE=$1
if [[ -z "$PHASE" ]]; then
    echo "Usage: ./scripts/complete-phase.sh 2A"
    echo "Available phases: 2A, 2B, 2C, 2D, 3B, 4A, 4B"
    exit 1
fi

# Ensure we're in the right directory
if [[ ! -f "Cargo.toml" ]]; then
    echo "❌ Must run from dbfast project root (Cargo.toml not found)"
    exit 1
fi

# Validate phase
declare -A PHASE_NAMES=(
    ["2A"]="SQL File Execution"
    ["2B"]="Template Creation"
    ["2C"]="Database Cloning"
    ["2D"]="Auto-Rebuild Integration"
    ["3B"]="Environment Commands"
    ["4A"]="Remote Configuration"
    ["4B"]="Backup Management"
)

if [[ -z "${PHASE_NAMES[$PHASE]:-}" ]]; then
    echo "❌ Unknown phase: $PHASE"
    exit 1
fi

echo "🔍 Preparing Phase $PHASE (${PHASE_NAMES[$PHASE]}) for completion..."

# Check we're on the right branch
CURRENT_BRANCH=$(git branch --show-current)
EXPECTED_PATTERN="phase-$(echo $PHASE | tr '[:upper:]' '[:lower:]')"

if [[ ! "$CURRENT_BRANCH" =~ $EXPECTED_PATTERN ]]; then
    echo "⚠️  Current branch: $CURRENT_BRANCH"
    echo "⚠️  Expected pattern: *$EXPECTED_PATTERN*"
    read -p "Continue anyway? (y/N): " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        exit 1
    fi
fi

# Check for uncommitted changes
if ! git diff-index --quiet HEAD --; then
    echo "❌ You have uncommitted changes. Please commit them first."
    git status --short
    exit 1
fi

# Run all quality checks
echo "⚙️  Running comprehensive quality checks..."
if ! make prepare-pr 2>/dev/null; then
    echo "❌ Quality checks failed. Please fix the following issues:"
    echo ""
    echo "🧪 Testing issues:"
    cargo test --all 2>&1 | head -20 || true
    echo ""
    echo "🔍 Linting issues:"
    cargo clippy --all-targets --all-features -- -D warnings 2>&1 | head -10 || true
    echo ""
    echo "💫 Formatting issues:"
    cargo fmt --all -- --check 2>&1 | head -10 || true
    echo ""
    echo "Fix these issues and run again."
    exit 1
fi

# Check for TDD commits that might need cleanup
echo "🔍 Checking commit history..."
TDD_COMMITS=$(git log --grep="🔴\|🟢" --oneline dev..HEAD 2>/dev/null || true)
TOTAL_COMMITS=$(git rev-list --count dev..HEAD 2>/dev/null || echo "0")

if [[ -n "$TDD_COMMITS" ]]; then
    echo "⚠️  Found TDD commits that could be squashed:"
    echo "$TDD_COMMITS"
    echo ""
    echo "💡 Consider cleaning up with: git rebase -i dev"
    echo "   (Combine related 🔴 RED and 🟢 GREEN commits)"
    echo ""
    read -p "Continue with current commit history? (y/N): " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        echo "📝 Clean up commits and run this script again"
        exit 1
    fi
fi

# Verify phase plan exists
PHASE_FILE="PHASING/PHASE_$PHASE.md"
if [[ ! -f "$PHASE_FILE" ]]; then
    echo "❌ Phase plan missing: $PHASE_FILE"
    exit 1
fi

# Get phase title for PR
PHASE_TITLE=$(head -1 "$PHASE_FILE" | sed 's/^#[[:space:]]*//' || echo "${PHASE_NAMES[$PHASE]}")

# Show commit summary
echo "📊 Phase Summary:"
echo "   Commits: $TOTAL_COMMITS"
echo "   Branch: $CURRENT_BRANCH"
echo "   Title: $PHASE_TITLE"
echo ""

# Check if gh CLI is available
if ! command -v gh &> /dev/null; then
    echo "❌ GitHub CLI (gh) not found. Please install it first:"
    echo "   https://cli.github.com/"
    exit 1
fi

# Check if authenticated with GitHub
if ! gh auth status &>/dev/null; then
    echo "❌ Not authenticated with GitHub. Run:"
    echo "   gh auth login"
    exit 1
fi

# Final confirmation
echo "🚀 Ready to create PR for Phase $PHASE"
read -p "Create pull request? (y/N): " -n 1 -r
echo
if [[ ! $REPLY =~ ^[Yy]$ ]]; then
    echo "❌ Aborted"
    exit 1
fi

# Create PR
echo "📝 Creating pull request..."

# Create PR body with phase plan
PR_BODY=$(cat "$PHASE_FILE")

if gh pr create \
    --title "Phase $PHASE: $PHASE_TITLE" \
    --body "$PR_BODY" \
    --base dev \
    --assignee @me \
    --label "phase-$PHASE" \
    --label "enhancement"; then

    echo "✅ Phase $PHASE PR created successfully!"
    echo ""
    echo "⏳ Next steps:"
    echo "   1. Wait for CI checks to pass"
    echo "   2. Review and address any feedback"
    echo "   3. Merge when approved (squash merge recommended)"
    echo "   4. Start next phase after merge"
    echo ""

    # Show next phase recommendation
    case $PHASE in
        "2A") echo "   💡 Next phase: ./scripts/run-phase.sh 2B" ;;
        "2B") echo "   💡 Next phase: ./scripts/run-phase.sh 2C" ;;
        "2C") echo "   💡 Next phase: ./scripts/run-phase.sh 2D" ;;
        "2D") echo "   💡 Next phase: ./scripts/run-phase.sh 3B" ;;
        "3B") echo "   💡 Next phase: ./scripts/run-phase.sh 4A" ;;
        "4A") echo "   💡 Next phase: ./scripts/run-phase.sh 4B" ;;
        "4B") echo "   🎉 Final phase complete! Ready for release preparation." ;;
    esac

    # Show PR URL
    PR_URL=$(gh pr view --json url --jq .url 2>/dev/null || echo "")
    if [[ -n "$PR_URL" ]]; then
        echo "   🔗 PR URL: $PR_URL"
    fi

else
    echo "❌ Failed to create PR"
    echo "💡 You can create it manually with:"
    echo "   gh pr create --title 'Phase $PHASE: $PHASE_TITLE' --base dev"
    exit 1
fi
