use rocket::{
    serde::{
        json::Json,
        Serialize
    },
    response::status::Custom,
    http::Status,
};

use crate::structs::SteamUser;

#[derive(Serialize)]
#[serde(tag = "type")]
pub enum SteamError {
    NonPublicUser{
        id: u64
    },
}

pub type Result<T> = std::result::Result<Json<T>, Custom<Json<SteamError>>>;

#[post("/get_steam_users_info", data = "<steam_ids>")]
pub fn get_steam_users_info<'r>(steam_ids: Json<Vec<u64>>) -> Result<Vec<SteamUser<'r>>>
{
    if steam_ids.is_empty()
    {
        return Ok(vec![].into());
    }

    Err(Custom(
        Status::Forbidden,
        SteamError::NonPublicUser{id: 456}.into()
    ))
}