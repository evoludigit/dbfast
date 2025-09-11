# Phase Orchestrator: Linear Development Pipeline

**Goal**: Execute phases 2A→2B→2C→2D→3B→4A→4B using Phase 6 methodology with seamless integration and platinum code quality.

## 🎯 Orchestration Overview

This orchestrator guides the systematic execution of all remaining phases using the Phase 6 linear pipeline methodology. Each phase builds on previous work through clean, tested interfaces.

## 📋 Phase Execution Sequence

```
Current State → 2A → 2B → 2C → 2D → 3B → 4A → 4B → Complete
              ↓    ↓    ↓    ↓    ↓    ↓    ↓
             SQL  TMPL CLONE AUTO  ENV  RMT  BKUP
```

### **Phase Dependencies Map**
```
2A: SQL File Execution (Foundation)
└── 2B: Template Creation (needs SQL execution)
    └── 2C: Database Cloning (needs templates)
        └── 2D: Auto-Rebuild (needs cloning + file scanner)
            └── 3B: Environment Commands (needs template system)
                └── 4A: Remote Config (needs environment validation)
                    └── 4B: Backup Management (needs remote connections)
```

## 🚀 Phase Launcher Protocol

### **Step 1: Initialize Phase**
```bash
# Use the phase runner script (with Claude Code integration)
./scripts/run-phase.sh 2A

# The script will:
# 1. Create and switch to phase branch
# 2. Validate phase plan exists
# 3. Generate phase-specific Claude Code prompt
# 4. Offer to launch Claude Code with full context

# Or manually:
git checkout dev && git pull origin dev
git checkout -b phase-2a-sql-execution
```

### **Step 2: Create Phase Plan**
Ensure `PHASING/PHASE_2A.md` exists with:
- Clear objectives
- TDD cycle breakdown
- Success criteria
- Integration points with previous phases

### **Step 3: Execute TDD Cycles**
Follow Phase 6 methodology:
```bash
# RED 🔴 - Failing test
git commit --no-verify -m "🔴 Add test for [specific feature]"

# GREEN 🟢 - Minimal implementation
git commit --no-verify -m "🟢 Implement [feature] - tests passing"

# REFACTOR 🔧 - Quality improvement
make check-all && git commit -m "🔧 Refactor [component] - [improvement]"
```

### **Step 4: Complete Phase**
```bash
# Prepare for PR
make prepare-pr

# Create PR
gh pr create --title "Phase 2A: SQL File Execution" --base dev

# Wait for review → merge → tag
```

### **Step 5: Transition to Next Phase**
```bash
# Update local dev
git checkout dev && git pull origin dev

# Start next phase immediately
./scripts/run-phase.sh 2B
```

## 📊 Phase-Specific Integration Points

### **Phase 2A: SQL File Execution**
**Builds On**: Current placeholder implementation
**Provides**: Real SQL file reading and execution engine
**Integration**: Replace `seed.rs` placeholder with actual PostgreSQL execution

**Key TDD Cycles**:
1. SQL file reading with error handling
2. PostgreSQL connection integration
3. Transaction management and rollback
4. Performance optimization

**Success Gate**: `dbfast seed` executes real SQL files with proper error handling

---

### **Phase 2B: Template Creation**
**Builds On**: 2A SQL execution engine
**Provides**: PostgreSQL template database creation
**Integration**: Use 2A's SQL execution to build templates

**Key TDD Cycles**:
1. CREATE DATABASE WITH TEMPLATE functionality
2. Template metadata and validation
3. Atomic template operations
4. Template naming and cleanup

**Success Gate**: Can create template databases from SQL files in <30s

---

### **Phase 2C: Database Cloning**
**Builds On**: 2B template creation
**Provides**: Fast database cloning (<200ms)
**Integration**: Clone from 2B templates using CREATE DATABASE WITH TEMPLATE

**Key TDD Cycles**:
1. Basic template cloning syntax
2. Performance optimization for <200ms target
3. Concurrent clone support
4. Clone cleanup and naming conventions

**Success Gate**: Can clone databases from templates in <200ms consistently

---

