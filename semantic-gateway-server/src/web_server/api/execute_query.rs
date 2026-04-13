use actix_web::{HttpResponse, post};

#[post("/query/execute")]
#[tracing::instrument(name = "Execute query")]
pub(crate) async fn execute_query() -> HttpResponse {
    HttpResponse::Ok().finish()
}
