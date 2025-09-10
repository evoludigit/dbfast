# DBFast Development Phases

This directory contains the structured development plan for DBFast, broken down into 5 implementable phases.

## Overview

DBFast development follows a **Test-Driven Development (TDD)** approach where each phase:

1. **Defines clear deliverables** with success criteria
2. **Provides example tests** to write first  
3. **Specifies implementation details** to make tests pass
4. **Builds incrementally** on previous phases

## Phase Structure

### [Phase 1 - Core Foundation](PHASE_1.md)
- **Duration**: 1-2 weeks
- **Focus**: Basic Rust project, PostgreSQL connectivity, file scanning
- **Key Deliverables**: Configuration, database connection, file discovery
- **Success**: Can connect to DB and scan SQL files

### [Phase 2 - Template Management](PHASE_2.md)  
- **Duration**: 1-2 weeks
- **Focus**: Template creation, database cloning, change detection
- **Key Deliverables**: Build templates, clone in <200ms, validate templates
- **Success**: Fast database seeding works reliably

### [Phase 3 - Environment Filtering](PHASE_3.md)
- **Duration**: 1 week
- **Focus**: Environment-specific file filtering, multiple templates
- **Key Deliverables**: Filter files by environment, prevent prod accidents
- **Success**: Different environments get different files safely

### [Phase 4 - Remote Deployment](PHASE_4.md)
- **Duration**: 1-2 weeks  
- **Focus**: Safe remote deployment, backup/rollback, dump-based deployment
- **Key Deliverables**: Deploy to remotes, automatic backup, rollback on failure
- **Success**: Production-safe deployments with full recovery

### [Phase 5 - CLI Polish & Production](PHASE_5.md)
- **Duration**: 1 week
- **Focus**: UX polish, multi-repo support, watch mode, production readiness
- **Key Deliverables**: Great CLI experience, advanced features, monitoring
- **Success**: Production-ready tool with enterprise features

## Development Approach

### TDD Workflow for Each Phase

1. **Read the phase document** to understand deliverables
2. **Write tests first** based on the provided examples
3. **Run tests** (they should fail initially)
4. **Implement minimal code** to make tests pass
5. **Refactor** while keeping tests green
6. **Add more tests** for edge cases
7. **Complete phase** when all success criteria met

### Phase Dependencies

- **Phase 1** → **Phase 2**: Need DB connection and file scanning for templates
- **Phase 2** → **Phase 3**: Need templates working before environment filtering  
- **Phase 3** → **Phase 4**: Need environment filtering before remote deployment
- **Phase 4** → **Phase 5**: Need core functionality before polish features

Each phase builds on the previous one, so they should be completed in order.

## Getting Started

### For Phase 1
```bash
# Start with Phase 1
cat PHASING/PHASE_1.md

# Create the basic project structure
cargo init
# Then follow Phase 1 TDD approach
```

### When Starting Each Phase
1. Read the full phase document
2. Set up the test fixtures described
3. Write the example tests provided
4. Run `cargo test` (should fail)
5. Implement code to make tests pass
6. Move to next phase when success criteria met

## Success Metrics

### Phase 1 Complete
- [ ] Can connect to PostgreSQL
- [ ] Can scan SQL files and calculate hashes
- [ ] Configuration loads correctly
- [ ] Basic CLI commands work

### Phase 2 Complete  
- [ ] Can build template from SQL files
- [ ] Can clone database in <200ms
- [ ] Change detection works correctly
- [ ] Template validation passes

### Phase 3 Complete
- [ ] Environment filtering works correctly
- [ ] Production never gets dev/test files
- [ ] Multiple environment templates supported
- [ ] Environment commands work

### Phase 4 Complete
- [ ] Can deploy safely to remote databases
- [ ] Automatic backup and rollback works
- [ ] Dump-based deployment reliable
- [ ] Environment safety prevents accidents

### Phase 5 Complete
- [ ] CLI is polished and user-friendly
- [ ] Multi-repo support works
- [ ] Watch mode auto-rebuilds
- [ ] Production monitoring ready

## Final Deliverable

After all 5 phases: **A production-ready DBFast tool** that transforms database seeding from a 60-second bottleneck into a 100ms delight, with enterprise-grade safety and reliability.

## Injecting Current Phase

When prompting Claude to work on a specific phase, include:

```
Current Phase: PHASE_X

[Include the relevant PHASE_X.md content]

Please implement this phase using TDD approach.
```

This gives Claude the exact context and deliverables for focused development.