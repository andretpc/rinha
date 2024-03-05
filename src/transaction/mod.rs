mod deser;
use chrono::prelude::*;
use serde::{Deserialize, Serialize};
use speedy::{Readable, Writable};

use crate::utils::{Error, Value};

#[derive(Serialize, Deserialize, Debug, Clone, Readable, Writable)]
#[serde(rename_all = "lowercase")]

pub enum Kind {
    C,
    D,
}

impl std::fmt::Display for Kind {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Readable, Writable)]

pub struct TransactionDTO {
    #[serde(
        alias = "valor",
        rename(serialize = "valor"),
        deserialize_with = "deser::deserialize_value"
    )]
    pub value: i32,

    #[serde(
        alias = "tipo",
        rename(serialize = "tipo"),
        deserialize_with = "deser::deserialize_kind"
    )]
    pub kind: Kind,

    #[serde(
        alias = "descricao",
        rename(serialize = "descricao"),
        deserialize_with = "deser::deserialize_description"
    )]
    pub description: String,

    #[serde(rename(serialize = "realizada_em"), default = "default_date")]
    pub date: String,
}

fn default_date() -> String {
    Utc::now().to_rfc3339_opts(SecondsFormat::Micros, true)
}

#[derive(Readable, Writable, Serialize, Deserialize, Debug, Clone)]
pub struct Transaction {
    pub client: i32,

    pub value: i32,

    pub kind: Kind,

    pub description: String,

    pub date: String,
}

impl Into<Value> for Transaction {
    fn into(self) -> Value {
        Value::Bin(self.write_to_vec().unwrap())
    }
}

impl TryInto<Transaction> for &Value {
    type Error = Error;

    fn try_into(self) -> Result<Transaction, Self::Error> {
        match self {
            Value::Bin(bin) => Ok(Transaction::read_from_buffer(&bin).unwrap()),
            _ => Err(Error::TypeError),
        }
    }
}

impl Into<TransactionDTO> for Transaction {
    fn into(self) -> TransactionDTO {
        TransactionDTO {
            value: self.value,
            kind: self.kind,
            description: self.description,
            date: self.date,
        }
    }
}

impl Transaction {
    pub fn new(client: i32, transaction_dto: TransactionDTO) -> Self {
        Self {
            client,
            value: transaction_dto.value,
            kind: transaction_dto.kind,
            description: transaction_dto.description,
            date: transaction_dto.date,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TransactionResponse {
    #[serde(rename(serialize = "saldo"))]
    pub balance: i32,

    #[serde(rename(serialize = "limite"))]
    pub limit: i32,
}
