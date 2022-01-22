use super::{ 
    SteamID, SteamUser,
    GetFriendsResponse
};
use reqwest::{
    StatusCode, Client, ClientBuilder,
    Response,
};
use std::{
    ops::Deref,
    time::Duration,
    collections::HashMap,
    borrow::Cow,
};
use itertools::Itertools;
use serde_json::Value;
use rocket::{
    fairing::{ self, Fairing, Kind, Info },
    Rocket, Build,
};

/// Various errors that can occur with the SteamClient
#[derive(Debug)]
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

    /// The request to Steam returned a 404/500/502/503/504 error code
    SteamUnavailable,

    /// Steam had an unhandled error code response
    SteamErrorStatus{
        code: u16,
        message: String,
    },

    /// User has their friends list set to private or friends-only
    PrivateFriendsList,
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

/// Result type for processing results from Steam API
pub type SteamResult<T> = std::result::Result<T, SteamError>;

/// Deref wrapper for a reqwest Client that provides shortcut functions to Steam API endpoints
pub struct SteamClient(Client, Cow<'static, str>);
impl Deref for SteamClient {
    type Target = Client;

    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug, Default)]
pub struct SteamClientFairing;

#[rocket::async_trait]
impl Fairing for SteamClientFairing {
    fn info(&self) -> Info {
        Info {
            name: "Steam Client",
            kind: Kind::Ignite | Kind::Singleton,
        }
    }

    /// Retrieves the Steam webkey under the key `steam_webkey`,
    /// constructs a SteamClient, tests the webkey with the
    /// Steam API and hands the SteamClient to Rocket
    async fn on_ignite(&self, rocket: Rocket<Build>) -> fairing::Result {
        use rocket::{
            yansi::{ Paint, Color },
            log::PaintExt,
        };

        info!("{}{}:", Paint::emoji("💨 "), Paint::magenta("Steam Client"));
        let figment = rocket.figment();
        let webkey = figment.extract_inner("steam_webkey");
        if let Err(err) = webkey
        {
            info_!("Found webkey: {}{}", Paint::emoji("❌ ").fg(Color::Red), Paint::red("Fail"));
            error!("Failed to load the Steam webkey: {}", err);
            return Err(rocket)
        }
        let webkey = webkey.unwrap();
        info_!("Found webkey: {}{}", Paint::emoji("✔ ").fg(Color::Green), Paint::green("Pass"));
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
            .map(|client| SteamClient(client, webkey));
        if let Err(err) = client
        {
            info_!("Constructed Steam Client: {}{}", Paint::emoji("❌ ").fg(Color::Red), Paint::red("Fail"));
            error!("Failed to initialize the SteamClient: {}", err);
            return Err(rocket)
        }
        let client = client.unwrap();
        info_!("Constructed Steam Client: {}{}", Paint::emoji("✔ ").fg(Color::Green), Paint::green("Pass"));

        let skip_steam_test = figment.extract_inner("skip_steam_test").unwrap_or_else(|_err| {
            warn!("skip_steam_test failed to deserialize properly, defaulting to 'false'");
            warn!("Error: {}", _err);
            false
        });
        
        // Establish connection, test webkey
        if skip_steam_test
        {
            info_!("Webkey test: {}{}", Paint::emoji("⏭ ").fg(Color::Yellow), Paint::yellow("Skip"));
            Ok(rocket.manage(client))
        }
        else
        {
            let test = client.get_player_summaries(&[76561197960435530]).await; // Robin Walker
            match test {
                Ok(_) => {
                    info_!("Webkey test: {}{}", Paint::emoji("✔ ").fg(Color::Green), Paint::green("Pass"));
                    Ok(rocket.manage(client))
                }
                Err(err) => {
                    info_!("Webkey test: {}{}", Paint::emoji("❌ ").fg(Color::Red), Paint::red("Fail"));
                    error!("Steam webkey test failed: {:?}", err);
                    Err(rocket)
                },
            }
        }
    }
}

impl SteamClient {
    async fn common_steam_errors(result: Response) -> SteamResult<Response>
    {
        if !result.status().is_success()
        {
            match result.status()
            {
                StatusCode::FORBIDDEN => return Err(SteamError::BadWebKey),
                StatusCode::INTERNAL_SERVER_ERROR |
                    StatusCode::BAD_GATEWAY |
                    StatusCode::SERVICE_UNAVAILABLE |
                    StatusCode::GATEWAY_TIMEOUT => return Err(SteamError::SteamUnavailable),
                _ => {
                    let code = result.status().as_u16();
                    return Err(SteamError::SteamErrorStatus{
                        code,
                        message: result.text().await.unwrap_or(format!("Steam returned an empty response with code {}", code)),
                    })
                }
            }
        }

        Ok(result)
    }

    #[inline(always)]
    pub fn fairing() -> SteamClientFairing
    {
        Default::default()
    }

    /// Fetch the Steam profiles of the given Steam IDs
    pub async fn get_player_summaries(&self, steam_ids: &[SteamID]) -> SteamResult<HashMap<SteamID, SteamUser>> {
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
            ("key", webkey.as_ref()),
            ("steamids", steam_ids.as_str()),
            ("format", "json"),
        ]).send().await?;

        let result = Self::common_steam_errors(result).await?;

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

    pub async fn get_friends_list(&self, user: SteamID, get_info: bool) -> SteamResult<GetFriendsResponse>
    {
        let webkey = &self.1;
        let user_str = user.to_string();
        let result = self.get(
            "http://api.steampowered.com/ISteamUser/GetFriendList/v0001/"
        )
        .query(&[
            ("key", webkey.as_ref()),
            ("steamid", user_str.as_str()),
            ("relationship", "friend"),
            ("format", "json"),
        ]).send().await?;

        let result = Self::common_steam_errors(result).await?;

        let result: Value = result.json().await
            .map_err(|e| SteamError::BadSteamResponse(e.to_string()))?;

        let result = result["friendslist"]["friends"].as_array().ok_or(SteamError::PrivateFriendsList)?;

        let result: Vec<SteamID> = result.iter()
            .filter_map(|user| {
                user["steamid"].as_str().and_then(|s| s.parse().ok())
            }).collect();
        
        if !get_info {
            Ok(GetFriendsResponse::Type1(result))
        }
        else {
            Ok(GetFriendsResponse::Type2(
                self.get_player_summaries(&result).await?
            ))
        }
    }
}