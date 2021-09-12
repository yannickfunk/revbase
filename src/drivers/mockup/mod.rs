use crate::entities::{BannedUser, Bot, File, Invite, Subscription, User};
use crate::util::result::Result;
use crate::Queries;
use mongodb::bson::Document;
use rocket::async_trait;
use web_push::SubscriptionInfo;

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

    async fn get_user_id_by_bot_token(&self, token: &str) -> Result<String> {
        todo!()
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

    async fn delete_user(&self, id: &str) -> Result<()> {
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

    async fn apply_profile_changes(&self, id: &str, change_doc: Document) -> Result<()> {
        todo!()
    }

    async fn remove_user_from_relations(&self, id: &str, target: &str) -> Result<()> {
        todo!()
    }

    async fn get_accounts_subscriptions(
        &self,
        target_ids: Vec<&str>,
    ) -> Option<Vec<SubscriptionInfo>> {
        todo!()
    }

    async fn subscribe(
        &self,
        account_id: &str,
        session_id: &str,
        subscription: Subscription,
    ) -> Result<()> {
        todo!()
    }

    async fn unsubscribe(&self, account_id: &str, session_id: &str) -> Result<()> {
        todo!()
    }

    async fn get_attachment(&self, id: &str, tag: &str, parent_type: &str) -> Result<File> {
        todo!()
    }

    async fn link_attachment_to_parent(
        &self,
        id: &str,
        parent_type: &str,
        parent_id: &str,
    ) -> Result<()> {
        todo!()
    }

    async fn delete_attachment(&self, id: &str) -> Result<()> {
        todo!()
    }

    async fn delete_attachments(&self, ids: Vec<&str>) -> Result<()> {
        todo!()
    }

    async fn delete_attachments_of_messages(&self, message_ids: Vec<&str>) -> Result<()> {
        todo!()
    }

    async fn get_bot_count_owned_by_user(&self, user_id: &str) -> Result<u64> {
        todo!()
    }

    async fn get_bots_owned_by_user_id(&self, id: &str) -> Result<Vec<Bot>> {
        todo!()
    }

    async fn add_bot(&self, bot: &Bot) -> Result<()> {
        todo!()
    }

    async fn delete_bot(&self, id: &str) -> Result<()> {
        todo!()
    }

    async fn apply_bot_changes(&self, id: &str, change_doc: Document) -> Result<()> {
        todo!()
    }

    async fn delete_invites_associated_to_channel(&self, id: &str) -> Result<()> {
        todo!()
    }

    async fn get_invite_by_id(&self, id: &str) -> Result<Invite> {
        todo!()
    }

    async fn add_invite(&self, invite: &Invite) -> Result<()> {
        todo!()
    }

    async fn delete_invite(&self, id: &str) -> Result<()> {
        todo!()
    }

    async fn get_invites_of_server(&self, server_id: &str) -> Result<Vec<Invite>> {
        todo!()
    }

    async fn delete_channel_unreads(&self, channel_id: &str) -> Result<()> {
        todo!()
    }

    async fn delete_multi_channel_unreads_for_user(
        &self,
        channel_ids: Vec<&str>,
        user_id: &str,
    ) -> Result<()> {
        todo!()
    }

    async fn add_mentions_to_channel_unreads(
        &self,
        channel_id: &str,
        mentions: Vec<&str>,
        message: &str,
    ) -> Result<()> {
        todo!()
    }

    async fn add_channels_to_unreads_for_user(
        &self,
        channel_ids: Vec<&str>,
        user_id: &str,
        current_time: &str,
    ) -> Result<()> {
        todo!()
    }
}
