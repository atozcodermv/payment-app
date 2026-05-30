#[test]
fn retry_schedule_matches_assignment() {
    assert_eq!(dodo_payments::infrastructure::webhooks::retry_worker::retry_delay_minutes(0), Some(1));
    assert_eq!(dodo_payments::infrastructure::webhooks::retry_worker::retry_delay_minutes(1), Some(10));
    assert_eq!(dodo_payments::infrastructure::webhooks::retry_worker::retry_delay_minutes(2), Some(60));
    assert_eq!(dodo_payments::infrastructure::webhooks::retry_worker::retry_delay_minutes(3), Some(360));
    assert_eq!(dodo_payments::infrastructure::webhooks::retry_worker::retry_delay_minutes(4), Some(720));
    assert_eq!(dodo_payments::infrastructure::webhooks::retry_worker::retry_delay_minutes(5), None);
}
