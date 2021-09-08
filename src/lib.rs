use crate::entities::User;
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
            let driver = Mockup {};
            let db = Database::new_from_mockup(driver);
            db.get_user_by_username("penis").await
        });
        println!("{:?}", user);
    }
}
