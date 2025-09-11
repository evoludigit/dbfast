#!/bin/bash
# scripts/run-phase.sh - Phase Orchestrator: Start a new development phase

set -euo pipefail

PHASE=$1
DESCRIPTION=${2:-""}

if [[ -z "$PHASE" ]]; then
    echo "Usage: ./scripts/run-phase.sh 2A [description]"
    echo "Available phases: 2A, 2B, 2C, 2D, 3B, 4A, 4B"
    echo ""
    echo "Phase descriptions:"
    echo "  2A: SQL File Execution (Foundation)"
    echo "  2B: Template Creation (needs 2A)"
    echo "  2C: Database Cloning (needs 2B)"
    echo "  2D: Auto-Rebuild Integration (needs 2C)"
    echo "  3B: Environment Commands (needs 2D)"
    echo "  4A: Remote Configuration (needs 3B)"
    echo "  4B: Backup Management (needs 4A)"
    exit 1
fi

# Phase descriptions
declare -A PHASE_NAMES=(
    ["2A"]="sql-execution"
    ["2B"]="template-creation"
    ["2C"]="database-cloning"
    ["2D"]="auto-rebuild"
    ["3B"]="environment-commands"
    ["4A"]="remote-config"
    ["4B"]="backup-management"
)

PHASE_NAME=${PHASE_NAMES[$PHASE]}
if [[ -z "$PHASE_NAME" ]]; then
    echo "❌ Unknown phase: $PHASE"
    echo "Available phases: ${!PHASE_NAMES[@]}"
    exit 1
fi

echo "🚀 Starting Phase $PHASE: $PHASE_NAME"

# Ensure we're in the right directory
if [[ ! -f "Cargo.toml" ]]; then
    echo "❌ Must run from dbfast project root (Cargo.toml not found)"
    exit 1
fi

# Ensure we're on clean dev
echo "🔄 Switching to dev branch..."
git checkout dev
git pull origin dev

# Check for uncommitted changes
if ! git diff-index --quiet HEAD --; then
    echo "❌ You have uncommitted changes. Please commit or stash them first."
    git status --short
    exit 1
fi

# Create new branch
BRANCH_NAME="phase-$(echo $PHASE | tr '[:upper:]' '[:lower:]')-$PHASE_NAME"
echo "🌿 Creating branch: $BRANCH_NAME"

if git show-ref --verify --quiet "refs/heads/$BRANCH_NAME"; then
    echo "⚠️  Branch $BRANCH_NAME already exists"
    read -p "Switch to existing branch? (y/N): " -n 1 -r
    echo
    if [[ $REPLY =~ ^[Yy]$ ]]; then
        git checkout "$BRANCH_NAME"
    else
        echo "❌ Aborting"
        exit 1
    fi
else
    git checkout -b "$BRANCH_NAME"
fi

# Check phase plan exists
PHASE_FILE="PHASING/PHASE_$PHASE.md"
if [[ ! -f "$PHASE_FILE" ]]; then
    echo "❌ Phase plan missing: $PHASE_FILE"
    echo "📝 Please create the phase plan first with:"
    echo "   - Clear objectives"
    echo "   - TDD cycle breakdown"
    echo "   - Success criteria"
    echo "   - Integration points"
    exit 1
fi

echo "✅ Phase $PHASE environment ready"
echo ""
echo "📖 Phase Plan: $PHASE_FILE"
echo "🌿 Branch: $BRANCH_NAME"
echo ""
echo "💡 TDD Workflow Commands:"
echo "   🔴 make commit-tdd     # For RED/GREEN phases (--no-verify)"
echo "   🔧 make commit-clean   # For REFACTOR phases (full checks)"
echo "   🔍 make check-all      # Quality verification"
echo "   📊 ./scripts/phase-status.sh  # Check progress"
echo "   🚀 ./scripts/complete-phase.sh $PHASE  # When phase done"
echo ""

# Show integration point based on phase
echo "🔗 Integration Focus:"
case $PHASE in
    "2A")
        echo "   Replace placeholder implementation in src/commands/seed.rs"
        echo "   Add real SQL file execution with PostgreSQL integration"
        echo "   Build foundation for template system"
        ;;
    "2B")
        echo "   Use Phase 2A SQL execution engine for template creation"
        echo "   Implement CREATE DATABASE WITH TEMPLATE functionality"
        echo "   Add template validation and metadata"
        ;;
    "2C")
        echo "   Use Phase 2B templates for fast database cloning"
        echo "   Target <200ms clone performance"
        echo "   Support concurrent clones"
        ;;
    "2D")
        echo "   Connect existing src/scanner.rs with Phase 2B templates"
        echo "   Implement smart template rebuilding"
        echo "   Add template cache management"
        ;;
    "3B")
        echo "   Add CLI commands using existing src/environment.rs"
        echo "   Implement 'dbfast environments' and 'validate-env' commands"
        echo "   Enhance status command with environment info"
        ;;
    "4A")
        echo "   Expand existing src/remote.rs stub with real functionality"
        echo "   Add remote connection validation"
        echo "   Implement remote management CLI commands"
        ;;
    "4B")
        echo "   Complete existing src/backup.rs stub"
        echo "   Integrate pg_dump/pg_restore functionality"
        echo "   Add backup rotation and cleanup"
        ;;
