use crate::app_config::Config;
use crate::app_error::AppError;
use crate::statement::Client;
use crate::transaction::Transaction;
use crate::utils::{Cache, MonaeicacheClient};
use mongodb::bson::doc;
use mongodb::options::{ClientOptions, ServerAddress};
use mongodb::results::InsertOneResult;
use std::sync::Arc;

pub struct AppState {
    pub db: mongodb::Database,
    pub cache: Cache,
    pub monaei_cache: MonaeicacheClient,
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
            monaei_cache: MonaeicacheClient::new().await,
        })
    }

    pub async fn insert_transaction(
        &self,
        transaction: &Transaction,
    ) -> Result<InsertOneResult, mongodb::error::Error> {
        self.db
            .collection::<Transaction>("transactions")
            .insert_one(transaction, None)
            .await
    }

    pub async fn find_client(&self, id: i32) -> Result<Client, AppError> {
        self.db
            .collection::<Client>("clients")
            .find_one(doc! { "_id": id }, None)
            .await?
            .ok_or(AppError::ClientNotFound(id))
    }

    pub async fn update_client(
        &self,
        client: Client,
    ) -> Result<Option<Client>, mongodb::error::Error> {
        self.db
            .collection::<Client>("clients")
            .find_one_and_replace(doc! { "_id": client._id }, client, None)
            .await
    }
}
