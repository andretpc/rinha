use crate::{
    app_error::AppError,
    app_state::AppState,
    client::Client,
    statement::StatementDTO,
    transaction::{Transaction, TransactionDTO, TransactionResponse},
};
use axum::{
    body::Bytes,
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use mongodb::bson::doc;
use std::sync::Arc;
use tokio::spawn;

pub async fn transaction(
    app_state: State<Arc<AppState>>,
    Path(id): Path<i32>,
    body: Bytes,
) -> Result<(StatusCode, Json<TransactionResponse>), AppError> {
    let transaction_dto = serde_json::from_slice::<TransactionDTO>(&body)?;

    let client = app_state
        .update_client_balance(id, &transaction_dto)
        .await?;

    let client_clone = client.clone();

    spawn(async move {
        app_state
            .db
            .collection::<Transaction>("transactions")
            .insert_one(Transaction::new(id, transaction_dto), None)
            .await?;

        app_state
            .db
            .collection::<Client>("clients")
            .find_one_and_replace(doc! { "_id": client_clone._id }, &client_clone, None)
            .await?;

        Ok::<(), AppError>(())
    });

    return Ok((StatusCode::OK, Json(client.into())));
}

pub async fn statement(
    app_state: State<Arc<AppState>>,
    Path(id): Path<i32>,
) -> Result<(StatusCode, Json<StatementDTO>), AppError> {
    let client = app_state.get_client(id).await?;

    Ok((StatusCode::OK, Json(client.into())))
}
