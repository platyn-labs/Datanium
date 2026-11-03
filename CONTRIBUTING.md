# Contributing to Datanium

## From issue to squash merge

1. Search open and closed issues before opening one. State the expected behavior,
   acceptance criteria and affected crate. Use an area label such as `storage`
   with `enhancement`, `bug` or `documentation` as appropriate.
2. Update your local checkout with `git switch main` and `git pull --ff-only`.
   Start a short-lived branch, for example `git switch -c feat/storage-interface`.
3. Keep each PR focused on one reviewable change. Separate a public contract,
   its first backend and unrelated refactoring. Include tests for the behavior
   introduced by each PR and a regression test for each bug fix.
4. Use Conventional Commits for commits and PR titles, for example
   `feat(storage): define key-value operations` or
   `fix(storage): preserve the previous value on write failure`.
   Use a type such as feat, fix, docs, test, refactor, ci or chore.
   Mark a breaking change with `!` and explain the migration in the PR.
5. Push the branch and open a PR against `main`. Link the issue with
   `Closes #<number>` when the PR meets all its acceptance criteria.
   Record test results and any integrity or compatibility impact.
6. Check the final revision with `gh pr checks --watch`, resolve review
   conversations and squash merge. The PR title becomes the commit title on
   `main`. Start the next branch from updated `main`.

## Checks before a pull request

Run from the workspace root. Commit Cargo.lock changes together with the
dependency change that caused them.

```sh
cargo fmt --all -- --check
cargo check --workspace --all-targets --locked
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --locked
cargo build --workspace --locked
```

GitHub Actions reports these commands as `Rust Checks`. The separate
`Security Audit` checks Cargo.lock against RustSec advisories weekly.
Run it on demand with `gh workflow run security.yml`, for example when
reviewing a dependency update. Inspect the result for the revision you changed.

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
