use super::Kind;
use serde::{Deserialize, Deserializer};

pub fn deserialize_value<'de, D>(deserializer: D) -> Result<i32, D::Error>
where
    D: Deserializer<'de>,
{
    if let Ok(value) = i32::deserialize(deserializer) {
        if value > 0 {
            return Ok(value);
        }
    }

    Err(serde::de::Error::custom(
        "Campo 'valor' deve ser um inteiro positivo.",
    ))
}

pub fn deserialize_description<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    if let Ok(value) = String::deserialize(deserializer) {
        if (1..=10).contains(&value.len()) {
            return Ok(value);
        }
    }

    Err(serde::de::Error::custom(
        "Campo 'descricao' deve ser uma string entre 1 e 10 caracteres.",
    ))
}

pub fn deserialize_kind<'de, D>(deserializer: D) -> Result<Kind, D::Error>
where
    D: Deserializer<'de>,
{
    match Kind::deserialize(deserializer) {
        Ok(value) => Ok(value),
        Err(_err) => Err(serde::de::Error::custom(
            "Campo 'tipo' deve ser 'c' para crédito ou 'd' para débito.",
        )),
    }
}
