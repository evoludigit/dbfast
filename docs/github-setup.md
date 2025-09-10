# GitHub Configuration & CI/CD Setup

This directory contains professional-grade GitHub workflows and configurations for the `dbfast` project.

## 🛡️ Branch Protection

### Setup Instructions

1. Go to your repository Settings → Branches
2. Click "Add rule" for the `main` branch
3. Configure the following settings:

```
☑️ Require a pull request before merging
  ☑️ Require approvals (1 required)
  ☑️ Dismiss stale reviews when new commits are pushed
  ☑️ Require review from CODEOWNERS

☑️ Require status checks to pass before merging
  ☑️ Require branches to be up to date before merging
  Required status checks:
    - Quality Checks
    - Test & Coverage  
    - Security Audit

☑️ Require conversation resolution before merging
☑️ Include administrators
☑️ Restrict pushes that create files
☑️ Restrict pushes that delete this branch
```

### Alternative: CLI Setup

Run this command to set up branch protection via GitHub CLI:

```bash
gh api repos/:owner/:repo/branches/main/protection \
  --method PUT \
  --field required_status_checks='{"strict":true,"contexts":["Quality Checks","Test & Coverage","Security Audit"]}' \
  --field enforce_admins=true \
  --field required_pull_request_reviews='{"required_approving_review_count":1,"dismiss_stale_reviews":true}' \
  --field restrictions=null \
  --field allow_deletions=false \
  --field allow_force_pushes=false
```

## 🔄 Workflows Overview

| Workflow | Trigger | Purpose |
|----------|---------|---------|
| **CI** (`ci.yml`) | Push/PR | Core quality checks, testing, security |
| **Security** (`security.yml`) | Push/PR/Schedule | Security scanning, vulnerability checks |
| **PR Validation** (`pr-validation.yml`) | PR events | PR metadata validation, smoke tests |
| **Auto-merge** (`auto-merge.yml`) | Dependabot PRs | Automated dependency updates |
| **Release** (`release.yml`) | Tags/Manual | Multi-platform builds, GitHub releases |
| **Deploy** (`deploy.yml`) | Manual/Push | Environment deployments |
| **Rollback** (`rollback.yml`) | Manual | Emergency rollback procedures |

## 🔧 Required Secrets

Set these secrets in your repository settings:

| Secret | Description | Required For |
|--------|-------------|--------------|
| `CODECOV_TOKEN` | Codecov integration | Coverage reporting |
| `CARGO_REGISTRY_TOKEN` | Crates.io publishing | Release workflow |

### Optional Secrets for Enhanced Features

| Secret | Description | Use Case |
|--------|-------------|----------|
| `SLACK_WEBHOOK_URL` | Slack notifications | Deployment notifications |
| `DISCORD_WEBHOOK` | Discord notifications | Release notifications |

## 📋 Environment Setup

### 1. Create GitHub Environments

Go to Settings → Environments and create:

#### Staging Environment
- **Name**: `staging`
- **Deployment branches**: Only `main` branch
- **Environment secrets**: staging-specific configs

#### Production Environment  
- **Name**: `production`
- **Protection rules**: 
  - ✅ Required reviewers (1)
  - ✅ Wait timer (5 minutes)
- **Deployment branches**: Only `main` branch and release tags
- **Environment secrets**: production configs

#### Production Rollback Environment
- **Name**: `production-rollback`
- **Protection rules**: 
  - ✅ Required reviewers (2)
  - ✅ Wait timer (10 minutes)

### 2. Configure Repository Labels

Create these labels for better PR management:

```bash
# Areas
gh label create "area: core" --color "0052cc"
gh label create "area: cli" --color "0052cc"  
gh label create "area: ci" --color "1d76db"
gh label create "area: docs" --color "0e8a16"

# Types
gh label create "type: bug" --color "d73a4a"
gh label create "type: feature" --color "a2eeef"
gh label create "type: maintenance" --color "fef2c0"

# Priority
gh label create "priority: high" --color "b60205"
gh label create "priority: low" --color "0e8a16"

# Status
gh label create "auto-merge" --color "ededed"
gh label create "breaking-change" --color "b60205"
gh label create "security" --color "d73a4a"
gh label create "performance" --color "fbca04"
```

