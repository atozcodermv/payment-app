use crate::{domain::value_objects::invoice_state::InvoiceState, shared::errors::AppError};

pub fn ensure_transition(current: &str, next: InvoiceState) -> Result<(), AppError> {
    let current = InvoiceState::parse(current);
    if current.can_transition_to(&next) {
        Ok(())
    } else {
        Err(AppError::Conflict("invalid_state_transition", "Invalid invoice state transition"))
    }
}
