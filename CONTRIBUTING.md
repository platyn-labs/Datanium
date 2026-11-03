# Contributing to Datanium

## From issue to squash merge

Use GitHub Flow with a short-lived branch for each focused change. Run the
commands below from the workspace root, where Cargo.toml is located. The example
uses issue #2 and the branch `feat/storage-interface`. Replace the issue number,
branch name and commit or PR title for your own task.

### 1. Choose an issue

The GitHub CLI needs an authenticated account. Check with `gh auth status` and
use `gh auth login` if necessary.

Search open and closed issues before creating one, then read the acceptance
criteria of the issue you want to work on.

```sh
gh issue list --state all --search "storage interface"
gh issue view 2
```

If an issue is missing, create it interactively with `gh issue create`. Describe
the expected behavior, acceptance criteria and affected crate. Use an area label
such as `storage` with `enhancement`, `bug` or `documentation` as appropriate.

### 2. Start from an updated main branch

Check your working directory first. Commit unfinished work on its own branch or
save it before switching tasks. Once the working directory is clean, update main
and create your feature branch.

```sh
git status
git switch main
git pull --ff-only
git switch -c feat/storage-interface
```

### 3. Develop and test the change

Keep each PR focused on one reviewable change. Separate a public contract, its
first backend and unrelated refactoring. Include tests for the behavior
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
git commit -m "feat(storage): define storage interface"
```

Use Conventional Commits for commits and PR titles. Choose a type such as feat,
fix, docs, test, refactor, ci or chore, with an optional scope such as `(storage)`.
Mark a breaking change with `!` and explain the migration in the PR. You can make
several small commits on the same feature branch as work progresses.

### 5. Push the branch and open a pull request

```sh
git push -u origin feat/storage-interface
gh pr create --base main --title "feat(storage): define storage interface" --web
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
gh workflow run security.yml --ref feat/storage-interface
gh run list --workflow security.yml --branch feat/storage-interface
```

Use `gh run view RUN_ID` to inspect the run for your latest commit, replacing
`RUN_ID` with the ID from the list. The audit runs separately from PR checks.

## Storage contracts and data integrity

- Document observable behavior in Rustdoc. Define the meaning of a missing key,
  an empty value, overwrite and repeated deletion. Return typed errors so a
  caller can distinguish absence from a storage failure.
- Specify the state after a failed write. For an operation promised to be
  atomic, test that readers see either the complete old value or the complete
  new value. Exercise error paths as well as successful operations.
- State when a write is acknowledged and what that acknowledgment guarantees.
  For persistent backends, test reopen and interrupted writes against that
  guarantee. Handle truncated or corrupt data explicitly and preserve the
  original data for diagnosis.
- Document on-disk format changes, version detection and migration or explicit
  rejection of older formats. Keep compatibility fixtures before changing a
  released format. Use isolated temporary directories in tests.
- Keep the backend replaceable. Exercise the same contract against each backend.
  Measure optimizations with reproducible benchmarks and retain correctness tests.