esac

echo ""
echo "📋 Recommended First Steps:"
echo "   1. Review $PHASE_FILE thoroughly"
echo "   2. Write your first failing test (RED 🔴)"
echo "   3. Commit with: make commit-tdd"
echo "   4. Follow TDD cycle: RED → GREEN → REFACTOR"
echo ""
echo "🎯 Remember: Focus on small, incremental progress with platinum quality!"
echo ""

# Offer to launch Claude Code for this phase
echo "🤖 Claude Code Integration:"
read -p "Launch Claude Code for Phase $PHASE development? (Y/n): " -r
echo

if [[ ! $REPLY =~ ^[Nn]$ ]]; then
    echo "🚀 Launching Claude Code for Phase $PHASE..."

    # Create phase-specific prompt file
    PROMPT_FILE=".claude-phase-$PHASE-prompt.md"

    # Generate phase-specific prompt
    cat > "$PROMPT_FILE" << EOF
# Phase $PHASE: ${PHASE_NAMES[$PHASE]} - Development Session

## 🎯 Phase Objective
$(head -3 "$PHASE_FILE" | tail -1 | sed 's/^#* *//')

## 📋 Current Status
- **Branch**: $BRANCH_NAME
- **Phase Plan**: $PHASE_FILE
- **Integration Point**: $(case $PHASE in
    "2A") echo "Replace placeholder in src/commands/seed.rs with real SQL execution" ;;
    "2B") echo "Use Phase 2A SQL execution for template creation" ;;
    "2C") echo "Use Phase 2B templates for <200ms database cloning" ;;
    "2D") echo "Connect existing src/scanner.rs with Phase 2B templates" ;;
    "3B") echo "Add CLI commands using existing src/environment.rs" ;;
    "4A") echo "Expand existing src/remote.rs stub with real functionality" ;;
    "4B") echo "Complete existing src/backup.rs stub with pg_dump integration" ;;
esac)

## 🔄 TDD Methodology (Phase 6)
Follow the micro TDD cycles:

