use crate::application::dto::*;
use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(
    components(schemas(
        CreateApiKeyRequest, CreateApiKeyResponse, CreateCustomerRequest, CustomerResponse,
        CreateLineItemRequest, CreateInvoiceRequest, LineItemResponse, InvoiceResponse,
        PayInvoiceRequest, CreateWebhookRequest, WebhookResponse
    )),
    tags((name = "dodo-payments", description = "Dodo Payments assignment API"))
)]
pub struct ApiDoc;
