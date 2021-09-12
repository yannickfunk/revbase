mod migrations;
use crate::entities::{BannedUser, Bot, File, Invite, Subscription, User};
use crate::util::result::*;
use crate::Queries;
use migrations::{init, scripts};
use mongodb::{
    bson::{doc, from_document, to_document, Document},
    error::Result as MongoResult,
    options::{Collation, FindOneOptions, FindOptions, UpdateOptions},
    Client, Collection, Database,
};

use futures::{StreamExt, TryStreamExt};
use rocket::async_trait;
use rocket::http::ext::IntoCollection;
use web_push::SubscriptionInfo;

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

    async fn get_user_id_by_bot_token(&self, token: &str) -> Result<String> {
        let maybe_bot_doc = self
            .revolt
            .collection("bots")
            .find_one(
                doc! {
                    "token": token
                },
                None,
            )
            .await
            .map_err(|_| Error::DatabaseError {
                operation: "find_one",
                with: "user",
            })?;
        if let Some(doc) = maybe_bot_doc {
            Ok(doc.get_str("_id").unwrap().to_string())
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

    async fn delete_user(&self, id: &str) -> Result<()> {
        let username = format!("Deleted User {}", id);
        self.revolt
            .collection("users")
            .update_one(
                doc! {
                    "_id": id
                },
                doc! {
                    "$set": {
                        "username": &username,
                        "flags": 2
                    },
                    "$unset": {
                        "avatar": 1,
                        "status": 1,
                        "profile": 1
                    }
                },
                None,
            )
            .await
            .map_err(|_| Error::DatabaseError {
                with: "user",
                operation: "update_one",
            })?;
        Ok(())
    }

    async fn update_username(&self, id: &str, new_username: &str) -> Result<()> {
        self.revolt
            .collection("users")
            .update_one(
                doc! { "_id": id },
                doc! {
                    "$set": {
                        "username": new_username
                    }
                },
                None,
            )
            .await
            .map_err(|_| Error::DatabaseError {
                operation: "update_one",
                with: "user",
            })?;
        Ok(())
    }

    async fn make_user_already_in_relations_blocked(
        &self,
        origin_id: &str,
        target_id: &str,
    ) -> Result<()> {
        self.revolt
            .collection("users")
            .update_one(
                doc! {
                    "_id": origin_id,
                    "relations._id": target_id
                },
                doc! {
                    "$set": {
                        "relations.$.status": "Blocked"
                    }
                },
                None,
            )
            .await
            .map_err(|_| Error::DatabaseError {
                operation: "update_one",
                with: "user",
            })?;
        Ok(())
    }

    async fn make_user_already_in_relations_blocked_by(
        &self,
        target_id: &str,
        origin_id: &str,
    ) -> Result<()> {
        self.revolt
            .collection("users")
            .update_one(
                doc! {
                    "_id": target_id,
                    "relations._id": origin_id
                },
                doc! {
                    "$set": {
                        "relations.$.status": "BlockedOther"
                    }
                },
                None,
            )
            .await
            .map_err(|_| Error::DatabaseError {
                operation: "update_one",
                with: "user",
            })?;
        Ok(())
    }

    async fn make_user_not_in_relations_blocked(
        &self,
        origin_id: &str,
        target_id: &str,
    ) -> Result<()> {
        self.revolt
            .collection("users")
            .update_one(
                doc! {
                    "_id": origin_id
                },
                doc! {
                    "$push": {
                        "relations": {
                            "_id": target_id,
                            "status": "Blocked"
                        }
                    }
                },
                None,
            )
            .await
            .map_err(|_| Error::DatabaseError {
                operation: "update_one",
                with: "user",
            })?;
        Ok(())
    }

    async fn make_user_not_in_relations_blocked_by(
        &self,
        target_id: &str,
        origin_id: &str,
    ) -> Result<()> {
        self.revolt
            .collection("users")
            .update_one(
                doc! {
                    "_id": target_id
                },
                doc! {
                    "$push": {
                        "relations": {
                            "_id": origin_id,
                            "status": "BlockedOther"
                        }
                    }
                },
                None,
            )
            .await
            .map_err(|_| Error::DatabaseError {
                operation: "update_one",
                with: "user",
            })?;
        Ok(())
    }

    async fn apply_profile_changes(&self, id: &str, change_doc: Document) -> Result<()> {
        self.revolt
            .collection("users")
            .update_one(doc! { "_id": id }, change_doc, None)
            .await
            .map_err(|_| Error::DatabaseError {
                operation: "update_one",
                with: "user",
            })?;
        Ok(())
    }

    async fn remove_user_from_relations(&self, id: &str, target: &str) -> Result<()> {
        self.revolt
            .collection("users")
            .update_one(
                doc! {
                    "_id": id
                },
                doc! {
                    "$pull": {
                        "relations": {
                            "_id": target
                        }
                    }
                },
                None,
            )
            .await
            .map_err(|_| Error::DatabaseError {
                operation: "update_one",
                with: "user",
            })?;
        Ok(())
    }

    async fn get_accounts_subscriptions(
        &self,
        target_ids: Vec<&str>,
    ) -> Option<Vec<SubscriptionInfo>> {
        if let Ok(mut cursor) = self
            .revolt
            .collection("accounts")
            .find(
                doc! {
                    "_id": {
                        "$in": target_ids
                    },
                    "sessions.subscription": {
                        "$exists": true
                    }
                },
                FindOptions::builder()
                    .projection(doc! { "sessions": 1 })
                    .build(),
            )
            .await
        {
            let mut subscriptions = vec![];
            while let Some(result) = cursor.next().await {
                if let Ok(doc) = result {
                    if let Ok(sessions) = doc.get_array("sessions") {
                        for session in sessions {
                            if let Some(doc) = session.as_document() {
                                if let Ok(sub) = doc.get_document("subscription") {
                                    let endpoint = sub.get_str("endpoint").unwrap().to_string();
                                    let p256dh = sub.get_str("p256dh").unwrap().to_string();
                                    let auth = sub.get_str("auth").unwrap().to_string();

                                    subscriptions
                                        .push(SubscriptionInfo::new(endpoint, p256dh, auth));
                                }
                            }
                        }
                    }
                }
            }
            Some(subscriptions)
        } else {
            None
        }
    }

    async fn subscribe(
        &self,
        account_id: &str,
        session_id: &str,
        subscription: Subscription,
    ) -> Result<()> {
        self.revolt.collection("accounts")
            .update_one(
                doc! {
                "_id": account_id,
                "sessions.id": session_id
            },
                doc! {
                "$set": {
                    "sessions.$.subscription": to_document(&subscription)
                        .map_err(|_| Error::DatabaseError { operation: "to_document", with: "subscription" })?
                }
            },
                None,
            )
            .await
            .map_err(|_| Error::DatabaseError { operation: "update_one", with: "account" })?;
        Ok(())
    }

    async fn unsubscribe(&self, account_id: &str, session_id: &str) -> Result<()> {
        self.revolt
            .collection("accounts")
            .update_one(
                doc! {
                    "_id": account_id,
                    "sessions.id": session_id
                },
                doc! {
                    "$unset": {
                        "sessions.$.subscription": 1
                    }
                },
                None,
            )
            .await
            .map_err(|_| Error::DatabaseError {
                operation: "to_document",
                with: "subscription",
            })?;
        Ok(())
    }

    async fn get_attachment(&self, id: &str, tag: &str, parent_type: &str) -> Result<File> {
        let key = format!("{}_id", parent_type);
        if let Some(doc) = self
            .revolt
            .collection("attachments")
            .find_one(
                doc! {
                    "_id": id,
                    "tag": tag,
                    key.clone(): {
                        "$exists": false
                    }
                },
                None,
            )
            .await
            .map_err(|_| Error::DatabaseError {
                operation: "find_one",
                with: "attachment",
            })?
        {
            let attachment = from_document::<File>(doc).map_err(|_| Error::DatabaseError {
                operation: "from_document",
                with: "attachment",
            })?;
            Ok(attachment)
        } else {
            Err(Error::UnknownAttachment)
        }
    }

    async fn link_attachment_to_parent(
        &self,
        id: &str,
        parent_type: &str,
        parent_id: &str,
    ) -> Result<()> {
        let key = format!("{}_id", parent_type);
        self.revolt
            .collection("attachments")
            .update_one(
                doc! {
                    "_id": id
                },
                doc! {
                    "$set": {
                        key: &parent_id
                    }
                },
                None,
            )
            .await
            .map_err(|_| Error::DatabaseError {
                operation: "update_one",
                with: "attachment",
            })?;
        Ok(())
    }

    async fn delete_attachment(&self, id: &str) -> Result<()> {
        self.revolt
            .collection("attachments")
            .update_one(
                doc! {
                    "_id": id
                },
                doc! {
                    "$set": {
                        "deleted": true
                    }
                },
                None,
            )
            .await
            .map(|_| ())
            .map_err(|_| Error::DatabaseError {
                operation: "update_one",
                with: "attachment",
            })
    }

    async fn delete_attachments(&self, ids: Vec<&str>) -> Result<()> {
        self.revolt
            .collection("attachments")
            .update_many(
                doc! {
                    "_id": {
                        "$in": ids
                    }
                },
                doc! {
                    "$set": {
                        "deleted": true
                    }
                },
                None,
            )
            .await
            .map_err(|_| Error::DatabaseError {
                operation: "update_one",
                with: "attachment",
            })?;
        Ok(())
    }

    async fn delete_attachments_of_messages(&self, message_ids: Vec<&str>) -> Result<()> {
        self.revolt
            .collection("attachments")
            .update_many(
                doc! {
                    "message_id": {
                        "$in": message_ids
                    }
                },
                doc! {
                    "$set": {
                        "deleted": true
                    }
                },
                None,
            )
            .await
            .map_err(|_| Error::DatabaseError {
                operation: "update_many",
                with: "attachments",
            })?;
        Ok(())
    }

    async fn get_bot_count_owned_by_user(&self, user_id: &str) -> Result<u64> {
        Ok(self
            .revolt
            .collection("bots")
            .count_documents(
                doc! {
                    "owner": user_id
                },
                None,
            )
            .await
            .map_err(|_| Error::DatabaseError {
                operation: "count_documents",
                with: "bots",
            })? as u64)
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

    async fn add_bot(&self, bot: &Bot) -> Result<()> {
        self.revolt
            .collection("bots")
            .insert_one(
                to_document(bot).map_err(|_| Error::DatabaseError {
                    with: "bot",
                    operation: "to_document",
                })?,
                None,
            )
            .await
            .map_err(|_| Error::DatabaseError {
                operation: "insert_one",
                with: "user",
            })?;
        Ok(())
    }

    async fn delete_bot(&self, id: &str) -> Result<()> {
        self.revolt
            .collection("bots")
            .delete_one(
                doc! {
                    "_id": id
                },
                None,
            )
            .await
            .map_err(|_| Error::DatabaseError {
                with: "bot",
                operation: "delete_one",
            })?;
        Ok(())
    }

    async fn apply_bot_changes(&self, id: &str, change_doc: Document) -> Result<()> {
        self.revolt
            .collection("bots")
            .update_one(doc! { "_id": id }, change_doc, None)
            .await
            .map_err(|_| Error::DatabaseError {
                operation: "update_one",
                with: "bot",
            })?;
        Ok(())
    }

    async fn delete_invites_associated_to_channel(&self, id: &str) -> Result<()> {
        self.revolt
            .collection("channel_invites")
            .delete_many(
                doc! {
                    "channel": id
                },
                None,
            )
            .await
            .map(|_| ())
            .map_err(|_| Error::DatabaseError {
                operation: "delete_many",
                with: "channel_invites",
            })
    }

    async fn get_invite_by_id(&self, id: &str) -> Result<Invite> {
        let doc = self
            .revolt
            .collection("channel_invites")
            .find_one(doc! { "_id": id }, None)
            .await
            .map_err(|_| Error::DatabaseError {
                operation: "find_one",
                with: "invite",
            })?
            .ok_or_else(|| Error::UnknownServer)?;

        from_document::<Invite>(doc).map_err(|_| Error::DatabaseError {
            operation: "from_document",
            with: "invite",
        })
    }

    async fn add_invite(&self, invite: &Invite) -> Result<()> {
        self.revolt
            .collection("channel_invites")
            .insert_one(
                to_document(invite).map_err(|_| Error::DatabaseError {
                    operation: "to_bson",
                    with: "invite",
                })?,
                None,
            )
            .await
            .map_err(|_| Error::DatabaseError {
                operation: "insert_one",
                with: "invite",
            })?;
        Ok(())
    }

    async fn delete_invite(&self, id: &str) -> Result<()> {
        self.revolt
            .collection("channel_invites")
            .delete_one(
                doc! {
                    "_id": id
                },
                None,
            )
            .await
            .map_err(|_| Error::DatabaseError {
                operation: "delete_one",
                with: "invite",
            })?;
        Ok(())
    }

    async fn get_invites_of_server(&self, server_id: &str) -> Result<Vec<Invite>> {
        let mut cursor = self
            .revolt
            .collection("channel_invites")
            .find(
                doc! {
                    "server": server_id
                },
                None,
            )
            .await
            .map_err(|_| Error::DatabaseError {
                operation: "find",
                with: "channel_invites",
            })?;

        let mut invites = vec![];
        while let Some(result) = cursor.next().await {
            if let Ok(doc) = result {
                if let Ok(invite) = from_document::<Invite>(doc) {
                    invites.push(invite);
                }
            }
        }
        Ok(invites)
    }

    async fn delete_channel_unreads(&self, channel_id: &str) -> Result<()> {
        self.revolt
            .collection("channel_unreads")
            .delete_many(
                doc! {
                    "_id.channel": channel_id
                },
                None,
            )
            .await
            .map_err(|_| Error::DatabaseError {
                operation: "delete_many",
                with: "channel_unreads",
            })?;
        Ok(())
    }

    async fn delete_multi_channel_unreads_for_user(
        &self,
        channel_ids: Vec<&str>,
        user_id: &str,
    ) -> Result<()> {
        self.revolt
            .collection("channel_unreads")
            .delete_many(
                doc! {
                    "_id.channel": {
                        "$in": channel_ids
                    },
                    "_id.user": user_id
                },
                None,
            )
            .await
            .map_err(|_| Error::DatabaseError {
                operation: "delete_many",
                with: "channel_unreads",
            })?;
        Ok(())
    }

    async fn add_mentions_to_channel_unreads(
        &self,
        channel_id: &str,
        mentions: Vec<&str>,
        message: &str,
    ) -> Result<()> {
        self.revolt
            .collection("channel_unreads")
            .update_many(
                doc! {
                    "_id.channel": channel_id,
                    "_id.user": {
                        "$in": mentions
                    }
                },
                doc! {
                    "$push": {
                        "mentions": message
                    }
                },
                UpdateOptions::builder().upsert(true).build(),
            )
            .await
            .map_err(|_| Error::DatabaseError {
                operation: "update_many",
                with: "channel_unreads",
            })?;
        Ok(())
    }

    async fn add_channels_to_unreads_for_user(
        &self,
        channel_ids: Vec<&str>,
        user_id: &str,
        current_time: &str,
    ) -> Result<()> {
        self.revolt
            .collection("channel_unreads")
            .insert_many(
                channel_ids
                    .iter()
                    .map(|channel| {
                        doc! {
                            "_id": {
                                "channel": channel,
                                "user": user_id
                            },
                            "last_id": current_time
                        }
                    })
                    .collect::<Vec<Document>>(),
                None,
            )
            .await
            .map_err(|_| Error::DatabaseError {
                operation: "update_many",
                with: "channel_unreads",
            })
            .map(|_| ())
    }

    async fn get_unreads_for_user(&self, user_id: &str) -> Result<Vec<Document>> {
        Ok(self
            .revolt
            .collection("channel_unreads")
            .find(
                doc! {
                    "_id.user": user_id
                },
                None,
            )
            .await
            .map_err(|_| Error::DatabaseError {
                operation: "find_one",
                with: "user_settings",
            })?
            .filter_map(async move |s| s.ok())
            .collect::<Vec<Document>>()
            .await)
    }

    async fn update_last_message_in_channel_unreads(
        &self,
        channel_id: &str,
        user_id: &str,
        message_id: &str,
    ) -> Result<()> {
        self.revolt
            .collection("channel_unreads")
            .update_one(
                doc! {
                    "_id.channel": channel_id,
                    "_id.user": user_id
                },
                doc! {
                    "$unset": {
                        "mentions": 1
                    },
                    "$set": {
                        "last_id": message_id
                    }
                },
                UpdateOptions::builder().upsert(true).build(),
            )
            .await
            .map_err(|_| Error::DatabaseError {
                operation: "update_one",
                with: "channel_unreads",
            })?;
        Ok(())
    }

    async fn does_channel_exist_by_nonce(&self, nonce: &str) -> Result<bool> {
        Ok(self
            .revolt
            .collection("channels")
            .find_one(
                doc! {
                    "nonce": nonce
                },
                None,
            )
            .await
            .map_err(|_| Error::DatabaseError {
                operation: "find_one",
                with: "channel",
            })?
            .is_some())
    }

    async fn remove_recipient_from_channel(
        &self,
        channel_id: &str,
        recipient_id: &str,
    ) -> Result<()> {
        self.revolt
            .collection("channels")
            .update_one(
                doc! {
                    "_id": channel_id
                },
                doc! {
                    "$pull": {
                        "recipients": recipient_id
                    }
                },
                None,
            )
            .await
            .map_err(|_| Error::DatabaseError {
                operation: "update_one",
                with: "channel",
            })?;
        Ok(())
    }

    async fn update_channel_role_permissions(
        &self,
        channel_id: &str,
        role: &str,
        permissions: i32,
    ) -> Result<()> {
        self.revolt
            .collection("channels")
            .update_one(
                doc! { "_id": channel_id },
                doc! {
                    "$set": {
                        "role_permissions.".to_owned() + role: permissions
                    }
                },
                None,
            )
            .await
            .map_err(|_| Error::DatabaseError {
                operation: "update_one",
                with: "channel",
            })?;
        Ok(())
    }

    async fn update_channel_permissions(&self, channel_id: &str, permissions: i32) -> Result<()> {
        self.revolt
            .collection("channels")
            .update_one(
                doc! { "_id": channel_id },
                doc! {
                    "$set": {
                        "permissions": permissions
                    }
                },
                None,
            )
            .await
            .map_err(|_| Error::DatabaseError {
                operation: "update_one",
                with: "channel",
            })?;
        Ok(())
    }

    async fn update_channel_default_permissions(
        &self,
        channel_id: &str,
        default_permissions: i32,
    ) -> Result<()> {
        self.revolt
            .collection("channels")
            .update_one(
                doc! { "_id": channel_id },
                doc! {
                    "$set": {
                        "default_permissions": default_permissions
                    }
                },
                None,
            )
            .await
            .map_err(|_| Error::DatabaseError {
                operation: "update_one",
                with: "channel",
            })?;
        Ok(())
    }

    async fn delete_server_channels_role_permissions(
        &self,
        server_id: &str,
        role_id: &str,
    ) -> Result<()> {
        self.revolt
            .collection("channels")
            .update_many(
                doc! {
                    "server": server_id
                },
                doc! {
                    "$unset": {
                        "role_permissions.".to_owned() + role_id: 1
                    }
                },
                None,
            )
            .await
            .map_err(|_| Error::DatabaseError {
                operation: "update_one",
                with: "channels",
            })?;
        Ok(())
    }

    async fn get_dm_channels_from_user(&self, user_id: &str) -> Result<Vec<Document>> {
        let mut cursor = self
            .revolt
            .collection("channels")
            .find(
                doc! {
                    "$or": [
                        {
                            "channel_type": "DirectMessage",
                            "active": true
                        },
                        {
                            "channel_type": "Group"
                        }
                    ],
                    "recipients": user_id
                },
                None,
            )
            .await
            .map_err(|_| Error::DatabaseError {
                operation: "find",
                with: "channels",
            })?;

        let mut channels = vec![];
        while let Some(result) = cursor.next().await {
            if let Ok(doc) = result {
                channels.push(doc);
            }
        }
        Ok(channels)
    }
}
