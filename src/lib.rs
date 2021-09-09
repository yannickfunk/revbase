#![feature(async_closure)]

use crate::entities::{BannedUser, Bot, User};
use crate::util::result::Result;
use drivers::{mockup::Mockup, mongo::MongoDB};
use enum_dispatch::enum_dispatch;
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
    pub fn new_from_mongo(driver: MongoDB) -> Self {
        Self {
            driver: Driver::from(driver),
        }
    }

    pub fn new_from_mockup(driver: Mockup) -> Self {
        Self {
            driver: Driver::from(driver),
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
            let driver = MongoDB::new("").await;
            let db = Database::new_from_mongo(driver);
            let mutual_friends = db
                .get_mutual_friends_ids("01FDX1NCVAKFPVSXNNVEVMQHAF", "01FDX1DHBVS9NF6KSQECFVRFGB")
                .await
                .unwrap();
            db.get_user_by_id(&mutual_friends[0]).await
        });
        println!("{:?}", user);
    }
}
