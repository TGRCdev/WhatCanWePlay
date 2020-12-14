use reqwest;
use serde::{Deserialize, Serialize};
use serde_json;
use std::collections::HashSet;
use std::fmt::{Debug, Write};
use crate::errors::IGDBError;
use reqwest::StatusCode;
use serde_json::Value;
use std::convert::{TryFrom, TryInto};
use std::cmp::{max, min};

const API_URL : &str = "https://api.igdb.com/v4/";
const TOKEN_URL : &str = "https://id.twitch.tv/oauth2/token";

#[derive(Serialize, Deserialize, Debug)]
/// Struct for retrieving a bearer token from the Twitch Developer API
pub struct Token {
    pub access_token: String,
    pub expires_in: u64
}

#[derive(Debug, Serialize)]
/// Struct for games described by IGDB
pub struct GameInfo {
    pub steam_id: u64,
    pub igdb_id: u64,
    pub name: String,
    pub supported_players: u8,
    
    /// Cover image url
    ///
    /// Usage: `https://images.igdb.com/igdb/image/upload/t_cover_small/<cover_id>.jpg`
    pub cover_id: String, 

    pub has_multiplayer: bool,
}

/// Takes a serde_json Value and tries to get an i64
/// either by parsing an i64 directly, or by parsing
/// a stringified i64.
fn parse_u8_from_value(value: &Value) -> Option<u8> {
    let num = parse_u64_from_value(value);
    match num {
        Some(num) =>
            return num.try_into().ok(),
        None =>
            return None
    }
}

/// Takes a serde_json Value and tries to get an i64
/// either by parsing an u64 directly, or by parsing
/// a stringified u64.
fn parse_u64_from_value(value: &Value) -> Option<u64> {
    match value {
        Value::Number(num) => {
            return num.as_u64();
        },
        Value::String(string) => {
            return string.parse().ok();
        },
        _ => return None
    }
}

impl TryFrom<&Value> for GameInfo {
    type Error = IGDBError;

    /// Used to extract game information from IGDB query results
    fn try_from(value: &Value) -> Result<Self, IGDBError> {
        let steam_id : u64;
        match parse_u64_from_value(&value["uid"]) {
            Some(id) => steam_id = id,
            None => return Err(IGDBError::BadResponse)
        }

        let game = &value["game"];
        if !game.is_object()
        {
            return Err(IGDBError::BadResponse);
        }

        let igdb_id : u64;
        match parse_u64_from_value(&game["id"]) {
            Some(id) => igdb_id = id,
            None => return Err(IGDBError::BadResponse)
        }

        let name : &str;
        match game["name"].as_str() {
            Some(val) => name = val,
            None => return Err(IGDBError::BadResponse),
        }

        let cover_id : &str;
        match game["cover"]["image_id"].as_str() {
            Some(val) => cover_id = val,
            None => cover_id = "",
        }

        // Figure out multiplayer and whatnot

        let game_modes = &game["game_modes"];
        let has_multiplayer: bool;
        let supported_players : u8;

        if game_modes.is_array() {
            let gamemode_arr = game_modes.as_array().unwrap();
            
            has_multiplayer = gamemode_arr.iter().any(|x| x == 5 || x == 2); // TODO: This isn't really needed with game["multiplayer_modes"], replace it
            if has_multiplayer
            {
                let multiplayer_modes = &game["multiplayer_modes"];
                if let Some(multimodes) = multiplayer_modes.as_array() {
                    let mut mostplayers = 0;
                    for mode in multimodes.iter() {
                        let onlinemax : u8 = parse_u8_from_value(&mode["onlinemax"]).unwrap_or(0);
                        let coopmax : u8 = parse_u8_from_value(&mode["onlinecoopmax"]).unwrap_or(0);
                        mostplayers = max(onlinemax, coopmax);
                    }
                    supported_players = max(min(255u8, mostplayers), 0u8).try_into().unwrap();
                }
                else
                {
                    supported_players = 0; // Displayed as ? for number of supported players
                }
            }
            else
            {
                supported_players = 1;
            }
        }
        else
        {
            has_multiplayer = false;
            supported_players = 1;
        }

        return Ok(GameInfo {
            steam_id,
            igdb_id,
            name: name.to_string(),
            cover_id: cover_id.to_string(),
            supported_players,
            has_multiplayer,
        });
    }
}

