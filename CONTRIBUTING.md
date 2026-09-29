# Contributing to sorowill-contracts

This repo participates in the **Stellar Wave Program** on [Drips](https://drips.network/wave). Contribution work is tied to issues that maintainers tag for an active Wave, and contributors earn rewards proportional to the Points assigned to the issues they resolve.

## Ground rules

- **Do not start work on any issue until you have been assigned by the maintainer.** Applying to an issue does not mean you're assigned — wait for confirmation (via the Drips Wave dashboard or a direct assignment on GitHub) before opening a PR.
- Keep PRs scoped to the issue they resolve. Unrelated changes slow down review and can cost you the Wave window.
- Be responsive during an active Wave — issues must be resolved before the Wave ends for Points to be awarded.

## Branch naming

Use the issue number in your branch name:

```
feat/N-short-description
fix/N-short-description
```

For example: `feat/42-guardian-quorum-check` or `fix/17-checkin-deadline-rounding`.

## Pull requests

- Your PR description must reference the issue it resolves (e.g. `Closes #42`).
- Complete the [Before opening a PR](#before-opening-a-pr) checklist before requesting review.
- Add or update unit tests for any behavior change in `contracts/will/src/test.rs`.
- If you change validation rules or add an entry point, extend the fuzzing
  harness too — see [docs/FUZZING.md](./docs/FUZZING.md#adding-a-target).
- If your PR changes contract behavior (new entrypoint, validation change,
  event/error schema change, etc.), add an entry under `[Unreleased]` in
  [CHANGELOG.md](./CHANGELOG.md) describing it in the "Added" / "Changed" /
  "Fixed" section that fits. Pure docs/tooling/test-only PRs don't need one.
- If your PR adds a new `WillError` variant, add a row for it to the
  [error code table in the README](./README.md#error-codes) in the same PR —
  don't let the table and `contracts/will/src/errors.rs` drift apart.
- If your PR changes a public entry-point signature or a shared type
  (`Will`, `Beneficiary`, `Guardian`, `WillStatus`, `WillError`, ...), bump
  the crate `version` in `contracts/will/Cargo.toml` and regenerate the
  contract spec artifact — see [spec/README.md](./spec/README.md).

## Storage schema versioning

Every persisted `Will` has a `schema_version` field. Soroban decodes a
`#[contracttype]` struct strictly by field name. If you add, remove or rename
a field on `Will`, every will stored by an older contract version stops
decoding unless you also ship a migration path. The machinery is in
[`contracts/will/src/migration.rs`](./contracts/will/src/migration.rs):

- `CURRENT_SCHEMA_VERSION` is the version stamped on every newly created or
  migrated will.
- `decode_will` handles every read of a `DataKey::Will` entry. It tries the
  current layout first, then each legacy layout, and converts a legacy entry
  into the current `Will` shape. The converted will keeps its old
  `schema_version`.
- `upgrade` runs the `migrate_vN_to_vN+1` steps in order up to
  `CURRENT_SCHEMA_VERSION`. The owner-authorized `migrate_will` entry point
  calls it and persists the result.

**Version history**

| Version | Change |
|---|---|
| 0 | Wills written before `schema_version` was populated. |
| 1 | Current layout. |

**To introduce version N+1** (for example, adding a field to `Will`):

1. Copy the current `Will` definition unchanged into a legacy type, e.g.
   `WillV1` in a `legacy` module, keeping `#[contracttype]` and the original
   field names. This is the only way old entries can still be decoded.
2. Change `Will` (add the field) and bump `CURRENT_SCHEMA_VERSION` to N+1.
3. In `decode_will`, add a fallback that tries `WillVN::try_from_val` and
   converts it into `Will`. Give new fields a safe default and leave
   `schema_version` at N.
4. Add a `migrate_vN_to_vN1` step and wire it into the `match` in `upgrade`.
   Every version below `CURRENT_SCHEMA_VERSION` needs an arm.
5. Add a row to the version history table above and a CHANGELOG entry.
6. Add a test that writes a `WillVN` directly under `DataKey::Will(id)`,
   loads it through `get_will`, and calls `migrate_will` to confirm it ends up
   at N+1 with the new field set.

Never reorder, rename or retype an existing field without going through these
steps. Entry points must also accept wills that have not been migrated yet,
since `migrate_will` is opt-in.

## Before opening a PR

Run every command used by the [Test CI workflow](./.github/workflows/test.yml) and confirm it succeeds:

- [ ] `cargo clippy --all-targets -- -D warnings`
- [ ] `cargo test --workspace`
- [ ] `cargo build --workspace --release --target wasm32v1-none`
- [ ] Confirm the Test workflow is green on the PR.

## Local setup

See the [README](./README.md#local-setup) for toolchain installation and how to run the test suite.

## Mutation testing (cargo-mutants)

Code coverage alone can't tell you whether an assertion is too weak or an
edge case is missing — it only tells you a line ran, not that a test would
notice if the logic on that line broke. [`cargo-mutants`](https://mutants.rs/)
closes that gap: it systematically rewrites small pieces of the contract
(flipping a comparison, changing a constant, swapping a boolean) and reruns
the test suite against each mutant. A mutant that **survives** (tests still
pass) means no test would have caught that bug.

CI runs mutation testing automatically via `.github/workflows/mutants.yml`
on every push and PR to `main`, but it is currently **advisory only**
(`continue-on-error: true`) — a survived mutant does not fail the build. It
will graduate to a blocking check once we've triaged an initial baseline
and trust the signal.

### Running it locally

```sh
cargo install cargo-mutants --locked
cargo mutants --package will
```

This takes a while: cargo-mutants recompiles and reruns the test suite once
per candidate mutation. To scope a run while iterating on a single file:

```sh
cargo mutants --package will --file contracts/will/src/storage.rs
```

Results are written to `mutants.out/` (or wherever `--output` points) as
both a human-readable summary and a machine-readable list of mutants,
grouped as `caught`, `missed` (survived), `unviable` (didn't compile), and
`timeout`.

### Interpreting a "survived mutant" result

A survived mutant is a pointer at an *untested behavior*, not necessarily a
real bug. For each one:

1. Look at the diff cargo-mutants applied (e.g. `<` became `<=`, or a
   returned value was replaced with a constant).
2. Ask: is there a code path where this mutation would produce an
   observably different result? If yes, the test suite is missing an
   assertion or a test case for that path — add one.
3. If the mutated code is genuinely unreachable or provably
   behavior-preserving (rare, but happens with defensive/redundant checks),
   it's fine to leave as a known, documented survivor rather than write a
   test purely to satisfy the tool.

New survivors introduced by a PR should be fixed by adding or strengthening
a test in `contracts/will/src/test.rs` before merge, once the check is
blocking; until then, treat a growing survivor count as a signal to
prioritize test-writing, and file a follow-up issue for anything
out of scope for the PR at hand.

## Code coverage (cargo-llvm-cov)

CI measures code coverage via `.github/workflows/coverage.yml` using
[`cargo-llvm-cov`](https://github.com/taiki-e/cargo-llvm-cov), which works
well for `no_std`/Soroban crates because it runs against the host target —
the test suite doesn't need to run under `wasm32v1-none` for coverage to be
meaningful, only the release wasm build in `test.yml` does.

The workflow enforces a baseline threshold of **60% line coverage**,
chosen because the current suite already clears it comfortably. The goal
is a meaningful floor against large regressions (e.g. a new entry point
shipped with no tests), not a hard gate that blocks unrelated PRs over
small, incidental dips. Raise the threshold over time as coverage improves.

### Running it locally

```sh
cargo install cargo-llvm-cov --locked
rustup component add llvm-tools-preview

# Terminal summary
cargo llvm-cov --workspace

# lcov report (for editor integrations, e.g. VS Code's "Coverage Gutters")
cargo llvm-cov --workspace --lcov --output-path lcov.info

# Browsable HTML report
cargo llvm-cov --workspace --html --open
```

## Learn more

Full details on how Wave Programs work — applying, Points, rewards, and payouts — are documented at <https://drips.network/wave>.
