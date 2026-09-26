# Changelog

All notable changes to the `will` contract are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project uses the `version` field in
[`contracts/will/Cargo.toml`](./contracts/will/Cargo.toml) as its version
identifier. Each entry below the "in Cargo.toml (change grew here)" line
gets its own [contract spec artifact](./spec) once exported.

## [Unreleased]

### Added

- `WillError::DuplicateToken` (code 40): `create_will` and `batch_create_wills`
  now reject a `tokens` list that names the same token address twice (#350).
- `get_will_history` now records the `confirm_will` (`PendingConfirmation` to
  `Active`) and `close_will` (`Released` to `Settled`) transitions (#352).

### Fixed

- Restored five `WillError` variants (`InvalidTokenCount`, `InvalidPreimageLength`,
  `InvalidCommitmentLength`, `DuplicateCommitment`, `BatchTooLarge`) and the
  `total_balance` helper that a bad merge had dropped, which left `main` failing
  to compile. The variants are re-numbered 41-45 so the already-published
  `DuplicateToken` (40) keeps its code; `README.md` and `spec/will-v0.1.0.json`
  are updated to match.
- `create_will` / `cancel_will` now record the will's real status in the audit
  trail instead of a hardcoded `Active`, so `get_will_history` is accurate for
  wills created with a confirmation delay and cancelled while pending (#351).
- `cancel_will` now decrements `ProtocolStats.total_locked_by_token` for every
  token the will held, not just the primary token, so `get_protocol_stats` no
  longer overstates locked value after a multi-token cancellation (#353).
- `create_will` derives the legacy `token`/`balance` mirror from the
  accumulated `balances` map, so the two can no longer disagree (#350).

### Removed

- Removed unused `InvalidPercentage` (code 22) error variant from `WillError`.

## [0.1.0] - Initial shipped behavior

Seeded entry summarizing the contract's behavior as of this changelog's
introduction. See the [README's Contract Functions table](./README.md#contract-functions)
and [`spec/will-v0.1.0.json`](./spec/will-v0.1.0.json) for the authoritative,
up-to-date interface.

### Added

- Core will lifecycle: `create_will`, `check_in`, `trigger_will`,
  `emergency_checkin`, `release_inheritance`, `cancel_will`, `close_will`.
- Beneficiary management: `update_beneficiaries`, `renounce_beneficiary`,
  basis-point-based percentage splits (must sum to 10,000).
- Guardian override: up to 3 named guardians, weighted quorum voting via
  `guardian_trigger`, guardian list management via `update_guardians`, and
  a cooldown period after guardian-list changes before a vote can force a
  release.
- Multi-token support: a will can hold balances across multiple SEP-41
  tokens (or native XLM) simultaneously via `top_up`.
- Batch and convenience operations: `batch_check_in`, `batch_create_wills`,
  `clone_will`, `merge_wills`, `set_delegate` (delegated check-in),
  `migrate_will`, `archive_will`.
- Query surface: `get_will`, `get_wills_by_owner`,
  `get_wills_by_owner_and_status`, `get_wills_by_beneficiary`,
  `get_triggered_wills`, `get_protocol_stats`, `get_will_history`,
  `get_contract_version`.
- On-chain audit trail via `WillStatusTransition` records, retrievable
  through `get_will_history`.
- `WillError` numeric error codes for every failure mode (see the
  [README's error code reference](./README.md#error-codes)).
- Resource-cost profiling suite (`docs/RESOURCE_COSTS.md`) and a
  coverage-guided + property-based fuzzing suite (`docs/FUZZING.md`)
  covering `create_will` and `update_beneficiaries`.

[Unreleased]: https://github.com/SoroWill/sorowill-contracts/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/SoroWill/sorowill-contracts/releases/tag/v0.1.0
