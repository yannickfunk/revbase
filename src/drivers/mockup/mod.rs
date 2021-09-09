use crate::entities::{BannedUser, Bot, User};
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

    async fn get_users_as_banned_users(&self, user_ids: Vec<&str>) -> Result<Vec<BannedUser>> {
        todo!()
    }

    async fn get_bot_users_owned_by_user_id(&self, id: &str) -> Result<Vec<User>> {
        todo!()
    }

    async fn get_bots_owned_by_user_id(&self, id: &str) -> Result<Vec<Bot>> {
        todo!()
    }

    async fn get_mutual_friends_ids(
        &self,
        user_id_a: &str,
        user_id_b: &str,
    ) -> Result<Vec<String>> {
        todo!()
    }

    async fn add_user(&self, id: &str, username: &str) -> Result<()> {
        todo!()
    }

    async fn add_bot_user(&self, id: &str, username: &str, owner_id: &str) -> Result<()> {
        todo!()
    }

    async fn make_bot_user_deleted(&self, id: &str) -> Result<()> {
        todo!()
    }

    async fn update_username(&self, id: &str, new_username: &str) -> Result<()> {
        todo!()
    }

    async fn make_user_already_in_relations_blocked(
        &self,
        origin_id: &str,
        target_id: &str,
    ) -> Result<()> {
        todo!()
    }

    async fn make_user_already_in_relations_blocked_by(
        &self,
        target_id: &str,
        origin_id: &str,
    ) -> Result<()> {
        todo!()
    }

    async fn make_user_not_in_relations_blocked(
        &self,
        origin_id: &str,
        target_id: &str,
    ) -> Result<()> {
        todo!()
    }

    async fn make_user_not_in_relations_blocked_by(
        &self,
        target_id: &str,
        origin_id: &str,
    ) -> Result<()> {
        todo!()
    }
}
