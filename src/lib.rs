mod migrations;
mod permissions;

use mongodb::{Client, Collection, Database};

use migrations::{init, scripts};

pub struct MongoDB {
    connection: Client,
    db: Database,
}

impl MongoDB {
    pub async fn new(mongo_uri: &str) -> Self {
        let connection = Client::with_uri_str(mongo_uri)
            .await
            .expect("Failed to init db connection.");
        let db = connection.database("revolt");
        let mongodb = Self { connection, db };
        Self::run_migrations(&mongodb).await;
        mongodb
    }

    async fn run_migrations(&self) {
        let list = self
            .connection
            .list_database_names(None, None)
            .await
            .expect("Failed to fetch database names.");

        if list.iter().position(|x| x == "revolt").is_none() {
            init::create_database(&self.db).await;
        } else {
            scripts::migrate_database(&self.db).await;
        }
    }
}

/*
pub mod entities;
pub mod guards;
pub mod migrations;
pub mod permissions;

pub use entities::*;
pub use guards::*;
pub use permissions::*;
use proc_macro::bridge::client::Client;
 */

#[cfg(test)]
mod tests {
    use super::MongoDB;
    use async_std;
    use env_logger;

    #[test]
    fn it_works() {
        env_logger::init_from_env(env_logger::Env::default().filter_or("RUST_LOG", "info"));
        let mongodb = async_std::task::block_on(MongoDB::new(""));
        assert_eq!(2 + 2, 4);
    }
}
