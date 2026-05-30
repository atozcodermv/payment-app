pub fn retry_delay_minutes(attempt: i32) -> Option<i64> {
    match attempt {
        0 => Some(1),
        1 => Some(10),
        2 => Some(60),
        3 => Some(360),
        4 => Some(720),
        _ => None,
    }
}
