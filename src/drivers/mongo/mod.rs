mod migrations;
use crate::entities::{BannedUser, Bot, User};
use crate::util::result::*;
use crate::Queries;
use migrations::{init, scripts};
use mongodb::{
    bson::{doc, from_document, Document},
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
impl Queries for MongoDB {
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

    async fn get_users_as_banned_users(&self, user_ids: Vec<&str>) -> Result<Vec<BannedUser>> {
        let mut cursor = self
            .revolt
            .collection("users")
            .find(
                doc! {
                    "_id": {
                        "$in": user_ids
                    }
                },
                FindOptions::builder()
                    .projection(doc! {
                        "username": 1,
                        "avatar": 1
                    })
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
                if let Ok(user) = from_document::<BannedUser>(doc) {
                    users.push(user);
                }
            }
        }
        Ok(users)
    }

    async fn get_bot_users_owned_by_user_id(&self, id: &str) -> Result<Vec<User>> {
        Ok(self
            .revolt
            .collection("users")
            .find(
                doc! {
                    "bot.owner": id
                },
                None,
            )
            .await
            .map_err(|_| Error::DatabaseError {
                with: "users",
                operation: "find",
            })?
            .filter_map(async move |s| s.ok())
            .collect::<Vec<Document>>()
            .await
            .into_iter()
            .filter_map(|x| from_document(x).ok())
            .collect::<Vec<User>>())
    }

    async fn get_bots_owned_by_user_id(&self, id: &str) -> Result<Vec<Bot>> {
        Ok(self
            .revolt
            .collection("bots")
            .find(
                doc! {
                    "owner": id
                },
                None,
            )
            .await
            .map_err(|_| Error::DatabaseError {
                with: "bots",
                operation: "find",
            })?
            .filter_map(async move |s| s.ok())
            .collect::<Vec<Document>>()
            .await
            .into_iter()
            .filter_map(|x| from_document(x).ok())
            .collect::<Vec<Bot>>())
    }

    async fn get_mutual_friends_ids(
        &self,
        user_id_a: &str,
        user_id_b: &str,
    ) -> Result<Vec<String>> {
        Ok(self
            .revolt
            .collection("users")
            .find(
                doc! {
                    "$and": [
                        { "relations": { "$elemMatch": { "_id": user_id_a, "status": "Friend" } } },
                        { "relations": { "$elemMatch": { "_id": user_id_b, "status": "Friend" } } }
                    ]
                },
                FindOptions::builder().projection(doc! { "_id": 1 }).build(),
            )
            .await
            .map_err(|_| Error::DatabaseError {
                operation: "find",
                with: "users",
            })?
            .filter_map(async move |s| s.ok())
            .collect::<Vec<Document>>()
            .await
            .into_iter()
            .filter_map(|x| x.get_str("_id").ok().map(|x| x.to_string()))
            .collect())
    }

    async fn add_user(&self, id: &str, username: &str) -> Result<()> {
        self.revolt
            .collection("users")
            .insert_one(
                doc! {
                    "_id": id,
                    "username": username
                },
                None,
            )
            .await
            .map_err(|_| Error::DatabaseError {
                operation: "insert_one",
                with: "user",
            })?;
        Ok(())
    }

    async fn add_bot_user(&self, id: &str, username: &str, owner_id: &str) -> Result<()> {
        self.revolt
            .collection("users")
            .insert_one(
                doc! {
                    "_id": id,
                    "username": username,
                    "bot": {
                        "owner": owner_id
                    }
                },
                None,
            )
            .await
            .map_err(|_| Error::DatabaseError {
                operation: "insert_one",
                with: "user",
            })?;
        Ok(())
    }
}
