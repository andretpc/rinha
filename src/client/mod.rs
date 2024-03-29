use chrono::{SecondsFormat, Utc};
use serde::{Deserialize, Serialize};
use speedy::{Readable, Writable};

use crate::{
    app_error::AppError,
    balance::BalanceDTO,
    statement::StatementDTO,
    transaction::{Kind, TransactionDTO, TransactionResponse},
};

#[derive(Serialize, Deserialize, Debug, Clone, Readable, Writable)]
pub struct Client {
    pub _id: i32,

    pub balance: i32,

    pub limit: i32,

    pub latest_transactions: Vec<TransactionDTO>,
}

impl Into<StatementDTO> for Client {
    fn into(self) -> StatementDTO {
        StatementDTO {
            balance: BalanceDTO {
                total: self.balance,
                date: Utc::now().to_rfc3339_opts(SecondsFormat::Micros, true),
                limit: self.limit,
            },
            latest_transactions: self.latest_transactions,
        }
    }
}

impl Into<TransactionResponse> for Client {
    fn into(self) -> TransactionResponse {
        TransactionResponse {
            limit: self.limit,
            balance: self.balance,
        }
    }
}

impl Client {
    pub fn update(&mut self, transaction: &TransactionDTO) -> Result<&mut Self, AppError> {
        let value = match transaction.kind {
            Kind::C => transaction.value,
            Kind::D => -transaction.value,
        };

        self.balance += value;

        if self.balance < -self.limit {
            return Err(AppError::InsufficientBalanceError);
        };

        self.latest_transactions.insert(
            0,
            TransactionDTO {
                value: transaction.value,
                kind: transaction.kind.clone(),
                description: transaction.description.clone(),
                date: transaction.date.clone(),
            },
        );

        if self.latest_transactions.len() > 10 {
            self.latest_transactions.pop();
        };

        Ok(self)
    }
}
