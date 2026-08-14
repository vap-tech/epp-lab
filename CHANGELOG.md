# Changelog

## Unreleased

### Changed

- Added an AES-256-GCM SecretCipher boundary with random nonces and
  authentication-failure handling for future encrypted Contact authInfo.
- Added optional `CONTACT_AUTHINFO_KEY_HEX` configuration validation and
  injected the cipher into application state without persisting the key.
- Added the immutable Contact persistence migration with normalized postal,
  phone, status and disclosure tables; `authInfo` is stored only as ciphertext.
- Added the initial Contact repository identity boundary for future application
  commands.
- Added PostgreSQL coverage proving Contact identity round-trips through the
  repository with ciphertext storage and registrar foreign-key enforcement.
- Split EPP response wire XML from persisted XML so secret-bearing commands
  can safely redact transaction history without changing the registrar reply.
- Added aggregate-level Contact validation for required authInfo, postal/voice
  data, status combinations and timestamp ordering.
- Added protocol-level Contact command recognition for check/create/info/update/
  delete; execution remains explicitly unsupported until application commands
  are implemented.
- Added Contact check command parsing with namespace-aware IDs and malformed
  empty-check rejection.
- Added the application/storage availability boundary for Contact check,
  including PostgreSQL coverage for existing and unknown ROIDs.
- Connected authenticated `contact:check` to a real EPP response with
  per-ROID availability; other Contact commands remain explicitly pending.
- Added a dedicated Contact create XML DTO with required-field extraction,
  keeping protocol data separate from the Contact domain aggregate.
- Added full Contact create parser coverage for required and optional postal,
  phone and authInfo fields.
- Added application mapping for Contact create that validates the aggregate and
  encrypts authInfo before it can reach persistence.
- Added atomic Contact aggregate persistence for postal info, phones, statuses
  and disclosure fields alongside the encrypted identity row.
- Connected authenticated `contact:create` to application mapping, encrypted
  persistence and the EPP create response; missing encryption configuration is
  rejected without storing the request secret.
- Extended the external Python EPP client with an explicit `--create-contact`
  mode and verified a live Contact create against the VPS.
- Added read-only authenticated Contacts API list/detail endpoints that expose
  summaries only and never return authInfo or ciphertext.
- Added the initial RFC 5733 Contact domain foundation with validated
  identities, postal data, phone/email values, statuses and disclosure types.
- Added PostgreSQL-backed Admin API integration tests and a dedicated test
  harness that includes ignored database tests and stops PostgreSQL afterward.
- Added PostgreSQL-backed Zone persistence integration coverage for atomic
  creation, contact policy restoration, status updates and duplicate names.
- Added PostgreSQL-backed Admin API regression coverage for Zone
  authorization, security headers and API-vs-SPA 404 routing.
- Moved Zone creation, status changes and contact policy mutations behind the
  application boundary, keeping HTTP handlers focused on adapter concerns.
- Connected EPP greeting and login service negotiation to the active Zone
  extension assignments through an application-level capability boundary,
  while retaining configured extension URIs as a compatibility fallback.
- Kept capability loading fail-closed: PostgreSQL errors are no longer hidden
  by advertising stale configured extension URIs.
- Documented the Zone resolution and EPP capability boundaries in the
  architecture guide.
- Connected the Zone Extensions section to the real extension catalog and
  assignment APIs, with an honest empty production registry state.
- Added Zone detail UI with General status, Contact Usage policy controls and
  Extensions summary, linked from the Zones list.
- Added the Zones create dialog with real API mutation, CSRF handling,
  validation/duplicate feedback and list refresh.
- Added the authenticated Zones frontend list and sidebar navigation backed by
  the real Zone API.
- Added authenticated extension catalog and zone assignment endpoints; the
  production catalog is currently empty and unknown extensions are rejected.
- Added CSRF-protected contact policy updates at
  `PATCH /api/zones/:id/contact-policy` with explicit role enum validation.
- Added authenticated Zone detail and status update endpoints; Zone rename and
  deletion remain unsupported.
- Added CSRF-protected `POST /api/zones` with IDNA validation, explicit
  defaults and atomic Zone/contact-policy creation.
- Added the authenticated read-only `GET /api/zones` endpoint with explicit
  Zone and contact policy DTOs.
- Added SQLx persistence operations for listing and toggling zone extension
  assignments with deterministic ordering and upsert semantics.
- Added deterministic advertised-extension calculation from registered
  extensions, enabled assignments and active zones.
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
