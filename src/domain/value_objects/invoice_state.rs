use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum InvoiceState {
    Draft,
    Open,
    Paid,
    Void,
    Uncollectible,
}

impl InvoiceState {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Draft => "draft",
            Self::Open => "open",
            Self::Paid => "paid",
            Self::Void => "void",
            Self::Uncollectible => "uncollectible",
        }
    }

    pub fn parse(value: &str) -> Self {
        match value {
            "open" => Self::Open,
            "paid" => Self::Paid,
            "void" => Self::Void,
            "uncollectible" => Self::Uncollectible,
            _ => Self::Draft,
        }
    }

    pub fn can_transition_to(&self, next: &Self) -> bool {
        matches!(
            (self, next),
            (Self::Draft, Self::Open)
                | (Self::Draft, Self::Void)
                | (Self::Open, Self::Paid)
                | (Self::Open, Self::Void)
                | (Self::Open, Self::Uncollectible)
        )
    }
}
