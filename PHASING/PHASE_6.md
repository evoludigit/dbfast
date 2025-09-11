# Phase 6: Linear Pipeline Process

**Goal**: Establish a rigorous, linear development pipeline ensuring platinum code quality through micro TDD cycles and systematic branch management with pragmatic pre-commit handling.

## 🏗️ Linear Pipeline Overview

Each phase follows this exact sequence with **strategic flexibility** for TDD workflows:

```
Phase NN Planning → Branch Creation → Micro TDD Cycles → PR → Review → Merge → Next Phase
```

## 📋 Phase Execution Protocol

### 1. Phase Preparation
```bash
# Create new branch for phase
git checkout dev
git pull origin dev
git checkout -b phase-NN-description

# Create phase documentation
touch PHASING/PHASE_NN.md
# Write detailed phase plan with:
# - Clear objectives
# - Success criteria
# - Test scenarios
# - Implementation steps
```

### 2. Micro TDD Cycle Pattern

**Each feature/component follows exactly:**

#### Red 🔴 (Write Failing Test)
- Write test that captures next small requirement
- Test MUST fail for the right reason
- Run test suite to confirm failure
- **Commit with `--no-verify`** (failing tests expected):
```bash
git add tests/
git commit --no-verify -m "🔴 Add failing test for [feature]"
```

#### Green 🟢 (Minimal Implementation)
- Write MINIMAL code to make test pass
- Focus on making it work, NOT perfect
- Run test to ensure it passes
- **Commit with `--no-verify`** (may have quality issues):
```bash
git add src/
git commit --no-verify -m "🟢 Implement [feature] - tests passing"
```

#### Refactor 🔧 (Improve Quality)
- Improve code quality without changing behavior
- Remove duplication, improve naming, structure
- Ensure all tests still pass
- Run linting and formatting
- **Must pass full pre-commit hooks**:
```bash
git add .
git commit -m "🔧 Refactor [component] - improve [specific aspect]"
# ^ NO --no-verify - must pass all quality checks
```

#### Quality Gates ✅
**Every refactor cycle must pass:**
```bash
cargo test --all
cargo clippy --all-targets --all-features -- -D warnings
cargo fmt --all -- --check
cargo check --all-targets --all-features
```

### 3. Pre-commit Strategy

#### When `--no-verify` is ALLOWED ✅
- **Red phase commits** (failing tests are expected)
- **Green phase commits** (WIP implementations)
- **Experimental branches** (marked as WIP)
- **Emergency hotfixes** (exceptional circumstances only)

#### When `--no-verify` is FORBIDDEN ❌
- **Refactor phase commits** (must pass all quality checks)
- **Final phase commits** (going into PR)
- **Merge commits** (into dev/main branches)
- **Release commits** (tagged versions)

#### Makefile Helpers
```makefile
commit-tdd:
	@echo "🔴🟢 TDD commit - bypassing pre-commit hooks"
	git commit --no-verify
	@echo "⚠️  Remember to refactor and clean up!"

commit-clean:
	@echo "🔍 Running full quality checks before commit..."
	make check-all
	git commit
	@echo "✅ Clean commit completed"

commit-wip:
	@echo "⚠️  WIP commit - bypassing pre-commit hooks"
	git commit --no-verify
	@echo "🚨 Remember to clean up before PR!"
```

### 4. Commit Standards

#### Commit Message Format
```
<type> <description>

<optional detailed explanation>

Tests: [test coverage status]
Performance: [any performance notes]
Breaking: [any breaking changes]
```

#### Commit Types
- 🔴 `RED:` - Failing test added (--no-verify OK)
- 🟢 `GREEN:` - Minimal implementation (--no-verify OK)
- 🔧 `REFACTOR:` - Code quality improvement (must pass pre-commit)
- ✨ `FEAT:` - New feature completion (must pass pre-commit)
- 🐛 `FIX:` - Bug fix (must pass pre-commit)
- 📝 `DOCS:` - Documentation (must pass pre-commit)
- 🧪 `TEST:` - Test improvements (must pass pre-commit)
- 🔧 `CONFIG:` - Configuration changes (must pass pre-commit)

### 5. PR Preparation Protocol

