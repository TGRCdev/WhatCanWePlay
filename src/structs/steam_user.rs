use serde::{ Serialize, Deserialize };

use std::borrow::Cow;

#[derive(Debug, Serialize, Deserialize)]
pub struct SteamUser<'a> {
    #[serde(rename(deserialize = "steamid"))]
    pub steam_id: u64,

    #[serde(rename(deserialize = "personaname"))]
    pub screen_name: Cow<'a, str>,

    #[serde(rename(deserialize = "avatar"))]
    pub avatar_thumb: String,

    #[serde(rename(deserialize = "avatarmedium"))]
    pub avatar: String,

    #[serde(rename(deserialize = "communityvisibilitystate"))]
    pub visibility: i8,

    #[serde(rename(deserialize = "personastate"))]
    pub online: bool,
}