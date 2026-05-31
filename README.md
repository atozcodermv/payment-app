# Dodo Payments Backend Assignment

Rust/Axum invoice and payment API for the Dodo backend hiring assignment. It includes PostgreSQL persistence, SQLx migrations, API key authentication, idempotency, row-level payment locking, a mock PSP, webhook outbox delivery, background recovery workers, Docker Compose, Swagger UI, and assignment documentation.

## Run

```bash
cp .env.example .env
docker compose up --build
```

The invoice API runs on `http://localhost:8080`, the mock PSP runs on `http://localhost:8081`, and Swagger UI is available at `http://localhost:8080/swagger`.

The seeded development API key is:

```text
dodo_live_dev_secret_key
```

## Curl Examples

Create a customer:

```bash
curl -X POST http://localhost:8080/customers \
  -H "X-API-Key: dodo_live_dev_secret_key" \
  -H "Content-Type: application/json" \
  -d '{"email":"ada@example.com","name":"Ada Lovelace","metadata":{"tier":"gold"}}'
```

Create an invoice. Replace `CUSTOMER_ID` with the customer id returned above.

```bash
curl -X POST http://localhost:8080/invoices \
  -H "X-API-Key: dodo_live_dev_secret_key" \
  -H "Content-Type: application/json" \
  -d '{"customer_id":"CUSTOMER_ID","currency":"USD","line_items":[{"description":"API credits","quantity":2,"unit_amount_cents":1500}]}'
```

Finalize an invoice:

```bash
curl -X POST http://localhost:8080/invoices/INVOICE_ID/finalize \
  -H "X-API-Key: dodo_live_dev_secret_key"
```

Pay an invoice:

```bash
curl -X POST http://localhost:8080/invoices/INVOICE_ID/pay \
  -H "X-API-Key: dodo_live_dev_secret_key" \
  -H "Idempotency-Key: demo-pay-001" \
  -H "Content-Type: application/json" \
  -d '{"payment_token":"tok_success"}'
```

Other payment tokens: `tok_insufficient_funds`, `tok_card_declined`, `tok_timeout`, and `tok_network_error`.

Register a webhook endpoint:

```bash
curl -X POST http://localhost:8080/webhooks \
  -H "X-API-Key: dodo_live_dev_secret_key" \
  -H "Content-Type: application/json" \
  -d '{"url":"https://example.com/webhooks/dodo"}'
```

List webhook events:

```bash
curl http://localhost:8080/webhooks/events \
  -H "X-API-Key: dodo_live_dev_secret_key"
```

Create a new API key:

```bash
curl -X POST http://localhost:8080/api-keys \
  -H "Content-Type: application/json" \
  -d '{}'
```

## Architecture Overview

The code follows a clean, hexagonal layout. `domain` contains entities, value objects, repository traits, and domain services. `application` contains DTOs and use cases. `infrastructure` contains SQLx PostgreSQL repositories, Argon2 API key hashing, PSP integration, webhook dispatching, and workers. `interfaces` contains Axum routes, handlers, middleware, and OpenAPI wiring.

Payment processing uses `SELECT ... FOR UPDATE` on the invoice row before deciding whether a charge is allowed. Successful invoice updates and webhook outbox inserts happen in the same transaction. Idempotent payment requests cache the response by `(business_id, Idempotency-Key)` and reject key reuse with a different request hash.

## Tests

The `tests/` directory documents the required integration coverage for concurrency, idempotency, timeout recovery, webhook retry, and dead letters. Run them against a database with:

```bash
cargo test
```

## Demo Video

Demo video placeholder: add a short screen recording showing Docker Compose startup, Swagger, customer creation, invoice creation, finalization, payment, and webhook event listing.
