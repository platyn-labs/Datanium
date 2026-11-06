# Contributing to Datanium

## From issue to squash merge

Use GitHub Flow with a short-lived branch for each focused change. Run the
commands below from the workspace root, where Cargo.toml is located. The example
uses issue #2 and the branch `codex/column-management`. Replace the issue number,
branch name and commit or PR title for your own task.

### 1. Choose an issue

The GitHub CLI needs an authenticated account. Check with `gh auth status` and
use `gh auth login` if necessary.

Search open and closed issues before creating one, then read the acceptance
criteria of the issue you want to work on.

```sh
gh issue list --state all --search "column management"
gh issue view 2
```

If an issue is missing, create it interactively with `gh issue create`. Describe
the expected behavior, acceptance criteria and affected crate. Use an area label
such as `core` with `enhancement`, `bug` or `documentation` as appropriate.

### 2. Start from an updated main branch

Check your working directory first. Commit unfinished work on its own branch or
save it before switching tasks. Once the working directory is clean, update main
and create your feature branch.

```sh
git status
git switch main
git pull --ff-only
git switch -c feat/column-management
```

### 3. Develop and test the change

Keep each PR focused on one reviewable change. Separate a public contract, its
first implementation and unrelated refactoring. Include tests for the behavior
introduced by each PR and a regression test for each bug fix.

Format the code, then run the same checks as CI. Resolve failures before merging.

```sh
cargo fmt --all
cargo fmt --all -- --check
cargo check --workspace --all-targets --locked
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --locked
cargo build --workspace --locked
```

Commit Cargo.lock changes together with the dependency change that caused them.
GitHub Actions reports the checks above as `Rust Checks`.

### 4. Review and commit your work

Inspect the changes and select the ones belonging to your feature. `git add -p`
lets you stage individual changes in tracked files. Add new files explicitly
with `git add path/to/new-file.rs`, replacing the example path with your file.

```sh
git status
git diff
git add -p
git diff --cached
git commit -m "feat(core): implement column management"
```

Use Conventional Commits for commits and PR titles. Choose a type such as feat,
fix, docs, test, refactor, ci or chore, with an optional scope such as `(core)`.
Mark a breaking change with `!` and explain the migration in the PR. You can make
several small commits on the same feature branch as work progresses.

### 5. Push the branch and open a pull request

```sh
git push -u origin codex/column-management
gh pr create --base main --title "feat(core): implement column management" --web
```

Complete the PR template in the browser. Describe the change, verification
results and integrity or compatibility impact. Add `Closes #2` when the PR meets
all acceptance criteria of issue #2. GitHub closes the linked issue when the PR
is merged into main.

### 6. Check CI and squash merge

From your feature branch, wait for the PR checks and review the final diff.

```sh
gh pr checks --watch
gh pr diff
```

If a check fails or feedback requires a change, edit the code, rerun the relevant
local checks and commit the fix. Push it with `git push` to update the existing
PR, then check CI again.

Once `Rust Checks` passes and review conversations are resolved, squash merge.
The PR title becomes the commit title on main.

```sh
gh pr merge --squash --delete-branch
```

### 7. Prepare for the next feature

After the merge, update your local main branch and prune references to deleted
remote branches. Start your next feature branch from this updated main.

```sh
git switch main
git pull --ff-only
git fetch --prune
```

## Dependency security checks

The separate `Security Audit` workflow checks Cargo.lock against RustSec
advisories weekly and on manual request. When changing dependencies, push your
commits and run it for your feature branch.

```sh
gh workflow run security.yml --ref codex/column-management
gh run list --workflow security.yml --branch codex/column-management
```

Use `gh run view RUN_ID` to inspect the run for your latest commit, replacing
`RUN_ID` with the ID from the list. The audit runs separately from PR checks.

## Data integrity

- Document observable behavior in Rustdoc, including null values, type mismatches
  and errors. Return typed errors so callers can distinguish failure kinds.
- Failed operations must preserve rows, schema and ID counters. Test error paths
  as well as successful changes.
- Keep internal invariants covered by unit tests and exercise complete workflows
  through the public API in integration tests.
