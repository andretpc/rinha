use crate::{balance::BalanceDTO, transaction::TransactionDTO};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]

pub struct StatementDTO {
    #[serde(rename(serialize = "saldo"))]
    pub balance: BalanceDTO,

    #[serde(rename(serialize = "ultimas_transacoes"))]
    pub latest_transactions: Vec<TransactionDTO>,
}
