use rocket::{
    serde::{
        json::{ Json, Value },
        Serialize,
    },
    response::status::Custom,
    State,
    http::Status,
};
use std::{
    ops::Deref,
    time::Duration,
    collections::HashMap,
};
use reqwest::{ ClientBuilder, Client, StatusCode };
use itertools::Itertools;
use figment::Figment;

use crate::structs::SteamUser;

pub type SteamID = u64;

/// Various errors that can occur with the SteamClient
#[derive(Debug, Serialize)]
#[serde(tag = "type")]
pub enum SteamError {
    /// The steam user with the given Steam id has an inaccessible games list
    NonPublicUser {
        id: SteamID,
    },

    /// This WhatCanWePlay instance doesn't have a Steam webkey defined
    MissingWebKey,

    /// This WhatCanWePlay instance has an invalid Steam webkey defined
    BadWebKey,

    /// An error occurred while constructing the Steam request client
    ClientBuildError(String),

    /// Steam returned an unparseable response
    BadSteamResponse(String),

    /// Steam had an unhandled error code response
    SteamErrorStatus{
        code: u16,
        message: String,
    },
}

impl From<SteamError> for Custom<Json<SteamError>>
{
    fn from(err: SteamError) -> Self {
        match err {
            SteamError::NonPublicUser{..} => Custom(Status::Forbidden, err.into()),
            SteamError::BadSteamResponse(_) => Custom(Status::InternalServerError, err.into()),
            SteamError::SteamErrorStatus{code, ..} => Custom(Status::new(code), err.into()),

            // These won't ever be returned by the API
            SteamError::MissingWebKey => Custom(Status::InternalServerError, err.into()),
            SteamError::BadWebKey => Custom(Status::InternalServerError, err.into()),
            SteamError::ClientBuildError(_) => Custom(Status::InternalServerError, err.into()),
        }
    }
}

impl From<reqwest::Error> for SteamError
{
    fn from(err: reqwest::Error) -> Self {
        Self::SteamErrorStatus {
            code: err.status().unwrap_or(StatusCode::INTERNAL_SERVER_ERROR).as_u16(),
            message: err.status().map(|s| s.to_string()).unwrap_or_default(),
        }
    }
}

/// Deref wrapper for a reqwest Client that provides shortcut functions to Steam API endpoints
pub struct SteamClient(Client, String);
impl Deref for SteamClient {
    type Target = Client;

    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// Result type for returning to the HTTP client
pub type APIResult<T> = std::result::Result<Json<T>, Custom<Json<SteamError>>>;
/// Result type for processing results from Steam API
pub type SteamResult<T> = std::result::Result<T, SteamError>;


impl SteamClient {
    /// Attempt to construct a new SteamClient from the given Figment config
    /// 
    /// Will return Err if the config doesn't contain a key named 'steam_webkey',
    /// or if the webkey fails to authenticate with Steam API
    pub async fn new(figment: &Figment) -> SteamResult<SteamClient> {
        let webkey: String = figment
            .extract_inner("steam_webkey")
            .map_err(|_err| SteamError::MissingWebKey)?;
        // Webkey acquired

        // Construct HTTP client for Steam API
        let client = ClientBuilder::new()
            .user_agent(concat!(
                "WhatCanWePlay/",
                env!("CARGO_PKG_VERSION"),
            ))
            .timeout(Duration::from_secs(10))
            .gzip(true)
            .brotli(true)
            .deflate(true)
            .build()
            .map(|client| SteamClient(client, webkey))
            .map_err(|err| SteamError::ClientBuildError(err.to_string()))?;
        
        // Establish connection, test webkey
        let test = client.get_player_summaries(&[76561197960435530]).await; // Robin Walker
        match test {
            Ok(_) => Ok(client),
            Err(err) => {
                error!("{:#?}", err);
                Err(err)
            },
        }
    }

    /// Fetch the Steam profiles of the given Steam IDs
    pub async fn get_player_summaries(&self, steam_ids: &[SteamID]) -> std::result::Result<HashMap<SteamID, SteamUser>, SteamError> {
        if steam_ids.is_empty()
        {
            return Ok(Default::default())
        }

        let webkey = &self.1;
        let steam_ids = steam_ids.iter().format(",").to_string(); // URL format
        let result = self.get(
            "https://api.steampowered.com/ISteamUser/GetPlayerSummaries/v0002"
        )
        .query(&[
            ("key", webkey),
            ("steamids", &steam_ids),
        ]).send().await;

        // Check the error code
        let result = result.map_err(|err| {
            if err.status() == Some(StatusCode::FORBIDDEN)
            {
                SteamError::BadWebKey
            }
            else {
                SteamError::SteamErrorStatus{
                    code: err.status().unwrap_or(StatusCode::INTERNAL_SERVER_ERROR).as_u16(),
                    message: err.status().map(|e| e.to_string()).unwrap_or_default(),
                }
            }
        })?;

        if !result.status().is_success()
        {
            let code = result.status().as_u16();
            return Err(SteamError::SteamErrorStatus{
                code,
                message: result.text().await.unwrap_or(format!("Steam returned an empty response with code {}", code)),
            })
        }

        // Try to deserialize
        let mut result_json: Value = result.json().await // Top-level value
            .map_err(|err| SteamError::BadSteamResponse(err.to_string()))?;
        
        // Potentially the list of players
        let players = result_json["response"]["players"].take();

        // Confirmed as the list of players
        let players: Vec<SteamUser> = serde_json::from_value(players).map_err(|e| SteamError::BadSteamResponse(e.to_string()))?;

        // Convert to mapping
        let player_map: HashMap<SteamID,SteamUser> = players.into_iter().map(|user| (user.steam_id, user)).collect();

        Ok(player_map)
    }
}

#[post("/get_steam_users_info", data = "<steam_ids>")]
pub async fn get_steam_users_info(steam_ids: Json<Vec<SteamID>>, client: &State<SteamClient>) -> APIResult<HashMap<SteamID, SteamUser>>
{
    if steam_ids.is_empty()
    {
        return Ok(HashMap::default().into());
    }

    let response = client.get_player_summaries(&steam_ids).await;
    match response {
        Ok(info) => Ok(info.into()),
        Err(err) => {
            error!("{:#?}", err);
            Err(err.into())
        }
    }
}