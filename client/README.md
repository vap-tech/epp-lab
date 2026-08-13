# EPP test clients

These scripts run locally and are not deployed to the VPS.

Create a development registrar through the Admin API:

```bash
export EPP_PASSWORD=demo-secret
./client/create_dev_registrar.sh
```

Example:

```bash
export EPP_CLIENT_ID=DEMO-1
export EPP_PASSWORD=demo-secret
export EPP_SERVER_CA=/path/to/server-ca-or-system-ca.pem
export EPP_CLIENT_CERT=/path/to/client.crt
export EPP_CLIENT_KEY=/path/to/client.key
python client/epp_smoke.py
```

The script performs:

```text
TLS/mTLS → greeting → login → logout
```
