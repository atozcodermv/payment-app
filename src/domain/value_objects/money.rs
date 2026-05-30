use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Money {
    pub currency: String,
    pub amount_cents: i32,
}
