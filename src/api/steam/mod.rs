use rocket::serde::{ Serialize, Deserialize };
use serde_repr::{ Serialize_repr, Deserialize_repr };
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

#[derive(Debug, PartialEq, Serialize_repr, Deserialize_repr)]
#[repr(u8)]
pub enum SteamVisibility {
    Private = 1,
    FriendsOnly = 2,
    FriendsOfFriends = 3,
    UsersOnly = 4,
    Public = 5,
}

#[derive(Debug, PartialEq, Serialize_repr, Deserialize_repr)]
#[repr(u8)]
pub enum SteamUserState {
    Offline = 0,
    Online = 1,
    Busy = 2,
    Away = 3,
    Snooze = 4,
    LookingToTrade = 5,
    LookingToPlay = 6,
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
    pub visibility: SteamVisibility,

    #[serde(rename(deserialize = "personastate"))]
    pub user_state: SteamUserState,
}

pub mod requests {
    use serde::Deserialize;
    use super::SteamID;

    #[derive(Deserialize)]
    #[serde(untagged)]
    pub enum GetSteamUsersInfoRequest {
        Single(SteamID),
        Vec(Vec<SteamID>),
        Object {
            steam_ids: Vec<SteamID>,
        }
    }

    #[derive(Deserialize)]
    #[serde(untagged)]
    pub enum GetFriendsRequest {
        SteamID(SteamID),
        Object {
            steam_id: SteamID,
            #[serde(default)]
            get_info: bool,
        },
    }
}
pub use requests::*;

pub mod responses {
    use rocket::serde::Serialize;
    use std::collections::HashMap;
    use super::*;

    #[derive(Serialize)]
    #[serde(untagged)]
    pub enum GetFriendsResponse {
        SteamIDs(Vec<SteamID>),
        SteamUsers(HashMap<SteamID, SteamUser>),
    }

    #[derive(Serialize)]
    #[serde(untagged)]
    pub enum ResolveVanityURLResponse {
        UserID(SteamID),
        UserInfo(SteamUser),
    }
}
pub use responses::*;

pub mod frontend;
pub mod backend;

pub use frontend::routes;