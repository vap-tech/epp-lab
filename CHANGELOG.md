# Changelog

## Unreleased

### Changed

- Split EPP command execution out of the connection handler into
  `epp/dispatch.rs`.
- Moved `hello`, `login`, `logout`, and parse-error response handling into the
  dispatcher.
- Added transaction delivery tracking with `delivered`, `failed`, and
  `unknown` states.
- Made XML logging namespace-aware and removed unused registry placeholders.
- Isolated service negotiation in the dispatcher and added unit coverage for
  supported and unsupported service URIs.
- Preserved TLS failures as a separate disconnect category.
- Added positive and negative unit coverage for Argon2 registrar authentication.

### Verified

- `cargo fmt --check`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo test` (25 tests)
- VPS verification: `hello` transactions have a NULL EPP result code and
  `delivered` status; login/logout responses retain code `1000`.

## 2026-08-14

### Added

- Added `client/run_integration.sh` for running the EPP smoke test against a
  configured server.
- Added EOF coverage for incomplete EPP frame headers and bodies.
- Added a bounded graceful shutdown period, configurable with
  `EPP_SHUTDOWN_GRACE_PERIOD` (10 seconds by default).

### Verified

- `cargo fmt --check`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo test` (20 tests)
- Live TCP/mTLS smoke test against the VPS: greeting, login, hello and logout.

## Earlier changes

- `5979512` — persisted `hello` responses in the transaction log.
- `96b1cbc` — validated the EPP XML namespace.
- `4c2abca` — validated service negotiation during login.
- `9050650` — covered the complete EPP smoke flow.
- `5d36f31` — added a development helper for creating registrars.
- `70a74a5` — added the local Python EPP smoke client.
- `2ae56ed` — bootstrapped the backend, PostgreSQL schema, TLS/mTLS,
  EPP framing, session handling, Admin API and initial migrations.
