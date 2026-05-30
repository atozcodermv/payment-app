#[tokio::test]
#[ignore = "requires docker compose postgres and running invoice-service"]
async fn same_key_and_body_returns_cached_response() {
    // Send the same payment request twice with the same Idempotency-Key.
    // Assert identical response bodies and a single payment_attempt row.
}