## 🚀 Deployment Process

### Staging Deployment (Automatic)
- Triggered on push to `main` branch
- Runs full test suite
- Deploys to staging environment
- Performs health checks

### Production Deployment (Manual)
1. Go to Actions → Deploy workflow
2. Select "Run workflow"  
3. Choose `production` environment
4. Specify version (optional)
5. Workflow waits for approval
6. Deploys with comprehensive monitoring

### Emergency Rollback
1. Go to Actions → Emergency Rollback
2. Select environment to rollback
3. Specify target version (optional)
4. Type "ROLLBACK" to confirm
5. Provide rollback reason
6. For production: additional approval required

## 🔍 Security Features

### Automated Security Scanning
- **CodeQL**: Static analysis for security vulnerabilities
- **Cargo Audit**: Vulnerability scanning for Rust dependencies
- **Cargo Deny**: License and security policy enforcement
- **TruffleHog**: Secret scanning in commits
- **Dependabot**: Automated dependency updates

### Security Policies
- All dependencies scanned weekly
- High/critical vulnerabilities fail builds
- Secrets scanning on all commits
- License compliance checking

## 📊 Quality Gates

All PRs must pass:
1. **Quality Checks**: Formatting, linting, compilation
2. **Test & Coverage**: Full test suite with coverage reporting  
3. **Security Audit**: Vulnerability and license checks
4. **Metadata Validation**: PR title, branch naming conventions

## 🔄 Dependency Management

### Automated Updates via Dependabot
- **Rust dependencies**: Weekly updates with smart grouping
- **GitHub Actions**: Weekly updates  
- **Docker images**: Weekly patch updates only

### Auto-merge Strategy
- ✅ Patch updates: Auto-merged after tests pass
- ✅ Minor dev dependencies: Auto-merged after tests pass
- ❌ Major updates: Manual review required
- ❌ Minor direct dependencies: Manual review required

## 📈 Monitoring & Observability

### Workflow Monitoring
- All workflows report status to GitHub checks
- Failed workflows create notifications
- Deployment workflows create detailed reports
- Rollback workflows create incident issues

### Performance Tracking
- Benchmark results stored as artifacts
- Performance regression detection on performance-labeled PRs
- Build time tracking across workflows

## 🛠️ Development Workflow

### Standard Development Flow
1. Create feature branch (`feature/description`)
2. Make changes with commits following conventional format
3. Push branch and create PR
4. Automated validation runs (PR Validation workflow)
5. Code review and approval required
6. Auto-merge after all checks pass

### Release Flow  
1. Create release tag (`v1.0.0`)
2. Release workflow automatically triggers
3. Multi-platform binaries built
4. GitHub release created with assets
5. Optionally published to crates.io

### Hotfix Flow
1. Create hotfix branch from main (`fix/critical-issue`)
2. Make minimal changes
3. Create PR with `priority: high` label
4. Emergency merge possible with admin override
5. Deploy immediately to staging/production

## 🔧 Troubleshooting

### Common Issues

#### Workflow Permissions
If workflows fail with permission errors:
```bash
# Go to Settings → Actions → General
# Set "Workflow permissions" to "Read and write permissions"
```

#### Branch Protection Conflicts
If you can't merge due to branch protection:
1. Ensure all required checks are passing
2. Verify PR has required approvals
3. Check that branch is up-to-date with main

#### Security Scan Failures
If security scans fail:
1. Check `cargo audit` output for vulnerabilities
2. Update dependencies or add exceptions in `deny.toml`
3. Review CodeQL results in Security tab

### Getting Help

1. Check workflow logs in Actions tab
2. Review this documentation
3. Create an issue with the `ci` label
4. Contact repository maintainers

---

## 📚 Additional Resources

- [GitHub Branch Protection Rules](https://docs.github.com/en/repositories/configuring-branches-and-merges-in-your-repository/managing-protected-branches)
- [GitHub Environments](https://docs.github.com/en/actions/deployment/targeting-different-environments)
- [Dependabot Configuration](https://docs.github.com/en/code-security/dependabot/dependabot-version-updates)
- [GitHub Actions Documentation](https://docs.github.com/en/actions)

*Last updated: September 2025*