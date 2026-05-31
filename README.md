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
$body = @{
  email = "ada@example.com"
  name = "Ada Lovelace"
  metadata = @{
    tier = "gold"
  }
} | ConvertTo-Json -Depth 5

Invoke-RestMethod `
  -Method POST `
  -Uri "http://localhost:8080/customers" `
  -Headers @{ "X-API-Key" = "dodo_live_dev_secret_key" } `
  -ContentType "application/json" `
  -Body $body
```

Create an invoice. Replace `CUSTOMER_ID` with the customer id returned above.

```bash
$invoiceBody = @{
  customer_id = $customerId
  currency = "USD"
  line_items = @(
    @{
      description = "API credits"
      quantity = 2
      unit_amount_cents = 1500
    }
  )
} | ConvertTo-Json -Depth 5

$invoice = Invoke-RestMethod `
  -Method POST `
  -Uri "http://localhost:8080/invoices" `
  -Headers @{ "X-API-Key" = "dodo_live_dev_secret_key" } `
  -ContentType "application/json" `
  -Body $invoiceBody

$invoice
```
Save invoice id automatically:
$invoiceId = $invoice.id

Finalize an invoice:

```bash
$finalizedInvoice = Invoke-RestMethod `
  -Method POST `
  -Uri "http://localhost:8080/invoices/$invoiceId/finalize" `
  -Headers @{ "X-API-Key" = "dodo_live_dev_secret_key" }

$finalizedInvoice
```

Pay an invoice:

```bash
$payBody = @{
  payment_token = "tok_success"
} | ConvertTo-Json -Depth 5

$successPayment = Invoke-RestMethod `
  -Method POST `
  -Uri "http://localhost:8080/invoices/$invoiceId/pay" `
  -Headers @{
    "X-API-Key" = "dodo_live_dev_secret_key"
    "Idempotency-Key" = "payment-success-001"
  } `
  -ContentType "application/json" `
  -Body $payBody

$successPayment
```

Other payment tokens: `tok_insufficient_funds`, `tok_card_declined`, `tok_timeout`, and `tok_network_error`.

Register a webhook endpoint:

```bash
$secondInvoiceBody = @{
  customer_id = $customerId
  currency = "USD"
  line_items = @(
    @{
      description = "Decline demo invoice"
      quantity = 1
      unit_amount_cents = 2500
    }
  )
} | ConvertTo-Json -Depth 5

$secondInvoice = Invoke-RestMethod `
  -Method POST `
  -Uri "http://localhost:8080/invoices" `
  -Headers @{ "X-API-Key" = "dodo_live_dev_secret_key" } `
  -ContentType "application/json" `
  -Body $secondInvoiceBody

$secondInvoiceId = $secondInvoice.id
$secondInvoice
```

List webhook events:

```bash
Invoke-RestMethod `
  -Method GET `
  -Uri "http://localhost:8080/webhooks/events" `
  -Headers @{ "X-API-Key" = "dodo_live_dev_secret_key" }
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
part 1:- https://www.loom.com/share/d3cc84c2463340abaf97e23d9649cc34
paert2 :- https://www.loom.com/share/50db3fe452c94b619454179de4624415
part 3 :- https://www.loom.com/share/382a1fb4b5114f29a398b7b2b7205b9d 
In loom i find that for the free trial, we have 5 min limit, and I also don't have any other video recording app in my system, so I made 3 videos of 5 min