/// Fetches a Twitch app bearer token for use with the IGDB API.
///
/// # Errors
///
/// `IGDBError::BadClient` is returned if the client ID is invalid.
///
/// `IGDBError::BadSecret` is returned if the client secret is invalid.
///
/// `IGDBError::ServerError` is returned if Twitch returned an error code.
///
/// `IGDBError::UnknownError` is returned for any unexpected status codes, or if Twitch was unreachable.
pub fn get_twitch_token(client_id: &str, secret: &str) -> Result<Token, IGDBError> {
    let client = reqwest::blocking::Client::new();

    let res = client.post(TOKEN_URL)
        .query(&[
            ("client_id", client_id),
            ("client_secret", secret),
            ("grant_type", "client_credentials")
        ]).send();
    
    if let Err(e) = res {
        return Err(IGDBError::UnknownError(e.to_string()));
    }

    let res = res.unwrap();
    
    if !res.status().is_success() {
        match res.status() {
            StatusCode::BAD_REQUEST => 
                return Err(IGDBError::BadClient),
            StatusCode::FORBIDDEN => 
                return Err(IGDBError::BadSecret),
            StatusCode::INTERNAL_SERVER_ERROR | StatusCode::BAD_GATEWAY => 
                return Err(IGDBError::ServerError),
            _ =>
                return Err(IGDBError::UnknownError(res.text().unwrap_or("Unknown error".to_string())))
        }
    }
    
    let response = res.text().expect("Reqwest errored while getting response text");
    let t: Token = serde_json::from_str(&response).expect(&format!("Serde failed to load JSON response from IGDB (response: {})", response));
    
    return Ok(t);
}

/// Fetch the info for the provided set of steam app IDs
///
/// Returns a tuple of two `Vec`s, a `Vec` containing the info of found games, and a `Vec` of app IDs not found.
/// If `appids` is empty, returns two empty `Vec`s.
///
/// # Errors
///
/// `IGDBError::BadAuth` is returned if either the `client_id` or `bearer_token` are invalid.
/// 
/// `IGDBError::ServerError` is returned if IGDB was unable to process the request.
/// 
///`IGDBError::UnknownError` is returned for any unexpected status codes, or if IGDB was unreachable.
pub fn get_steam_game_info(client_id: &str, bearer_token: &str, appids: &[u64]) -> Result<(Vec<GameInfo>, HashSet<u64>), IGDBError> {
    if appids.is_empty()
    {
        return Ok((Vec::new(), HashSet::new()));
    }

    let client = reqwest::blocking::Client::new();
    let mut games_info : Vec<GameInfo> = Vec::new();
    let mut not_found : HashSet<u64> = HashSet::with_capacity(appids.len());

    for current_slice in appids.chunks(500)
    {
        let mut id_str = current_slice[0].to_string();

        for n in current_slice[1..].iter()
        {
            write!(&mut id_str, ",{}", n).unwrap();
        }

        let response = client.post(&format!("{}{}", API_URL, "external_games"))
            .header("Client-ID", client_id)
            .header("Authorization", format!("Bearer {}", bearer_token))
            .header("Accept", "application/json")
            .body(format!(
                "fields uid,game.name,game.game_modes,game.multiplayer_modes.onlinemax,
                game.multiplayer_modes.onlinecoopmax,game.cover.image_id; 
                where uid = ({}) & category = 1; limit {};",
                id_str, current_slice.len()
            )).send();

            if let Err(e) = response {
                return Err(IGDBError::UnknownError(e.to_string()));
            }

            let response = response.unwrap();
        
            if !response.status().is_success() {
                match response.status() {
                    StatusCode::UNAUTHORIZED => {
                        return Err(IGDBError::BadClient);
                    },
                    StatusCode::FORBIDDEN => {
                        return Err(IGDBError::BadAuth);
                    },
                    StatusCode::INTERNAL_SERVER_ERROR | StatusCode::BAD_GATEWAY =>
                        return Err(IGDBError::ServerError),
                    _ =>
                        return Err(IGDBError::UnknownError(
                                response.text().unwrap_or("Unknown error".to_string()))
                        ),
                }
            }

            for id in current_slice.iter() 
                { not_found.insert(*id); }

            if let Ok(games_json) = response.json::<Value>() {
                if let Some(games) = games_json.as_array() {
                    for game in games.iter() {
                        if let Ok(game_info) = GameInfo::try_from(game) {
                            not_found.remove(&game_info.steam_id);
                            games_info.push(game_info);
                        }
                    }
                }
            }
        }

        return Ok((games_info, not_found));
}

#[test]
fn unwrap_token() {
    let test_token = "
    {
        \"access_token\": \"thisisnotarealtokenbutpretenditis\",
        \"refresh_token\": \"\",
        \"expires_in\": 3600,
        \"scope\": [],
        \"token_type\": \"bearer\"
    }";

    let t: Token = serde_json::from_str(test_token).unwrap();

    println!("{:#?}", t);
}

#[test]
fn unwrap_game() {
    let games = "[{\"uid\": \"1\", \"game\":{\"id\": 5, \"name\":\"Test\"}}]";

    let games_json: Value = serde_json::from_str(games).unwrap();

    for game in games_json.as_array().unwrap()
    {
        let g: GameInfo = game.try_into().unwrap();
        println!("{:#?}", g);
    }
}