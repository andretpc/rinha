use axum::{http::StatusCode, response::IntoResponse};
use std::array::TryFromSliceError;

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("Cliente {0} não encontrado")]
    ClientNotFound(i32),

    #[error("Saldo insuficiente")]
    InsufficientBalanceError,

    #[error(transparent)]
    MongoError(#[from] mongodb::error::Error),

    #[error(transparent)]
    DeError(#[from] serde_json::error::Error),

    #[error(transparent)]
    TryFromSliceError(#[from] TryFromSliceError),

    #[error(transparent)]
    CacheError(#[from] crate::utils::Error),
}

impl IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        let message = self.to_string();

        match &self {
            AppError::ClientNotFound(_) => (StatusCode::NOT_FOUND, message).into_response(),

            AppError::InsufficientBalanceError => {
                (StatusCode::UNPROCESSABLE_ENTITY, message).into_response()
            }

            AppError::MongoError(_) => (StatusCode::INTERNAL_SERVER_ERROR, message).into_response(),

            AppError::DeError(_) => (StatusCode::UNPROCESSABLE_ENTITY, message).into_response(),

            AppError::TryFromSliceError(_) => {
                (StatusCode::INTERNAL_SERVER_ERROR, message).into_response()
            }

            AppError::CacheError(crate::utils::Error::LimitReached) => {
                (StatusCode::UNPROCESSABLE_ENTITY, message).into_response()
            }

            AppError::CacheError(_) => (StatusCode::UNPROCESSABLE_ENTITY, message).into_response(),
        }
    }
}
