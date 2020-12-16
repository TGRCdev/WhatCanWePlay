use serde::{Serialize, Deserialize};

const API_URL: &str = "https://api.steampowered.com/";

#[derive(Debug, Serialize, Deserialize)]
pub struct SteamUser {
    #[serde(rename(deserialize = "steamid"))]
    #[serde(deserialize_with = "u64_string_parse")]
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
    #[serde(deserialize_with = "bool_from_int")]
    pub online: bool
}

use crate::errors::SteamError;

use reqwest;
use reqwest::{StatusCode, Url};

use std::fmt::Write;
use std::collections::HashSet;

use serde::de::{self, Deserializer};

fn bool_from_int<'de, D>(deserializer: D) -> Result<bool, D::Error>
where
    D: Deserializer<'de>,
{
    match u8::deserialize(deserializer)? {
        0 => Ok(false),
        _ => Ok(true),
    }
}

fn u64_string_parse<'de, D>(deserializer: D) -> Result<u64, D::Error>
where
    D: Deserializer<'de>,
{
    let val : serde_json::Value = Deserialize::deserialize(deserializer)?;
    match val {
        serde_json::Value::Number(num) => {
            if let Some(num) = num.as_u64() {
                return Ok(num);
            }
        },
        serde_json::Value::String(string) => {
            if let Ok(num) = string.parse() {
                return Ok(num);
            }
        }
        _ => {}
    }

    return Err(de::Error::custom(&"expected u64 or stringified u64"));
}

pub fn get_steam_users_info(webkey: &str, steamids: &[u64]) -> Result<Vec<SteamUser>, SteamError> {
    if steamids.is_empty() {
        return Ok(Vec::new());
    }
    let client = reqwest::blocking::Client::new();
    
    let mut id_str = steamids[0].to_string();

    for n in steamids[1..].iter()
    {
        write!(&mut id_str, ",{}", n).unwrap();
    }

    let base_url = Url::parse(API_URL).unwrap();

    let response = client.get(base_url.join("ISteamUser/GetPlayerSummaries/v2/").unwrap())
        .query(&[
            ("key", webkey),
            ("format", "json"),
            ("steamids", &id_str)
        ])
        .send();
    
    if let Err(e) = response {
        return Err(SteamError::UnknownError(e.to_string()));
    }

    let response = response.unwrap();

    if !response.status().is_success() {
        match response.status() {
            StatusCode::BAD_GATEWAY | StatusCode::INTERNAL_SERVER_ERROR =>
                return Err(SteamError::ServerError),
            StatusCode::FORBIDDEN =>
                return Err(SteamError::BadWebkey),
            _ =>
                return Err(SteamError::UnknownError(response.text().unwrap_or("Unknown error".to_string())))
        }
    }

    let response_json = response.json();
    if let Err(_) = response_json {
        return Err(SteamError::BadResponse); // TODO: Turn BadResponse into BadResponse(String, String) to hold response and error text?
    }
    let mut response: serde_json::Value = response_json.unwrap();
    let players = &mut response["response"]["players"];
    
    let mut users = Vec::new();

    if players.is_array() {
        for player_json in players.as_array_mut().unwrap().iter_mut() {
            let user_info: Option<SteamUser> = serde_json::from_value(player_json.take()).unwrap();
            
            if let Some(user) = user_info {
                users.push(user);
            }
        }
    }

    return Ok(users);
}

