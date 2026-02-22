# wuminapp API SPEC (MVP)

## Base
- Base URL: `http://<host>:8787`
- Prefix: `/api/v1`
- Response envelope:
  - success: `{ "code": 0, "message": "ok", "data": ... }`
  - error: `{ "code": <non-zero>, "message": "...", "trace_id": "..." }`

## 1) Health
- `GET /api/v1/health`
- Purpose: service health probe
- Response example:
```json
{
  "code": 0,
  "message": "ok",
  "data": {
    "service": "wuminapp-backend",
    "version": "0.0.1",
    "status": "UP"
  }
}
```

## 2) SFID Bind (placeholder)
- `POST /api/v1/auth/sfid/bind`
- Purpose: bind account with SFID credential
- Body (draft):
```json
{
  "account": "5F...",
  "sfid_code": "CN-...",
  "credential_nonce": "...",
  "signature": "0x..."
}
```

## 3) Wallet Balance (placeholder)
- `GET /api/v1/wallet/balance?account=<address>`
- Purpose: get account balance and token units

## 4) Transaction Create (placeholder)
- `POST /api/v1/tx/create`
- Purpose: create or relay signed tx

## 5) Transaction History (placeholder)
- `GET /api/v1/tx/history?account=<address>&page=1&page_size=20`
- Purpose: paginated tx history (onchain/offchain merged view)