### **Phase 2D: Auto-Rebuild Integration**
**Builds On**: 2A+2B+2C + existing file scanner
**Provides**: Smart template rebuilding on file changes
**Integration**: Connect existing `scanner.rs` with 2B template creation

**Key TDD Cycles**:
1. File change detection integration
2. Template cache invalidation logic
3. Smart rebuild triggers (only when needed)
4. Template cache management

**Success Gate**: Templates rebuild automatically only when SQL files change

---

### **Phase 3B: Environment Commands**
**Builds On**: 2D complete template system + existing `environment.rs`
**Provides**: CLI commands for environment management
**Integration**: Add CLI commands that use existing environment filtering

**Key TDD Cycles**:
1. `dbfast environments` command implementation
2. `dbfast validate-env` command implementation
3. Enhanced status with environment info
4. Production safety warnings

**Success Gate**: Environment commands work with template system

---

### **Phase 4A: Remote Configuration**
**Builds On**: 3B environment system + existing `remote.rs` stub
**Provides**: Remote database configuration and validation
**Integration**: Expand `remote.rs` with real functionality

**Key TDD Cycles**:
1. Remote configuration parsing from `dbfast.toml`
2. Connection validation and testing
3. Environment linking (remote → environment config)
4. Remote management CLI commands

**Success Gate**: Can configure and validate remote database connections

---

### **Phase 4B: Backup Management**
**Builds On**: 4A remote connections + existing `backup.rs` stub
**Provides**: Full backup/restore functionality
**Integration**: Complete backup system with pg_dump/pg_restore

**Key TDD Cycles**:
1. pg_dump integration and automation
2. Backup metadata and storage management
3. pg_restore and rollback functionality
4. Backup cleanup and rotation

**Success Gate**: Complete backup/restore workflow with remote databases

## 🤖 Claude Code Integration

The orchestrator now includes seamless Claude Code integration for each phase:

### **Automatic Claude Code Launch**
When starting a phase with `./scripts/run-phase.sh 2A`, the script will:

1. **Generate Phase-Specific Prompt** - Creates a comprehensive prompt file with:
   - Phase objectives and success criteria
   - Current branch and integration context
   - TDD methodology guidance (Phase 6)
   - Phase-specific implementation areas
   - Critical tests to write
   - Available commands and workflows

2. **Launch Claude Code** - Automatically detects and launches:
   - Claude Code CLI (`claude --file prompt.md --project-context`)
   - VS Code with Claude extension
   - Fallback: Creates prompt file for manual copy

3. **Full Project Context** - Claude Code launches with:
   - Complete codebase access
   - Phase plan documentation
   - Current git branch context
   - Integration points with previous phases

### **Phase-Specific Prompts**
Each phase gets a tailored prompt including:

**Phase 2A (SQL Execution)**: Focus on replacing placeholder, PostgreSQL integration
**Phase 2B (Template Creation)**: Building on 2A, CREATE DATABASE WITH TEMPLATE
**Phase 2C (Database Cloning)**: Performance focus, <200ms target
**Phase 2D (Auto-Rebuild)**: Integration with existing scanner.rs
**Phase 3B (Environment Commands)**: CLI commands using environment.rs
**Phase 4A (Remote Configuration)**: Expanding remote.rs stub
**Phase 4B (Backup Management)**: Completing backup.rs with pg_dump

### **Claude Code Workflow**
```bash
# Start phase with Claude Code
./scripts/run-phase.sh 2A
# → Prompts to launch Claude Code
# → Y: Launches with phase context
# → N: Creates prompt file for manual use

# During development, Claude Code has access to:
# - Full codebase context
# - Phase-specific guidance
# - TDD methodology commands
# - Integration points documentation
```

## 🔧 Orchestrator Scripts

