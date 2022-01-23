use rocket::serde::{ Serialize, Deserialize };
use crate::serde_utils::{
    u64_or_parse_str, value_to_string,
};
use std::{
    fmt::Display,
    str::FromStr,
    num::ParseIntError,
};

#[derive(
    Debug, Deserialize, Serialize,
    PartialEq, Eq,
    PartialOrd, Ord,
    Hash, Clone, Copy,
)]
#[repr(transparent)]
pub struct SteamID (
    #[serde(serialize_with = "value_to_string")]
    #[serde(deserialize_with = "u64_or_parse_str")]
    u64
);

impl Display for SteamID {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl From<u64> for SteamID {
    fn from(id: u64) -> Self {
        Self(id)
    }
}

impl FromStr for SteamID {
    type Err = ParseIntError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(s.parse()?))
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SteamUser {
    #[serde(rename(deserialize = "steamid"))]
    pub steam_id: SteamID,

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