#### Clean-up Before PR
```bash
# Review all commits in branch
git log --oneline dev..HEAD

# Identify any --no-verify commits that need cleanup
git log --grep="🔴\|🟢" --oneline

# Interactive rebase to clean up TDD commits
git rebase -i dev

# Ensure final state passes all checks
make check-all
```

#### PR Requirements (All Must Pass)
- [ ] **No `--no-verify` commits in final PR** (squashed/rebased)
- [ ] All tests passing (unit, integration, benchmarks)
- [ ] Code coverage maintained or improved
- [ ] No clippy warnings
- [ ] Formatted code (rustfmt)
- [ ] Documentation updated
- [ ] Performance benchmarks stable
- [ ] Security audit clean (cargo audit)

### 6. Pull Request Process

#### PR Creation
```bash
# Ensure branch is clean
make check-all

# Push branch
git push origin phase-NN-description

# Create PR via GitHub CLI
gh pr create \
  --title "Phase NN: [Description]" \
  --body "$(cat PHASING/PHASE_NN.md)" \
  --base dev \
  --head phase-NN-description \
  --assignee @me
```

#### PR Review Process
1. **Automated Checks** - CI/CD pipeline must be GREEN
2. **Commit History Review** - No `--no-verify` commits in final PR
3. **Code Review** - Self-review before requesting review
4. **Manual Testing** - Test on local environment
5. **Documentation Review** - Ensure docs are accurate
6. **Performance Review** - Check benchmark results

### 7. Merge Protocol

#### Merge Requirements
```bash
# All conditions MUST be met:
✅ CI/CD pipeline GREEN
✅ All review comments addressed
✅ No merge conflicts with dev
✅ Performance benchmarks acceptable
✅ Documentation complete
✅ Test coverage maintained
✅ Clean commit history (no --no-verify commits)
```

#### Merge Process
```bash
# Squash and rebase merge ONLY
gh pr merge --squash --delete-branch

# Verify merge
git checkout dev
git pull origin dev
git log --oneline -5  # Verify clean history
```

### 8. Post-Merge Validation

```bash
# Validate dev branch after merge
cargo test --all --release
cargo bench --all
cargo doc --all --no-deps

# Tag phase completion
git tag phase-NN-complete
git push origin phase-NN-complete
```

## 🎯 Quality Standards (Platinum Level)

### Code Quality Metrics
- **Test Coverage**: ≥90% line coverage, ≥95% critical path coverage
- **Clippy**: Zero warnings on all targets
- **Documentation**: All public APIs documented with examples
- **Performance**: No regression, benchmarks within 5% of baseline
- **Security**: Zero vulnerabilities in dependencies

### Test Requirements
- **Unit Tests**: Every function/method tested
- **Integration Tests**: End-to-end scenarios covered
- **Property Tests**: Critical algorithms property-tested
- **Performance Tests**: Benchmarks for performance-critical code
- **Error Cases**: All error paths tested

### Documentation Standards
- **Code Comments**: Complex logic explained
- **API Documentation**: All public APIs with examples
- **README Updates**: Feature documentation current
- **Change Log**: Breaking changes documented
- **Architecture Notes**: Design decisions recorded

## 🚫 Forbidden Shortcuts

### NEVER Allowed
- ❌ `--no-verify` commits in final PR
- ❌ Merging without all checks GREEN
- ❌ Force pushing to shared branches
- ❌ Bypassing code review
- ❌ Committing TODO comments in production code
- ❌ Ignoring clippy warnings in final commits
- ❌ Merging with failing benchmarks

### Acceptable During Development
- ✅ `--no-verify` for TDD red/green phases
- ✅ WIP commits with quality issues
- ✅ Experimental code (marked as WIP)
- ✅ Quick saves during development

### Quality Gates That Cannot Be Bypassed
- All automated tests passing in PR
- Code coverage thresholds met in PR
- Performance benchmarks stable in PR
- Security vulnerabilities addressed
- Documentation complete and accurate in PR

## 📊 Phase Progress Tracking

### Daily Status Check
```bash
# Run before any work
make check-all  # Custom Makefile target for all quality checks
git status      # Ensure clean working directory
git log --oneline -10  # Review recent progress

# Check for cleanup needed
git log --grep="--no-verify\|WIP\|🔴\|🟢" --oneline
```

