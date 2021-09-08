use crate::entities::User;
use crate::util::result::Result;
use crate::Queries;
use rocket::async_trait;
pub struct Mockup {}

#[async_trait]
impl Queries for Mockup {
    async fn get_user_by_id(&self, id: &str) -> Result<User> {
        Ok(User {
            id: "".to_string(),
            username: "".to_string(),
            avatar: None,
            relations: None,
            badges: None,
            status: None,
            profile: None,
            flags: None,
            bot: None,
            relationship: None,
            online: None,
        })
    }

    async fn get_user_by_username(&self, username: &str) -> Result<User> {
        Ok(User {
            id: "".to_string(),
            username: "".to_string(),
            avatar: None,
            relations: None,
            badges: None,
            status: None,
            profile: None,
            flags: None,
            bot: None,
            relationship: None,
            online: None,
        })
    }

    async fn get_users(&self, user_ids: Vec<&str>) -> Result<Vec<User>> {
        todo!()
    }
}
