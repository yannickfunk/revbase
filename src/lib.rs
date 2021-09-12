#![feature(async_closure)]

extern crate mongodb;

use crate::entities::{BannedUser, Bot, File, Invite, Subscription, User};
use crate::util::result::Result;
use drivers::{mockup::Mockup, mongo::MongoDB};
use enum_dispatch::enum_dispatch;
use mongodb::bson::Document;
use rocket::async_trait;
use web_push::SubscriptionInfo;

pub mod drivers;
mod entities;
pub mod permissions;
pub mod util;

#[async_trait]
#[enum_dispatch]
pub trait Queries {
    // user collection
    async fn get_user_by_id(&self, id: &str) -> Result<User>;
    async fn get_user_by_username(&self, username: &str) -> Result<User>;
    async fn get_user_id_by_bot_token(&self, token: &str) -> Result<String>;
    async fn get_users(&self, user_ids: Vec<&str>) -> Result<Vec<User>>;
    async fn get_users_as_banned_users(&self, user_ids: Vec<&str>) -> Result<Vec<BannedUser>>;
    async fn get_bot_users_owned_by_user_id(&self, id: &str) -> Result<Vec<User>>;
    async fn get_mutual_friends_ids(&self, user_id_a: &str, user_id_b: &str)
        -> Result<Vec<String>>;
    async fn add_user(&self, id: &str, username: &str) -> Result<()>;
    async fn add_bot_user(&self, id: &str, username: &str, owner_id: &str) -> Result<()>;
    async fn delete_user(&self, id: &str) -> Result<()>;
    async fn update_username(&self, id: &str, new_username: &str) -> Result<()>;
    async fn make_user_already_in_relations_blocked(
        &self,
        origin_id: &str,
        target_id: &str,
    ) -> Result<()>;
    async fn make_user_already_in_relations_blocked_by(
        &self,
        target_id: &str,
        origin_id: &str,
    ) -> Result<()>;
    async fn make_user_not_in_relations_blocked(
        &self,
        origin_id: &str,
        target_id: &str,
    ) -> Result<()>;
    async fn make_user_not_in_relations_blocked_by(
        &self,
        target_id: &str,
        origin_id: &str,
    ) -> Result<()>;
    async fn apply_profile_changes(&self, id: &str, change_doc: Document) -> Result<()>;
    async fn remove_user_from_relations(&self, id: &str, target_id: &str) -> Result<()>;

    // accounts collection
    async fn get_accounts_subscriptions(
        &self,
        target_ids: Vec<&str>,
    ) -> Option<Vec<SubscriptionInfo>>;
    async fn subscribe(
        &self,
        account_id: &str,
        session_id: &str,
        subscription: Subscription,
    ) -> Result<()>;
    async fn unsubscribe(&self, account_id: &str, session_id: &str) -> Result<()>;

    // attachments
    async fn get_attachment(&self, id: &str, tag: &str, parent_type: &str) -> Result<File>;
    async fn link_attachment_to_parent(
        &self,
        id: &str,
        parent_type: &str,
        parent_id: &str,
    ) -> Result<()>;
    async fn delete_attachment(&self, id: &str) -> Result<()>;
    async fn delete_attachments(&self, ids: Vec<&str>) -> Result<()>;
    async fn delete_attachments_of_messages(&self, message_ids: Vec<&str>) -> Result<()>;

    // bots
    async fn get_bot_count_owned_by_user(&self, user_id: &str) -> Result<u64>;
    async fn get_bots_owned_by_user_id(&self, id: &str) -> Result<Vec<Bot>>;
    async fn add_bot(&self, bot: &Bot) -> Result<()>;
    async fn delete_bot(&self, id: &str) -> Result<()>;
    async fn apply_bot_changes(&self, id: &str, change_doc: Document) -> Result<()>;

    // channel_invites
    async fn delete_invites_associated_to_channel(&self, id: &str) -> Result<()>;
    async fn get_invite_by_id(&self, id: &str) -> Result<Invite>;
}

#[enum_dispatch(Queries)]
pub enum Driver {
    Mongo(MongoDB),
    Mockup(Mockup),
}

pub struct Database {
    driver: Driver,
}

impl Database {
    pub async fn new_from_mongo(mongo_uri: &str) -> Self {
        Self {
            driver: Driver::from(MongoDB::new(mongo_uri).await),
        }
    }

    pub fn new_from_mockup() -> Self {
        let mockup = Mockup {};
        Self {
            driver: Driver::from(mockup),
        }
    }
}

#[async_trait]
impl Queries for Database {
    async fn get_user_by_id(&self, id: &str) -> Result<User> {
        self.driver.get_user_by_id(id).await
    }

    async fn get_user_by_username(&self, username: &str) -> Result<User> {
        self.driver.get_user_by_username(username).await
    }

    async fn get_user_id_by_bot_token(&self, token: &str) -> Result<String> {
        self.driver.get_user_id_by_bot_token(token).await
    }

    async fn get_users(&self, user_ids: Vec<&str>) -> Result<Vec<User>> {
        self.driver.get_users(user_ids).await
    }

    async fn get_users_as_banned_users(&self, user_ids: Vec<&str>) -> Result<Vec<BannedUser>> {
        self.driver.get_users_as_banned_users(user_ids).await
    }

