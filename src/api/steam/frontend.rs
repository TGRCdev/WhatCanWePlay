use super::{
    SteamID, SteamUser,
    backend::{ SteamError, SteamClient },
};
use rocket::{
    serde::{
        Serialize,
        json::Json,
    },
    response::Responder,
    http::Status,
    State, Route,
};
use std::collections::HashMap;

// Errors returned from the WCWP Steam API
#[derive(Debug, Serialize)]
#[serde(tag = "type")]
pub enum APIError {
    /// The steam user with the given Steam id has an inaccessible games list
    NonPublicUser {
        id: SteamID,
    },

    /// This WhatCanWePlay instance doesn't have a Steam webkey defined
    MissingWebKey,

    /// This WhatCanWePlay instance has an invalid Steam webkey defined
    BadWebKey,

    /// Steam returned an unparseable response
    BadSteamResponse,

    /// Steam had an unhandled error code response
    SteamErrorStatus,

    /// User has their friends list set to private or friends-only
    PrivateFriendsList,

    /// The request to Steam returned a 404/500/502/503/504 error code
    SteamUnavailable,
}

impl From<SteamError> for APIError
{
    fn from(err: SteamError) -> Self {
        match err {
            SteamError::NonPublicUser { id } => Self::NonPublicUser{ id },
            SteamError::MissingWebKey => Self::MissingWebKey,
            SteamError::BadWebKey => Self::BadWebKey,
            SteamError::ClientBuildError(_) => panic!("API should not ever return ClientBuildError") ,
            SteamError::BadSteamResponse(_) => Self::BadSteamResponse,
            SteamError::SteamUnavailable => Self::SteamUnavailable,
            SteamError::SteamErrorStatus {..} => APIError::SteamErrorStatus,
            SteamError::PrivateFriendsList => Self::PrivateFriendsList,
        }
    }
}

impl<'r, 'o: 'r> Responder<'r, 'o> for APIError {
    fn respond_to(self, request: &'r rocket::Request<'_>) -> rocket::response::Result<'o> {
        match self {
            APIError::NonPublicUser {..}        =>   (Status::Forbidden, Json(self)).respond_to(request),
            APIError::MissingWebKey             =>   (Status::InternalServerError, Json(self)).respond_to(request),
            APIError::BadWebKey                 =>   (Status::InternalServerError, Json(self)).respond_to(request),
            APIError::BadSteamResponse          =>   (Status::InternalServerError, Json(self)).respond_to(request),
            APIError::SteamErrorStatus          =>   (Status::InternalServerError, Json(self)).respond_to(request),
            APIError::PrivateFriendsList        =>   (Status::Forbidden, Json(self)).respond_to(request),
            APIError::SteamUnavailable          =>   (Status::InternalServerError, Json(self)).respond_to(request),
        }
    }
}

/// Result type for returning to the HTTP client
pub type APIResult<T> = std::result::Result<Json<T>, APIError>;

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

pub fn routes() -> Vec<Route>
{
    routes![
        get_steam_users_info
    ]
}