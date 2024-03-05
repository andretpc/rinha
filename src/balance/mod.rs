use serde::{Deserialize, Serialize};
use std::fmt::Debug;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BalanceDTO {
    pub total: i32,

    #[serde(rename(serialize = "data_extrato"))]
    pub date: String,

    #[serde(rename(serialize = "limite"))]
    pub limit: i32,
}