### Phase Completion Criteria
Each phase is complete ONLY when:
- [ ] All planned features implemented with tests
- [ ] All TDD commits cleaned up and refactored
- [ ] All quality gates passing
- [ ] Documentation complete
- [ ] Performance benchmarks stable
- [ ] PR merged to dev branch with clean history
- [ ] Phase tagged in git
- [ ] Next phase planned and documented

## 🔄 Iteration Protocol

### Within Phase Iterations
1. **Plan** - Define next micro-feature
2. **Red** - Write failing test (`--no-verify` OK)
3. **Green** - Minimal implementation (`--no-verify` OK)
4. **Refactor** - Improve quality (must pass pre-commit)
5. **Repeat** - Until phase objectives met

### TDD Cleanup Pattern
```bash
# During development - quick TDD cycles
git commit --no-verify -m "🔴 Add test for feature X"
git commit --no-verify -m "🟢 Basic implementation of feature X"

# Before moving on - clean refactor
make check-all
git commit -m "🔧 Refactor feature X - platinum quality"

# Before PR - squash TDD commits if needed
git rebase -i dev  # Combine related TDD commits
```

### Between Phase Transitions
1. **Review** - Analyze phase outcomes
2. **Clean** - Ensure no `--no-verify` commits remain
3. **Document** - Update phase status
4. **Plan** - Design next phase thoroughly
5. **Branch** - Create clean branch for next phase
6. **Execute** - Follow pipeline protocol

## 🎯 Success Metrics

### Development Velocity
- Phases completed on schedule
- Minimal rework required
- Efficient TDD workflows
- Clean git history maintained
- PR review cycles minimized

### Code Quality
- Zero production bugs introduced
- Performance targets met
- Security vulnerabilities prevented
- Maintainability scores high
- All PRs pass quality gates

### Team Collaboration
- Clear progress visibility
- Predictable delivery timeline
- Knowledge sharing through documentation
- Minimal context switching
- Efficient development workflows

## 🔧 Tools and Automation

### Required Make Targets
```makefile
check-all: test lint format audit bench doc
test:
	cargo test --all --verbose
lint:
	cargo clippy --all-targets --all-features -- -D warnings
format:
	cargo fmt --all -- --check
audit:
	cargo audit
bench:
	cargo bench --all
doc:
	cargo doc --all --no-deps

# TDD helpers
commit-tdd:
	git commit --no-verify
commit-clean:
	make check-all && git commit
commit-wip:
	git commit --no-verify

# PR preparation
prepare-pr:
	@echo "🔍 Checking for --no-verify commits..."
	@git log --grep="🔴\|🟢" --oneline || echo "✅ No TDD commits found"
	@echo "🔍 Running full quality checks..."
	make check-all
	@echo "✅ Ready for PR!"
```

### Git Hooks Strategy

#### Pre-commit Hook (Selective)
```bash
#!/bin/sh
# .git/hooks/pre-commit

# Allow bypassing with --no-verify during TDD
# But enforce on refactor/final commits

# Check if this is a TDD commit (contains 🔴 or 🟢)
commit_msg=$(git log --format=%B -n 1 HEAD 2>/dev/null || echo "")
if echo "$commit_msg" | grep -q "🔴\|🟢"; then
    echo "⚠️  TDD commit detected - skipping some checks"
    # Run minimal checks only
    cargo test --all --quiet
else
    echo "🔍 Full quality checks..."
    make check-all || exit 1
fi
```

## 📈 Continuous Improvement

### Phase Retrospectives
After each phase completion:
- What went well with TDD workflow?
- Did pre-commit strategy help or hinder?
- How can we optimize the cleanup process?
- Were quality standards maintained?

### Pipeline Evolution
- Monitor TDD cycle efficiency
- Identify bottlenecks in quality gates
- Optimize pre-commit hook strategy
- Refine cleanup automation

---

**Remember: Platinum code quality is achieved through disciplined refactoring, not perfect first attempts. The `--no-verify` option enables efficient TDD workflows while maintaining uncompromising quality standards in the final deliverable.**