### **Enhanced Phase Runner Script**
```bash
#!/bin/bash
# scripts/run-phase.sh

PHASE=$1
DESCRIPTION=$2

if [[ -z "$PHASE" ]]; then
    echo "Usage: ./scripts/run-phase.sh 2A [description]"
    echo "Available phases: 2A, 2B, 2C, 2D, 3B, 4A, 4B"
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
    exit 1
fi

echo "🚀 Starting Phase $PHASE: $PHASE_NAME"

# Ensure we're on clean dev
git checkout dev
git pull origin dev

# Create new branch
BRANCH_NAME="phase-$(echo $PHASE | tr '[:upper:]' '[:lower:]')-$PHASE_NAME"
git checkout -b "$BRANCH_NAME"

# Check phase plan exists
PHASE_FILE="PHASING/PHASE_$PHASE.md"
if [[ ! -f "$PHASE_FILE" ]]; then
    echo "❌ Phase plan missing: $PHASE_FILE"
    echo "📝 Please create the phase plan first"
    exit 1
fi

echo "✅ Phase $PHASE environment ready"
echo "📖 Review plan: $PHASE_FILE"
echo "💡 TDD Workflow:"
echo "   🔴 make commit-tdd  # RED/GREEN phases"
echo "   🔧 make commit-clean # REFACTOR phases"
echo "   🔍 make check-all   # Quality verification"
echo "   🚀 ./scripts/complete-phase.sh $PHASE  # When done"

# Show integration point
case $PHASE in
    "2A") echo "🔗 Integration: Replace placeholder in src/commands/seed.rs" ;;
    "2B") echo "🔗 Integration: Use 2A SQL execution for template creation" ;;
    "2C") echo "🔗 Integration: Use 2B templates for fast cloning" ;;
    "2D") echo "🔗 Integration: Connect existing scanner.rs with 2B templates" ;;
    "3B") echo "🔗 Integration: Add CLI commands using existing environment.rs" ;;
    "4A") echo "🔗 Integration: Expand existing remote.rs stub" ;;
    "4B") echo "🔗 Integration: Complete existing backup.rs stub" ;;
esac
```

### **Phase Completion Script**
```bash
#!/bin/bash
# scripts/complete-phase.sh

PHASE=$1
if [[ -z "$PHASE" ]]; then
    echo "Usage: ./scripts/complete-phase.sh 2A"
    exit 1
fi

echo "🔍 Preparing Phase $PHASE for completion..."

# Run all quality checks
echo "⚙️  Running quality checks..."
if ! make prepare-pr; then
    echo "❌ Quality checks failed. Fix issues before completing phase."
    exit 1
fi

# Check for TDD commits that need cleanup
echo "🔍 Checking for TDD commits..."
TDD_COMMITS=$(git log --grep="🔴\|🟢" --oneline dev..HEAD || true)
if [[ -n "$TDD_COMMITS" ]]; then
    echo "⚠️  TDD commits found - consider squashing:"
    echo "$TDD_COMMITS"
    echo "💡 Run 'git rebase -i dev' to clean up history"
    read -p "Continue anyway? (y/N): " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        exit 1
    fi
fi

# Get phase title
PHASE_TITLE=$(head -1 "PHASING/PHASE_$PHASE.md" | sed 's/^#[[:space:]]*//')

# Create PR
echo "🚀 Creating PR for Phase $PHASE..."
gh pr create \
    --title "Phase $PHASE: $PHASE_TITLE" \
    --body "$(cat PHASING/PHASE_$PHASE.md)" \
    --base dev \
    --assignee @me

if [[ $? -eq 0 ]]; then
    echo "✅ Phase $PHASE PR created successfully"
    echo "⏳ Next steps:"
    echo "   1. Wait for CI checks to pass"
    echo "   2. Review and address any feedback"
    echo "   3. Merge when approved"
    echo "   4. Run next phase: ./scripts/run-phase.sh [NEXT_PHASE]"

    # Show next phase
    case $PHASE in
        "2A") echo "   💡 Next: ./scripts/run-phase.sh 2B" ;;
        "2B") echo "   💡 Next: ./scripts/run-phase.sh 2C" ;;
        "2C") echo "   💡 Next: ./scripts/run-phase.sh 2D" ;;
        "2D") echo "   💡 Next: ./scripts/run-phase.sh 3B" ;;
        "3B") echo "   💡 Next: ./scripts/run-phase.sh 4A" ;;
        "4A") echo "   💡 Next: ./scripts/run-phase.sh 4B" ;;
        "4B") echo "   🎉 Final phase complete! Ready for release." ;;
    esac
else
    echo "❌ Failed to create PR"
    exit 1
fi
```

