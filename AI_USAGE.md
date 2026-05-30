# AI Usage


## AI Tools Used

### ChatGPT

I used ChatGPT primarily as a learning and productivity aid during development.

Specific uses:

* Generated the initial Rust project boilerplate and folder structure to speed up setup.
* Assisted with SQL migration syntax and schema validation.
* Reviewed parts of the DESIGN.md document for completeness against the assignment requirements.
* Assisted in refining documentation and improving the clarity of explanations.



## Decisions I Made Independently

### 1. Using Row-Level Locking For Payment Concurrency

**AI Suggestion:**
Some suggestions included optimistic locking or advisory locks for handling concurrent payment requests.

**My Decision:**
I chose PostgreSQL row-level locking using:

```sql
SELECT * FROM invoices
WHERE id = $1
FOR UPDATE;
```

**Reason:**

Payments are high-value operations where correctness is more important than throughput. Row-level locking guarantees that only one payment transaction can process an invoice at a time and eliminates double-charge risks with simpler operational behavior.

---

### 2. Keeping Payment State Outside Invoice State

**AI Suggestion:**
A common suggestion was introducing an additional invoice state such as `payment_pending`.

**My Decision:**
I kept payment processing state inside the `payment_attempts` table instead of adding another invoice state.

**Reason:**

An invoice should remain `open` until payment success is confirmed. Separating payment attempt status from invoice status avoids invoices becoming stuck in an intermediate processing state and keeps the invoice state machine simpler.

---

### 3. Using Transactional Outbox For Webhooks

**AI Suggestion:**
Some generated implementations attempted to send webhooks directly during API processing.

**My Decision:**
I implemented a Transactional Outbox pattern using `webhook_outbox` and asynchronous workers.

**Reason:**

Webhook delivery should never impact API latency or payment throughput. The outbox pattern guarantees eventual delivery while ensuring invoice state changes and webhook event creation occur atomically.

---

## One Thing AI Got Wrong

One AI-generated suggestion was to mark a payment attempt as `failed` immediately when the PSP request timed out.

I did not use this approach.

A timeout does not necessarily mean the PSP failed to charge the customer. The PSP may still complete the charge after the client timeout occurs.

Instead, I introduced a `processing` / `unknown` state and a recovery worker that periodically reconciles payment attempts with the PSP. This approach prevents incorrect failure reporting and avoids duplicate charges.

I verified this behavior against the assignment requirements and designed the payment flow to preserve correctness under timeout and network failure scenarios.
