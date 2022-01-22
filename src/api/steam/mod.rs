use rocket::serde::{ Serialize, Deserialize };
use crate::serde_utils::u64_or_parse_str;

pub type SteamID = u64;

#[derive(Debug, Serialize, Deserialize)]
pub struct SteamUser {
    #[serde(rename(deserialize = "steamid"))]
    #[serde(deserialize_with = "u64_or_parse_str")]
    pub steam_id: u64,

    #[serde(rename(deserialize = "personaname"))]
    pub screen_name: String,

    #[serde(rename(deserialize = "avatar"))]
    pub avatar_thumb: String,

    #[serde(rename(deserialize = "avatarmedium"))]
    pub avatar: String,

    #[serde(rename(deserialize = "communityvisibilitystate"))]
    pub visibility: i8,

    #[serde(rename(deserialize = "personastate"))]
    pub user_state: i8,
}

pub mod responses {
    use rocket::serde::Serialize;
    use std::collections::HashMap;
    use super::*;

    #[derive(Serialize)]
    #[serde(untagged)]
    pub enum GetFriendsResponse {
        Type1(Vec<SteamID>),
        Type2(HashMap<SteamID, SteamUser>),
    }
}
pub use responses::*;

pub mod frontend;
pub mod backend;

pub use frontend::routes;