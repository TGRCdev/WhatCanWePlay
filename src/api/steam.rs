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
};
use reqwest::{ ClientBuilder, Client, StatusCode };
use itertools::Itertools;
use figment::Figment;

use crate::structs::SteamUser;

// TODO: Documentation

#[derive(Debug, Serialize)]
#[serde(tag = "type")]
pub enum SteamError {
    NonPublicUser {
        id: u64
    },
    MissingWebKey,
    BadWebKey,
    ClientBuildError(String),
    BadSteamResponse(String),
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

pub struct SteamClient(Client, String);
impl Deref for SteamClient {
    type Target = Client;

    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub type APIResult<T> = std::result::Result<Json<T>, Custom<Json<SteamError>>>;
pub type SteamResult<T> = std::result::Result<T, SteamError>;

impl SteamClient {
    pub async fn new(figment: &Figment) -> SteamResult<SteamClient> {
        let webkey: String = figment
            .extract_inner("steam_webkey")
            .map_err(|_err| SteamError::MissingWebKey)?;

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

    pub async fn get_player_summaries(&self, steam_ids: &[u64]) -> std::result::Result<Vec<SteamUser>, SteamError> {
        let webkey = &self.1;
        let steam_ids = steam_ids.iter().format(",").to_string();
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
        
        let players = result_json["response"]["players"].take(); // Potentially the list of players

        match serde_json::from_value::<Vec<SteamUser>>(players)
        {
            Ok(players) => Ok(players),
            Err(err) => Err(SteamError::BadSteamResponse(err.to_string()))
        }
    }
}

#[post("/get_steam_users_info", data = "<steam_ids>")]
pub async fn get_steam_users_info(steam_ids: Json<Vec<u64>>, client: &State<SteamClient>) -> APIResult<Vec<SteamUser>>
{
    if steam_ids.is_empty()
    {
        return Ok(vec![].into());
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