### RED 🔴 (Failing Test)
1. Write test that captures next small requirement
2. Ensure test fails for the right reason
3. Commit: \`make commit-tdd\` (uses --no-verify)

### GREEN 🟢 (Minimal Implementation)
1. Write minimal code to make test pass
2. Focus on making it work, not perfect
3. Commit: \`make commit-tdd\` (uses --no-verify)

### REFACTOR 🔧 (Quality Improvement)
1. Improve code quality without changing behavior
2. Remove duplication, improve naming, structure
3. Commit: \`make commit-clean\` (full quality checks)

## 🎯 Phase-Specific Guidance

$(case $PHASE in
    "2A") cat << 'PHASE_2A'
### Phase 2A: SQL File Execution Foundation

**Key Implementation Areas:**
- Replace placeholder in \`src/commands/seed.rs\`
- Add real SQL file reading and execution
- Integrate with existing PostgreSQL connection pool
- Add proper error handling and transaction management

**Critical Tests to Write:**
1. SQL file reading with error handling
2. PostgreSQL connection integration
3. Transaction management and rollback
4. Performance under typical workloads

**Success Criteria:**
- \`dbfast seed\` executes real SQL files
- Proper error handling and rollback
- Transaction safety maintained
- Performance < 5s for typical seed files
PHASE_2A
    ;;
    "2B") cat << 'PHASE_2B'
### Phase 2B: Template Creation

**Key Implementation Areas:**
- BUILD on Phase 2A SQL execution engine
- Implement CREATE DATABASE WITH TEMPLATE functionality
- Add template metadata and validation
- Ensure atomic template operations

**Critical Tests to Write:**
1. CREATE DATABASE WITH TEMPLATE syntax
2. Template metadata storage and retrieval
3. Template validation and integrity
4. Atomic template creation and cleanup

**Success Criteria:**
- Can create template databases from SQL files in <30s
- Template validation prevents corrupt templates
- Atomic operations (all-or-nothing)
- Template metadata tracking
PHASE_2B
    ;;
    "2C") cat << 'PHASE_2C'
### Phase 2C: Database Cloning Performance

**Key Implementation Areas:**
- BUILD on Phase 2B template creation
- Implement fast cloning (<200ms target)
- Support concurrent clone operations
- Add clone cleanup and naming conventions

**Critical Tests to Write:**
1. Basic template cloning functionality
2. Performance benchmarks (<200ms target)
3. Concurrent cloning support
4. Clone cleanup and error handling

**Success Criteria:**
- Consistent <200ms clone times
- Concurrent clones work reliably
- Proper cleanup on success/failure
- Clone naming follows conventions
PHASE_2C
    ;;
    "2D") cat << 'PHASE_2D'
### Phase 2D: Auto-Rebuild Integration

**Key Implementation Areas:**
- INTEGRATE existing src/scanner.rs with Phase 2B templates
- Implement smart template rebuilding logic
- Add template cache management
- Connect file changes to template invalidation

**Critical Tests to Write:**
1. File change detection integration
2. Template cache invalidation logic
3. Smart rebuild triggers (only when needed)
4. Template cache performance

**Success Criteria:**
- Templates rebuild automatically on file changes
- Smart rebuilding (only when necessary)
- Template cache improves performance
- File scanner integration seamless
PHASE_2D
    ;;
    "3B") cat << 'PHASE_3B'
### Phase 3B: Environment Commands

**Key Implementation Areas:**
- ADD CLI commands using existing src/environment.rs
- Implement \`dbfast environments\` command
- Implement \`dbfast validate-env\` command
- Enhance status with environment information

**Critical Tests to Write:**
1. \`dbfast environments\` command functionality
2. \`dbfast validate-env\` command validation
3. Enhanced status command integration
4. Environment configuration validation

**Success Criteria:**
- Environment commands work with template system
- Clear environment status and validation
- Production safety warnings implemented
- Integration with existing environment filtering
PHASE_3B
    ;;
    "4A") cat << 'PHASE_4A'
### Phase 4A: Remote Configuration

**Key Implementation Areas:**
- EXPAND existing src/remote.rs stub
- Add remote configuration parsing from dbfast.toml
- Implement connection validation and testing
- Add remote management CLI commands

**Critical Tests to Write:**
1. Remote configuration parsing
2. Connection validation and testing
3. Environment linking (remote → environment config)
4. Remote management CLI commands

**Success Criteria:**
- Can configure remote database connections
- Connection validation catches issues early
- Environment safety (staging vs production)
- Remote management through CLI
PHASE_4A
    ;;
    "4B") cat << 'PHASE_4B'
### Phase 4B: Backup Management

**Key Implementation Areas:**
- COMPLETE existing src/backup.rs stub
- Integrate pg_dump/pg_restore functionality
- Add backup metadata and storage management
- Implement backup rotation and cleanup

**Critical Tests to Write:**
1. pg_dump integration and automation
2. Backup metadata and storage
3. pg_restore and rollback functionality
4. Backup cleanup and rotation

**Success Criteria:**
- Complete backup/restore workflow works
- Backup metadata tracking reliable
- Automatic backup before deployments
- Backup rotation prevents disk overflow
PHASE_4B
    ;;
esac)

## 🔧 Available Commands
- \`make commit-tdd\` - TDD RED/GREEN commits (--no-verify)
- \`make commit-clean\` - REFACTOR commits (full quality checks)
- \`make check-all\` - Run all quality verification
- \`make phase-status\` - Check progress across all phases
- \`./scripts/complete-phase.sh $PHASE\` - Complete phase and create PR

## 📖 Reference Files
- **Phase Plan**: $PHASE_FILE
- **Integration Guide**: PHASING/ORCHESTRATOR.md
- **Quality Standards**: PHASING/PHASE_6.md

## 🚀 Getting Started
1. Review the phase plan thoroughly: $PHASE_FILE
2. Start with your first failing test (RED 🔴)
3. Follow TDD cycle: RED → GREEN → REFACTOR
4. Use \`make commit-tdd\` for RED/GREEN, \`make commit-clean\` for REFACTOR
5. Complete with \`./scripts/complete-phase.sh $PHASE\` when ready

---

**Focus on small, incremental progress with platinum quality! Each TDD cycle should be a small, focused improvement.**
EOF

    echo "📝 Created phase-specific prompt: $PROMPT_FILE"
    echo ""

    # Launch Claude Code with the phase-specific context
    if command -v claude &>/dev/null; then
        echo "🤖 Starting Claude Code session for Phase $PHASE..."
        claude --file "$PROMPT_FILE" --project-context
    elif command -v code &>/dev/null && [[ -d ~/.vscode/extensions/*claude* ]]; then
        echo "🤖 Opening VS Code with Claude extension..."
        code . "$PROMPT_FILE"
    else
        echo "⚠️  Claude Code CLI not found. Alternatives:"
        echo "   1. Install Claude Code CLI: https://claude.ai/code"
        echo "   2. Copy prompt content from: $PROMPT_FILE"
        echo "   3. Start Claude Code manually with project context"
        echo ""
        echo "📋 Phase prompt ready at: $PROMPT_FILE"
        echo "💡 You can copy this content to Claude Code when ready"
    fi
else
    echo "⏭️  Skipping Claude Code launch"
    echo "💡 You can start manually later with phase context"
fi
