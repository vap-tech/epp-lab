#!/usr/bin/env bash
set -euo pipefail

client_env=${EPP_CLIENT_ENV:-/etc/epp-lab/client.env}
if [[ ! -r "$client_env" ]]; then
    echo "client environment file is not readable: $client_env" >&2
    exit 1
fi

set -a
# shellcheck disable=SC1090
source "$client_env"
set +a

health_url=${ADMIN_HEALTH_URL:-}
if [[ -n "$health_url" ]] && command -v curl >/dev/null 2>&1; then
    health=$(curl --fail --silent --show-error "$health_url")
    [[ "$health" == *'"status":"ok"'* && "$health" == *'"database":"ok"'* ]] || {
        echo "unexpected health response: $health" >&2
        exit 1
    }
elif [[ -n "$health_url" ]]; then
    echo "curl is required for the health check" >&2
    exit 1
fi

script_dir=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)
python3 "$script_dir/epp_smoke.py" "$@"
