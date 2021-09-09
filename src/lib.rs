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
            db.get_users(vec![
                "01FDFSV68HTQ164AZPKJE879Z2",
                "01FDX1DHBVS9NF6KSQECFVRFGB",
            ])
            .await
        });
        println!("{:?}", user);
    }
}
