mod migrations;
use crate::entities::User;
use crate::util::result::*;
use crate::Queries as DatabaseTrait;
use migrations::{init, scripts};
use mongodb::{
    bson::{doc, from_document},
    error::Result as MongoResult,
    options::{Collation, FindOneOptions, FindOptions},
    Client, Collection, Database,
};

use futures::{StreamExt, TryStreamExt};
use rocket::async_trait;
use rocket::http::ext::IntoCollection;

pub struct MongoDB {
    connection: Client,
    revolt: Database,
}

impl MongoDB {
    pub async fn new(mongo_uri: &str) -> Self {
        let connection = Client::with_uri_str(mongo_uri)
            .await
            .expect("Failed to init db connection.");
        let db = connection.database("revolt");
        let mongodb = Self {
            connection,
            revolt: db,
        };
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
            init::create_database(&self.revolt).await;
        } else {
            scripts::migrate_database(&self.revolt).await;
        }
    }
}

#[async_trait]
impl DatabaseTrait for MongoDB {
    async fn get_user_by_id(&self, id: &str) -> Result<User> {
        if let Some(doc) = self
            .revolt
            .collection("users")
            .find_one(
                doc! {
                    "_id": &id
                },
                None,
            )
            .await
            .map_err(|_| Error::DatabaseError {
                operation: "find_one",
                with: "server",
            })?
        {
            Ok(from_document(doc).expect("schema should match"))
        } else {
            Err(Error::NotFound)
        }
    }

    async fn get_user_by_username(&self, username: &str) -> Result<User> {
        if let Some(doc) = self
            .revolt
            .collection("users")
            .find_one(
                doc! {
                    "username": username
                },
                FindOneOptions::builder()
                    .collation(Collation::builder().locale("en").strength(2).build())
                    .build(),
            )
            .await
            .map_err(|_| Error::DatabaseError {
                operation: "find_one",
                with: "user",
            })?
        {
            Ok(from_document(doc).expect("schema should match"))
        } else {
            Err(Error::NotFound)
        }
    }

    async fn get_users(&self, user_ids: Vec<&str>) -> Result<Vec<User>> {
        let mut cursor = self.revolt.collection("users")
            .find(
                doc! {
                    "_id": {
                        "$in": user_ids
                    }
                },
                FindOptions::builder()
                    .projection(
                        doc! { "_id": 1, "username": 1, "avatar": 1, "badges": 1, "status": 1, "flags": 1, "bot": 1 },
                    )
                    .build(),
            )
            .await
            .map_err(|_| Error::DatabaseError {
                operation: "find",
                with: "users",
            })?;
        let mut users = vec![];
        while let Some(result) = cursor.next().await {
            if let Ok(doc) = result {
                let user: User = from_document(doc).map_err(|_| Error::DatabaseError {
                    operation: "from_document",
                    with: "user",
                })?;
                users.push(user);
            }
        }
        Ok(users)
    }
}
