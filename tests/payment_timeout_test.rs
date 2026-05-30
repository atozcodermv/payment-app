#[tokio::test]
#[ignore = "requires docker compose postgres and running invoice-service"]
async fn timeout_returns_processing_and_recovery_marks_paid() {
    // Pay with tok_timeout.
    // Assert 202 processing, invoice remains open, then recovery worker later marks it paid.
}
