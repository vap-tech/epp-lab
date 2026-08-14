check:
    cd backend && cargo fmt --check && cargo clippy --all-targets --all-features -- -D warnings && cargo test

run:
    cd backend && cargo run

db-up:
    docker compose up -d postgres

db-down:
    docker compose stop postgres

test-with-db:
    #!/usr/bin/env bash
    set -euo pipefail
    trap 'docker compose stop postgres' EXIT
    docker compose up -d postgres
    until docker compose exec -T postgres pg_isready -U epp_lab -d epp_lab >/dev/null 2>&1; do sleep 1; done
    DATABASE_URL=postgres://epp_lab:epp_lab@localhost:5433/epp_lab just check
