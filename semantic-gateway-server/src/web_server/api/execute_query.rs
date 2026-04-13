use actix_web::{HttpResponse, post, web};

#[post("/query/execute")]
#[tracing::instrument(name = "Execute query")]
pub(crate) async fn execute_query(request: web::Json<QueryRequest>) -> HttpResponse {
    HttpResponse::Ok().finish()
}

#[derive(serde::Deserialize, Debug)]
struct QueryRequest {
    metrics: Vec<String>,
    dimensions: Vec<String>,
}
