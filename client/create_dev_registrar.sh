#!/usr/bin/env bash
set -euo pipefail

: "${EPP_PASSWORD:?set EPP_PASSWORD before running this script}"
admin_url="${ADMIN_URL:-http://127.0.0.1:8080}"
handle="${EPP_HANDLE:-demo}"
name="${EPP_REGISTRAR_NAME:-Demo Registrar}"
client_id="${EPP_CLIENT_ID:-DEMO-1}"

python3 - "$admin_url" "$handle" "$name" "$client_id" "$EPP_PASSWORD" <<'PY'
import json
import sys
import urllib.request

url, handle, name, client_id, password = sys.argv[1:]
payload = json.dumps({
    "handle": handle,
    "name": name,
    "client_id": client_id,
    "password": password,
}).encode()
request = urllib.request.Request(
    f"{url}/api/registrars",
    data=payload,
    headers={"content-type": "application/json"},
    method="POST",
)
with urllib.request.urlopen(request) as response:
    print(response.read().decode())
PY
