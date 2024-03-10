use crate::app_config::Config;
use crate::app_error::AppError;
use crate::client::Client;
use crate::transaction::TransactionDTO;
use crate::utils::{Cache, Semaphore};
use mongodb::bson::doc;
use mongodb::options::{ClientOptions, ServerAddress};
use std::sync::Arc;

pub struct AppState {
    pub db: mongodb::Database,
    pub cache: Cache,
    pub named_semaphore: Semaphore,
}

impl AppState {
    pub async fn new(config: &Config) -> Arc<Self> {
        let opts = ClientOptions::builder()
            .min_pool_size(3)
            .hosts(vec![ServerAddress::parse(&config.mongodb_url).unwrap()])
            .default_database(String::from("rinha"))
            .build();

        let mongodb = mongodb::Client::with_options(opts).unwrap();

        mongodb.warm_connection_pool().await;

        Arc::new(Self {
            db: mongodb.default_database().unwrap(),
            cache: Cache::new(),
            named_semaphore: Semaphore::new(),
        })
    }

    pub async fn update_client_balance(
        &self,
        id: i32,
        transaction: &TransactionDTO,
    ) -> Result<Client, AppError> {
        let key = id.to_string();

        self.named_semaphore.wait(&key).await;

        let result = self._update_client_balance(id, transaction).await;

        self.named_semaphore.release(&key).await;

        result
    }

    async fn _update_client_balance(
        &self,
        id: i32,
        transaction: &TransactionDTO,
    ) -> Result<Client, AppError> {
        let key = id.to_string();

        let mut client = match self.cache.get(&key).await {
            None => self
                .db
                .collection::<Client>("clients")
                .find_one(doc! { "_id": id }, None)
                .await?
                .ok_or(AppError::ClientNotFound(id)),
            Some(client) => Ok(client),
        }?;

        client.update(transaction)?;

        self.cache.insert(&key, &client).await;

        Ok(client)
    }

    pub async fn get_client(&self, id: i32) -> Result<Client, AppError> {
        let key = format!("{id}");

        self.named_semaphore.wait(&key).await;

        let result = self._get_client(id).await;

        self.named_semaphore.release(&key).await;

        result
    }

    async fn _get_client(&self, id: i32) -> Result<Client, AppError> {
        let key = format!("{id}");

        match self.cache.get(&key).await {
            None => self
                .db
                .collection::<Client>("clients")
                .find_one(doc! { "_id": id }, None)
                .await?
                .ok_or(AppError::ClientNotFound(id)),
            Some(client) => {
                self.cache.insert(&key, &client).await;
                Ok(client)
            }
        }
    }
}
