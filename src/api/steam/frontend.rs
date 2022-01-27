use super::{
    SteamID, SteamUser,
    backend::{ SteamError, SteamClient },
    responses::*,
    requests::*,
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
use std::{collections::HashMap};

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

    // The given vanity url could not be resolved
    VanityUrlNotFound,

    // The ID string the user gave can't be interpreted into a Steam ID
    UnresolvableID,

    // The given Steam ID did not return a valid user
    UserNotFound,
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
            SteamError::VanityUrlNotFound => Self::VanityUrlNotFound,
            SteamError::UserNotFound => Self::UserNotFound,
        }
    }
}

impl<'r, 'o: 'r> Responder<'r, 'o> for APIError {
    fn respond_to(self, request: &'r rocket::Request<'_>) -> rocket::response::Result<'o> {
        match self {
            APIError::NonPublicUser {..}        =>  (Status::Forbidden, Json(self)).respond_to(request),
            APIError::MissingWebKey             =>  (Status::InternalServerError, Json(self)).respond_to(request),
            APIError::BadWebKey                 =>  (Status::InternalServerError, Json(self)).respond_to(request),
            APIError::BadSteamResponse          =>  (Status::InternalServerError, Json(self)).respond_to(request),
            APIError::SteamErrorStatus          =>  (Status::InternalServerError, Json(self)).respond_to(request),
            APIError::PrivateFriendsList        =>  (Status::Forbidden, Json(self)).respond_to(request),
            APIError::SteamUnavailable          =>  (Status::InternalServerError, Json(self)).respond_to(request),
            APIError::VanityUrlNotFound         =>  (Status::NotFound, Json(self)).respond_to(request),
            APIError::UserNotFound              =>  (Status::NotFound, Json(self)).respond_to(request),
            APIError::UnresolvableID            =>  (Status::NotFound, Json(self)).respond_to(request),
        }
    }
}

/// Result type for returning to the HTTP client
pub type APIResult<T> = std::result::Result<Json<T>, APIError>;

#[post("/get_steam_users_info", data = "<steam_ids>")]
pub async fn get_steam_users_info(steam_ids: Json<GetSteamUsersInfoRequest>, client: &State<SteamClient>) -> APIResult<HashMap<SteamID, SteamUser>>
{
    let steam_ids = match &*steam_ids {
        GetSteamUsersInfoRequest::Vec(steam_ids) => steam_ids,
        GetSteamUsersInfoRequest::Object { steam_ids } => steam_ids,
        GetSteamUsersInfoRequest::Single(steam_id) => core::slice::from_ref(steam_id),
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

#[post("/get_steam_user_info", data = "<steam_id>")]
pub async fn get_steam_user_info(steam_id: Json<SteamID>, client: &State<SteamClient>) -> APIResult<SteamUser>
{
    let response = client.get_player_summary(&steam_id).await;
    match response {
        Ok(steam_user) => Ok(steam_user.into()),
        Err(err) => {
            error!("{:#?}", err);
            Err(err.into())
        }
    }
}

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

    let result = client.get_friends_list(steam_id).await?;

    if !get_info
    {
        Ok(GetFriendsResponse::SteamIDs(result).into())
    }
    else
    {
        Ok(GetFriendsResponse::SteamUsers(
            client.get_player_summaries(&result).await?
        ).into())
    }
}

#[post("/interpret_id_input", data = "<id_str>")]
pub async fn interpret_id_input(mut id_str: String, client: &State<SteamClient>) -> APIResult<SteamUser>
{
    // If a URL was entered
    if let Some((_, new_id)) = id_str.rsplit_once('/') {
        id_str = new_id.to_string();
    }

    // Try a Steam ID
    if let Ok(steam_id) = id_str.parse()
    {
        let result = client.get_player_summary(&steam_id).await;

        if let Ok(user_info) = result
        {
            return Ok(user_info.into());
        }
    }

    println!("{}", id_str);

    // Not a Steam ID, maybe a vanity URL
    let result = client.resolve_vanity_url(&id_str).await.map_err(|e| {
        match e {
            SteamError::VanityUrlNotFound => APIError::UnresolvableID,
            _ => e.into(),
        }
    })?;

    let info = client.get_player_summary(&result).await?;

    Ok(info.into())
}

pub fn routes() -> Vec<Route>
{
    routes![
        get_steam_users_info,
        get_steam_user_info,
        get_friends_list,
        interpret_id_input,
    ]
}