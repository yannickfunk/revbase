use crate::entities::User;
use crate::util::result::Result;
use rocket::async_trait;

pub mod drivers;
mod entities;
pub mod permissions;
pub mod util;

#[async_trait]
pub trait Database {
    async fn get_user_by_id(&self, id: &str) -> Result<User>;
}
