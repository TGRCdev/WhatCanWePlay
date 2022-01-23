use super::{
    SteamID, SteamUser,
    backend::{ SteamError, SteamClient },
    GetFriendsResponse,
};
use rocket::{
    serde::{
        Serialize,
        json::{ Json },
    },
    response::Responder,
    http::Status,
    State, Route,
};
use std::collections::HashMap;
use crate::api::testing::{
    api_test, APITestInfo,
    APITestArgument, APITestArgType::*
};
use core::slice::from_ref;

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

pub mod requests {
    use serde::Deserialize;
    use super::super::SteamID;

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

#[post("/get_steam_users_info", data = "<steam_ids>")]
pub async fn get_steam_users_info(steam_ids: Json<GetSteamUsersInfoRequest>, client: &State<SteamClient>) -> APIResult<HashMap<SteamID, SteamUser>>
{
    let steam_ids = match &*steam_ids {
        GetSteamUsersInfoRequest::Vec(steam_ids) => steam_ids,
        GetSteamUsersInfoRequest::Object { steam_ids } => steam_ids,
        GetSteamUsersInfoRequest::Single(steam_id) => from_ref(steam_id),
    };

    if steam_ids.is_empty()
    {
        return Ok(HashMap::default().into());
    }

    let response = client.get_player_summaries(steam_ids).await;
    match response {
        Ok(info) => Ok(info.into()),
        Err(err) => {
            error!("{:#?}", err);
            Err(err.into())
        }
    }
}
api_test!(
    get_steam_users_info_test,
    "/get_steam_users_info",
    APITestInfo {
        func_name: "get_steam_users_info",
        func_args: vec![
            APITestArgument {
                name: "steam_ids",
                arg_type: CSLOfInt,
                default: None,
            }
        ]
    }
);

#[post("/get_friends_list", data = "<request>")]
pub async fn get_friends_list(request: Json<GetFriendsRequest>, client: &State<SteamClient>) -> APIResult<GetFriendsResponse>
{
    let steam_id;
    let get_info;
    match *request {
        GetFriendsRequest::SteamID(id) => {
            steam_id = id;
            get_info = false;
        },
        GetFriendsRequest::Object { steam_id: id, get_info: info } => {
            steam_id = id;
            get_info = info;
        }
    }

    let result = client.get_friends_list(steam_id, get_info).await?;

    Ok(result.into())
}
api_test!(
    get_friends_list_test,
    "/get_friends_list",
    APITestInfo {
        func_name: "get_friends_list",
        func_args: vec![
            APITestArgument {
                name: "steam_id",
                arg_type: Int,
                default: None,
            },
            APITestArgument {
                name: "get_info",
                arg_type: Bool,
                default: None,
            }
        ]
    }
);

pub fn routes() -> Vec<Route>
{
    routes![
        get_steam_users_info,
        get_steam_users_info_test,
        get_friends_list,
        get_friends_list_test,
    ]
}