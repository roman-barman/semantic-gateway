use actix_web::{HttpResponse, get};

#[get("/health")]
#[tracing::instrument(name = "Health check")]
pub(crate) async fn health() -> HttpResponse {
    HttpResponse::Ok().finish()
}