///
/// # Errors
///
/// `SteamError::BadWebkey` is returned if the provided webkey is invalid
///
/// `SteamError::ServerError` is returned if the given steamid does not exist, or if the server had an error processing the request.
/// (We can't differentiate between the two, they're both returned as 500 status code)
pub fn get_owned_steam_games(webkey: &str, steamid: u64) -> Result<HashSet<u64>, SteamError> {
    let base_url = Url::parse(API_URL).unwrap();
    let client = reqwest::blocking::Client::new();
    let response = client.get(base_url.join("IPlayerService/GetOwnedGames/v0001/").unwrap())
        .query(&[
            ("key", webkey),
            ("steamid", &steamid.to_string()),
            ("include_appinfo", "false"),
            ("include_played_free_games", "true"),
            ("format", "json")
        ]).send();
    
    if let Err(e) = response {
        return Err(SteamError::UnknownError(e.to_string()));
    }

    let response = response.unwrap();

    if !response.status().is_success() {
        match response.status() {
            StatusCode::UNAUTHORIZED =>
                return Err(SteamError::BadWebkey),
            StatusCode::INTERNAL_SERVER_ERROR =>
                return Err(SteamError::ServerError),
            _ =>
                return Err(SteamError::UnknownError(response.text().unwrap_or("Unknown error".to_string()))),
        }
    }

    let response_json = response.json();
    if let Err(_) = response_json {
        return Err(SteamError::BadResponse);
    }

    let response_json: serde_json::Value = response_json.unwrap();

    let mut app_ids = HashSet::new();

    let games_arr = &response_json["response"]["games"];
    if let Some(games) = games_arr.as_array() {
        for game in games {
            let id = &game["appid"];
            if let Some(id) = id.as_u64() {
                app_ids.insert(id);
            }
        }
    }

    return Ok(app_ids);
}

pub fn get_friend_list(webkey: &str, steamid: u64) -> Result<Vec<SteamUser>, SteamError>
{
    let base_url = Url::parse(API_URL).unwrap();
    let client = reqwest::blocking::Client::new();
    let response = client.get(base_url.join("ISteamUser/GetFriendList/v0001/").unwrap())
        .query(&[
            ("key", webkey),
            ("steamid", &steamid.to_string()),
            ("relationship", "friend"),
            ("format", "json")
        ]).send();
    
    if let Err(e) = response {
        return Err(SteamError::UnknownError(e.to_string()));
    }

    let response = response.unwrap();

    if !response.status().is_success() {
        match response.status() {
            StatusCode::UNAUTHORIZED =>
                return Err(SteamError::BadWebkey),
            StatusCode::INTERNAL_SERVER_ERROR =>
                return Err(SteamError::ServerError),
            _ =>
                return Err(SteamError::UnknownError(response.text().unwrap_or("Unknown error".to_string()))),
        }
    }

    let response_json = response.json();
    if let Err(_) = response_json {
        return Err(SteamError::BadResponse);
    }

    let response_json: serde_json::Value = response_json.unwrap();

    let friendslist = &response_json["friendslist"]["friends"];

    let mut user_ids = Vec::new();

    if let Some(friendslist) = friendslist.as_array()
    {
        for friend in friendslist
        {
            let id_val = &friend["steamid"];
            match id_val {
                serde_json::Value::Number(num) => {
                    if let Some(num) = num.as_u64()
                    {
                        user_ids.push(num);
                    }
                },
                serde_json::Value::String(numstr) => {
                    if let Ok(num) = numstr.parse() {
                        user_ids.push(num);
                    }
                },
                _ => {},
            }
        }
    }

    if user_ids.is_empty() {
        return Ok(Vec::new())
    }

    let friends_info = get_steam_users_info(webkey, &user_ids)?;

    return Ok(friends_info);
}

pub fn intersect_owned_game_ids(webkey: &str, steamids: &[u64])-> Result<HashSet<u64>, SteamError>
{
    if steamids.is_empty()
    {
        return Ok(HashSet::new());
    }

    let mut games_set = get_owned_steam_games(webkey, steamids[0])?;

    for &id in steamids[1..].iter() {
        let next_set = get_owned_steam_games(webkey, id)?;
        games_set = &games_set & &next_set; // Intersect the two sets

        if games_set.is_empty()
        { // No common owned games
            return Ok(HashSet::new());
        }
    }

    return Ok(games_set);
}