### **Phase Status Script**
```bash
#!/bin/bash
# scripts/phase-status.sh

echo "📊 DBFast Phase Status"
echo "======================"

# Check current branch
CURRENT_BRANCH=$(git branch --show-current)
echo "Current branch: $CURRENT_BRANCH"

# Show phase progress
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

echo
echo "Phase Progress:"
for phase in "${PHASES[@]}"; do
    # Check if phase is tagged as complete
    if git tag -l | grep -q "phase-$(echo $phase | tr '[:upper:]' '[:lower:]')-complete"; then
        status="✅ Complete"
    # Check if there's an open PR
    elif gh pr list --state open --head "*phase-$(echo $phase | tr '[:upper:]' '[:lower:]')*" --json number | grep -q "number"; then
        status="🔄 In Review"
    # Check if phase branch exists
    elif git branch -a | grep -q "phase-$(echo $phase | tr '[:upper:]' '[:lower:]')"; then
        status="🚧 In Progress"
    else
        status="⏳ Pending"
    fi

    printf "  Phase %s: %-25s %s\n" "$phase" "${PHASE_NAMES[$phase]}" "$status"
done

echo
echo "💡 Next Action:"
for phase in "${PHASES[@]}"; do
    if ! git tag -l | grep -q "phase-$(echo $phase | tr '[:upper:]' '[:lower:]')-complete"; then
        echo "   Start Phase $phase: ./scripts/run-phase.sh $phase"
        break
    fi
done
```

## 🎯 Quality Integration Gates

### **Cumulative Testing Strategy**
Each phase must pass ALL previous phase tests plus its own:

```bash
# Phase 2A: Basic SQL execution tests
cargo test sql_execution

# Phase 2B: 2A tests + template tests
cargo test sql_execution template_creation

# Phase 2C: 2A+2B tests + cloning tests
cargo test sql_execution template_creation database_cloning

# etc.
```

### **Integration Test Progression**
```rust
// tests/integration/phase_progression_test.rs

#[test]
fn test_2a_sql_execution() {
    // Test SQL file execution works
}

#[test]
fn test_2b_builds_on_2a() {
    // Test template creation uses 2A SQL execution
}

#[test]
fn test_2c_builds_on_2b() {
    // Test cloning uses 2B templates
}

#[test]
fn test_full_workflow_2a_through_2d() {
    // End-to-end test: file change → template rebuild → clone
}
```

## 📈 Timeline and Milestones

### **7-Week Execution Plan**
```
Week 1: Phase 2A (SQL Execution Foundation)
Week 2: Phase 2B (Template Creation)
Week 3: Phase 2C (Database Cloning)
Week 4: Phase 2D (Auto-Rebuild Integration)
Week 5: Phase 3B (Environment Commands)
Week 6: Phase 4A (Remote Configuration)
Week 7: Phase 4B (Backup Management)
```

### **Daily Orchestrator Rhythm**
```bash
# Morning standup
./scripts/phase-status.sh

# Work: Follow Phase 6 TDD methodology
# RED → GREEN → REFACTOR cycles

# Evening review
git log --oneline -5  # Review day's progress
make check-all        # Verify quality maintained

# Friday phase completion (if ready)
./scripts/complete-phase.sh [CURRENT_PHASE]
```

## 🚦 Orchestrator Commands Reference

```bash
# Start a phase (with Claude Code integration)
./scripts/run-phase.sh 2A

# Check overall progress
./scripts/phase-status.sh

# Complete current phase
./scripts/complete-phase.sh 2A

# Alternative: Use Makefile shortcuts
make start-phase PHASE=2A        # Start with Claude Code
make complete-phase PHASE=2A     # Complete and create PR
make phase-status                # Show progress

# Emergency: Check quality across all phases
make check-all

# Emergency: Run all integration tests
cargo test --test '*integration*'
```

---

**The Orchestrator ensures seamless phase transitions while maintaining the disciplined TDD approach and platinum code quality standards established in Phase 6.**