    async fn get_bot_users_owned_by_user_id(&self, id: &str) -> Result<Vec<User>> {
        self.driver.get_bot_users_owned_by_user_id(id).await
    }

    async fn get_mutual_friends_ids(
        &self,
        user_id_a: &str,
        user_id_b: &str,
    ) -> Result<Vec<String>> {
        self.driver
            .get_mutual_friends_ids(user_id_a, user_id_b)
            .await
    }

    async fn add_user(&self, id: &str, username: &str) -> Result<()> {
        self.driver.add_user(id, username).await
    }

    async fn add_bot_user(&self, id: &str, username: &str, owner_id: &str) -> Result<()> {
        self.driver.add_bot_user(id, username, owner_id).await
    }

    async fn delete_user(&self, id: &str) -> Result<()> {
        self.driver.delete_user(id).await
    }

    async fn update_username(&self, id: &str, new_username: &str) -> Result<()> {
        self.driver.update_username(id, new_username).await
    }

    async fn make_user_already_in_relations_blocked(
        &self,
        origin_id: &str,
        target_id: &str,
    ) -> Result<()> {
        self.make_user_already_in_relations_blocked(origin_id, target_id)
            .await
    }

    async fn make_user_already_in_relations_blocked_by(
        &self,
        target_id: &str,
        origin_id: &str,
    ) -> Result<()> {
        self.driver
            .make_user_already_in_relations_blocked_by(target_id, origin_id)
            .await
    }

    async fn make_user_not_in_relations_blocked(
        &self,
        origin_id: &str,
        target_id: &str,
    ) -> Result<()> {
        self.driver
            .make_user_not_in_relations_blocked(origin_id, target_id)
            .await
    }

    async fn make_user_not_in_relations_blocked_by(
        &self,
        target_id: &str,
        origin_id: &str,
    ) -> Result<()> {
        self.driver
            .make_user_not_in_relations_blocked_by(target_id, origin_id)
            .await
    }

    async fn apply_profile_changes(&self, id: &str, change_doc: Document) -> Result<()> {
        self.driver.apply_profile_changes(id, change_doc).await
    }

    async fn remove_user_from_relations(&self, id: &str, target: &str) -> Result<()> {
        self.driver.remove_user_from_relations(id, target).await
    }

    async fn get_accounts_subscriptions(
        &self,
        target_ids: Vec<&str>,
    ) -> Option<Vec<SubscriptionInfo>> {
        self.driver.get_accounts_subscriptions(target_ids).await
    }

    async fn subscribe(
        &self,
        account_id: &str,
        session_id: &str,
        subscription: Subscription,
    ) -> Result<()> {
        self.driver
            .subscribe(account_id, session_id, subscription)
            .await
    }

    async fn unsubscribe(&self, account_id: &str, session_id: &str) -> Result<()> {
        self.driver.unsubscribe(account_id, session_id).await
    }

    async fn get_attachment(&self, id: &str, tag: &str, parent_type: &str) -> Result<File> {
        self.driver.get_attachment(id, tag, parent_type).await
    }

    async fn link_attachment_to_parent(
        &self,
        id: &str,
        parent_type: &str,
        parent_id: &str,
    ) -> Result<()> {
        self.driver
            .link_attachment_to_parent(id, parent_type, parent_id)
            .await
    }

    async fn delete_attachment(&self, id: &str) -> Result<()> {
        self.driver.delete_attachment(id).await
    }

    async fn delete_attachments(&self, ids: Vec<&str>) -> Result<()> {
        self.driver.delete_attachments(ids).await
    }

    async fn delete_attachments_of_messages(&self, message_ids: Vec<&str>) -> Result<()> {
        self.driver
            .delete_attachments_of_messages(message_ids)
            .await
    }

    async fn get_bot_count_owned_by_user(&self, user_id: &str) -> Result<u64> {
        self.driver.get_bot_count_owned_by_user(user_id).await
    }

    async fn get_bots_owned_by_user_id(&self, id: &str) -> Result<Vec<Bot>> {
        self.driver.get_bots_owned_by_user_id(id).await
    }

    async fn add_bot(&self, bot: &Bot) -> Result<()> {
        self.driver.add_bot(bot).await
    }

    async fn delete_bot(&self, id: &str) -> Result<()> {
        self.delete_bot(id).await
    }

    async fn apply_bot_changes(&self, id: &str, change_doc: Document) -> Result<()> {
        self.driver.apply_bot_changes(id, change_doc).await
    }

    async fn delete_invites_associated_to_channel(&self, id: &str) -> Result<()> {
        self.driver.delete_invites_associated_to_channel(id).await
    }

    async fn get_invite_by_id(&self, id: &str) -> Result<Invite> {
        self.get_invite_by_id(id).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_std;
    use env_logger;

    #[test]
    fn it_works() {
        env_logger::init_from_env(env_logger::Env::default().filter_or("RUST_LOG", "info"));
        let user = async_std::task::block_on(async {
            let db = Database::new_from_mongo("mongodb://test:test@localhost:27018/test?authSource=admin&readPreference=primary&ssl=false").await;
            let mutual_friends = db
                .get_mutual_friends_ids("01FDX1NCVAKFPVSXNNVEVMQHAF", "01FDX1DHBVS9NF6KSQECFVRFGB")
                .await
                .unwrap();
            db.get_user_by_id(&mutual_friends[0]).await
        });
        println!("{:?}", user);
    }
}
