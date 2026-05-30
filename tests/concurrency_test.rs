#[tokio::test]
#[ignore = "requires docker compose postgres and running invoice-service"]
async fn concurrent_payment_allows_only_one_success() {
    // Fire 50 concurrent POST /invoices/{id}/pay requests with unique idempotency keys.
    // Assert one 200 response, no duplicate successful charges, and final invoice state is paid.
}
