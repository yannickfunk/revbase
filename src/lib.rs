#![feature(async_closure)]

extern crate mongodb;

use crate::entities::{BannedUser, Bot, User};
use crate::util::result::Result;
use drivers::{mockup::Mockup, mongo::MongoDB};
use enum_dispatch::enum_dispatch;
use mongodb::bson::Document;
use rocket::async_trait;

pub mod drivers;
mod entities;
pub mod permissions;
pub mod util;

#[async_trait]
#[enum_dispatch]
pub trait Queries {
    async fn get_user_by_id(&self, id: &str) -> Result<User>;
    async fn get_user_by_username(&self, username: &str) -> Result<User>;
    async fn get_users(&self, user_ids: Vec<&str>) -> Result<Vec<User>>;
    async fn get_users_as_banned_users(&self, user_ids: Vec<&str>) -> Result<Vec<BannedUser>>;
    async fn get_bot_users_owned_by_user_id(&self, id: &str) -> Result<Vec<User>>;
    async fn get_bots_owned_by_user_id(&self, id: &str) -> Result<Vec<Bot>>;
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

    async fn get_users(&self, user_ids: Vec<&str>) -> Result<Vec<User>> {
        self.driver.get_users(user_ids).await
    }

    async fn get_users_as_banned_users(&self, user_ids: Vec<&str>) -> Result<Vec<BannedUser>> {
        self.driver.get_users_as_banned_users(user_ids).await
    }

    async fn get_bot_users_owned_by_user_id(&self, id: &str) -> Result<Vec<User>> {
        self.driver.get_bot_users_owned_by_user_id(id).await
    }

    async fn get_bots_owned_by_user_id(&self, id: &str) -> Result<Vec<Bot>> {
        self.driver.get_bots_owned_by_user_id(id).await
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
            let db = Database::new_from_mongo("").await;
            let mutual_friends = db
                .get_mutual_friends_ids("01FDX1NCVAKFPVSXNNVEVMQHAF", "01FDX1DHBVS9NF6KSQECFVRFGB")
                .await
                .unwrap();
            db.get_user_by_id(&mutual_friends[0]).await
        });
        println!("{:?}", user);
    }
}
