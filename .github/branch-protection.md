# Branch Protection Configuration

This document outlines the recommended branch protection rules for the main branch.

## GitHub Branch Protection Rules

Configure these settings in GitHub Settings → Branches → Add rule for `main`:

### Required Status Checks
- ✅ Require status checks to pass before merging
- ✅ Require branches to be up to date before merging
- Required checks:
  - `Quality Checks`
  - `Test & Coverage`
  - `Security Audit`

### Required Reviews
- ✅ Require pull request reviews before merging
- Required number of reviewers: `1`
- ✅ Dismiss stale reviews when new commits are pushed
- ✅ Require review from code owners (if CODEOWNERS file exists)
- ✅ Restrict pushes that create new commits

### Additional Restrictions
- ✅ Restrict pushes that delete this branch
- ✅ Require linear history (optional but recommended)
- ✅ Include administrators (applies rules to admins too)

### Auto-merge Requirements
- ✅ Allow auto-merge
- ✅ Automatically delete head branches

## CLI Configuration (Alternative)

You can also configure these rules via GitHub CLI:

```bash
# Enable branch protection
gh api repos/:owner/:repo/branches/main/protection \
  --method PUT \
  --field required_status_checks='{"strict":true,"contexts":["Quality Checks","Test & Coverage","Security Audit"]}' \
  --field enforce_admins=true \
  --field required_pull_request_reviews='{"required_approving_review_count":1,"dismiss_stale_reviews":true}' \
  --field restrictions=null \
  --field allow_deletions=false \
  --field allow_force_pushes=false
```

## Verification

After applying these rules:
1. Try to push directly to main (should be blocked)
2. Create a PR and verify status checks are required
3. Verify that failing CI blocks the merge
4. Test that admin users also follow the rules