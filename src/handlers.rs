use crate::{
    app_error::AppError,
    app_state::AppState,
    balance::BalanceDTO,
    statement::{Client, StatementDTO},
    transaction::{Kind, Transaction, TransactionDTO, TransactionResponse},
    utils::{Error, IncrementOpts, Incrementable, Value},
};
use axum::{
    body::Bytes,
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use chrono::Utc;
use speedy::{Readable, Writable};
use std::sync::Arc;
use tokio::spawn;

// pub async fn transaction(
//     app_state: State<Arc<AppState>>,
//     Path(id): Path<i32>,
//     body: Bytes,
// ) -> Result<(StatusCode, Json<TransactionResponse>), AppError> {
//     let body = serde_json::from_slice::<TransactionDTO>(&body)?;

//     app_state.cache.lock(id).await;

//     let bytes = match app_state.cache.get(id).await {
//         Some(bytes) => Ok(bytes),
//         None => match app_state.find_client(id).await {
//             Err(err) => Err(err),
//             Ok(client) => Ok(app_state
//                 .cache
//                 .init(id, &client.write_to_vec().unwrap())
//                 .await),
//         },
//     };

//     if let Err(err) = bytes {
//         app_state.cache.unlock(id).await;

//         return Err(err);
//     }

//     let mut client = Client::read_from_buffer(bytes?).unwrap();

//     let client = client.update(&body);

//     if let Ok(ref client) = client {
//         let bytes = client.write_to_vec().unwrap();

//         app_state.cache.write(id, &bytes).await;
//     }

//     app_state.cache.unlock(id).await;

//     let client = client?.clone();
//     let balance = client.balance;
//     let limit = client.limit;

//     spawn(async move {
//         app_state
//             .insert_transaction(&Transaction::new(client._id, body))
//             .await
//             .unwrap();
//         app_state.update_client(client.to_owned()).await.unwrap();
//     });

//     return Ok((StatusCode::OK, Json(TransactionResponse { limit, balance })));
// }

pub async fn transaction(
    app_state: State<Arc<AppState>>,
    Path(id): Path<i32>,
    body: Bytes,
) -> Result<(StatusCode, Json<TransactionResponse>), AppError> {
    let body = serde_json::from_slice::<TransactionDTO>(&body)?;

    let value = match body.kind {
        Kind::C => body.value,
        Kind::D => -body.value,
    };

    let res: Incrementable = match app_state
        .monaei_cache
        .increment(&format!("{id}"), value, None)
        .await
    {
        Err(Error::NotFound) => match app_state.find_client(id).await {
            Err(err) => Err(err),
            Ok(client) => Ok(app_state
                .monaei_cache
                .increment(
                    &format!("{id}"),
                    value,
                    Some(IncrementOpts {
                        lower_limit: Some(-client.limit),
                        upper_limit: None,
                        create: Some(true),
                    }),
                )
                .await?),
        },
        Err(err) => Err(err.into()),
        Ok(res) => Ok(res),
    }?
    .try_into()
    .unwrap();

    spawn(async move {
        let transaction = Transaction::new(id, body);
        app_state.insert_transaction(&transaction).await.unwrap();
        app_state
            .monaei_cache
            .add_to_set(&format!("{id}-transactions"), transaction, Some(10))
            .await;
    });

    return Ok((
        StatusCode::OK,
        Json(TransactionResponse {
            limit: -res.lower_limit.unwrap(),
            balance: res.value,
        }),
    ));
}

pub async fn statement(
    app_state: State<Arc<AppState>>,
    Path(id): Path<i32>,
) -> Result<(StatusCode, Json<StatementDTO>), AppError> {
    let inc = app_state.monaei_cache.get(&format!("{id}")).await;

    let inc = inc.as_inc().unwrap();

    let value = app_state
        .monaei_cache
        .get(&format!("{id}-transactions"))
        .await;

    let tb = value.as_set();

    let t: Option<Vec<TransactionDTO>> = tb.and_then(|tb| {
        Some(
            tb.iter()
                .map(|value| {
                    let transaction: Transaction = value.try_into().unwrap();

                    transaction.into()
                })
                .collect(),
        )
    });

    Ok((
        StatusCode::OK,
        Json(StatementDTO {
            balance: BalanceDTO {
                total: inc.value,
                date: Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Micros, true),
                limit: -inc.lower_limit.unwrap(),
            },
            latest_transactions: t.unwrap_or(vec![]),
        }),
    ))
}

// pub async fn statement(
//     app_state: State<Arc<AppState>>,
//     Path(id): Path<i32>,
// ) -> Result<(StatusCode, Json<StatementDTO>), AppError> {
//     app_state.cache.lock(id).await;

//     let bytes = match app_state.cache.get(id).await {
//         Some(bytes) => Ok(bytes),
//         None => match app_state.find_client(id).await {
//             Err(err) => Err(err),
//             Ok(client) => {
//                 let bytes: &[u8] = app_state
//                     .cache
//                     .init(id, &client.write_to_vec().unwrap())
//                     .await;

//                 Ok(bytes)
//             }
//         },
//     };

//     app_state.cache.unlock(id).await;

//     let client = Client::read_from_buffer(bytes?).unwrap();

//     Ok((StatusCode::OK, Json(client.into())))
// }
