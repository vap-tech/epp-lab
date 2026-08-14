# Changelog

## Unreleased

### Changed

- Added an immutable compile-time ExtensionRegistry with validated keys,
  namespace lookup and test-only extension coverage.
- Added the immutable Stage 3 database migration for zones, contact usage
  policies and zone extension assignments with explicit constraints.
- Added the Stage 3 implementation plan and the synchronous Zone domain
  foundation with IDNA names, contact usage policy and suffix resolution.
- Refined XML viewer controls with shared switch-style Raw/Wrap display state,
  per-payload copy icons, tooltips, and copy confirmation state.
- Simplified XML viewer controls with shared Raw and Wrap toggles for request
  and response payloads while keeping Copy local to each payload.
- Replaced Shiki with a minimal Prism XML tokenizer and custom EPP Lab token
  theme to keep production assets small and CSP-compatible.
- Added a reusable EPP XML viewer with pretty/raw modes, wrapping, original
  XML copy, theme-aware Shiki highlighting, and safe formatting fallbacks.
- Added read-only EPP Sessions and Transactions operational views with
  pagination, server-side filters, detail pages, XML inspection and
  session/transaction navigation.
- Added protected `/api/epp/sessions` and `/api/epp/transactions` list/detail
  endpoints with explicit DTOs and safe list payloads without raw XML.
- Added the React admin frontend with HTTPS-served SPA fallback, protected
  routes, server-side admin sessions, CSRF protection, Dashboard and Registrar
  views.
- Added direct Rust TLS for the Admin API on the production HTTPS listener and
  basic security headers including CSP.
- Added frontend authentication smoke tests and local full-build commands.
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
