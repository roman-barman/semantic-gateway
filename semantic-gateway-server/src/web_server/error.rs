use actix_web::http::StatusCode;
use actix_web::http::header::ContentType;
use actix_web::{HttpResponse, ResponseError};

#[derive(Debug, thiserror::Error)]
pub(crate) enum ServerError {
    #[error("{0}")]
    UnprocessableEntity(String),
    #[error("internal server error")]
    InternalServerError(String),
}

impl ResponseError for ServerError {
    fn status_code(&self) -> StatusCode {
        match self {
            ServerError::UnprocessableEntity(_) => StatusCode::UNPROCESSABLE_ENTITY,
            ServerError::InternalServerError(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn error_response(&self) -> HttpResponse {
        match self.status_code() {
            StatusCode::UNPROCESSABLE_ENTITY => tracing_log::log::warn!("{:?}", self),
            _ => tracing_log::log::error!("{:?}", self),
        };
        create_error_response(self.status_code(), self.to_string())
    }
}

pub(crate) fn create_error_response(status_code: StatusCode, error: String) -> HttpResponse {
    HttpResponse::build(status_code)
        .insert_header(ContentType::json())
        .json(ErrorResponse { error })
}

#[derive(Debug, serde::Serialize)]
struct ErrorResponse {
    error: String,
}
