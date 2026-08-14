# Stage 3 — Zone Foundation implementation plan

## Iteration 1 — Domain foundation

- Add `domain::zone` with `ZoneId`, `ZoneName`, `ZoneStatus`.
- Add `ContactRequirement` and `ContactUsagePolicy`.
- Add canonical ASCII/IDNA validation and Unicode display name handling.
- Add synchronous longest-label-suffix matching.
- Cover valid/invalid names, multi-label zones, IDN and contactless policy.

## Iteration 2 — Persistence

- Add immutable migrations for `zones`, `zone_contact_policies` and
  `zone_extensions`.
- Add explicit PostgreSQL storage operations and atomic zone creation.
- Add zone lookup query and PostgreSQL integration coverage.

## Iteration 3 — Extension registry and EPP capabilities

- Add `ExtensionKey`, `ExtensionDefinition` and immutable compile-time registry.
- Separate registered extensions from zone-enabled assignments.
- Derive advertised namespaces from active zones.
- Preserve empty-registry greeting/login behavior and add regression tests.

## Iteration 4 — Admin API

- Add authenticated and CSRF-protected zone and extension endpoints.
- Add explicit application services, DTOs, validation errors and schemas.
- Add backend API tests.

## Iteration 5 — Admin UI and final verification

- Add Zones navigation, list, create and detail views.
- Add contact policy and extension controls using shadcn components.
- Add frontend tests, update documentation/changelog and run full checks.

## Invariants

- Contacts remain zone-neutral; no `contacts.zone_id`.
- Domain code stays synchronous and persistence-independent.
- Zone rename and deletion are not implemented.
- No generic JSON extension configuration, Redis/cache or fake production
  extensions/zones.
- Existing EPP, Sessions, Transactions and XML Viewer behavior must remain
  intact.
