#[derive(Debug)]
pub enum IGDBError {
    UnknownError(String), // Unhandled error
    ServerError, // IGDB had an internal error
    BadResponse, // IGDB returned an unparseable response
    BadClient, // The supplied client ID is wrong
    BadSecret, // The supplied client secret is wrong
    BadToken, // The supplied bearer token is wrong
    BadAuth, // Some part of the supplied authentication is wrong
}

use std::fmt;

impl fmt::Display for IGDBError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            IGDBError::ServerError => return write!(f, "IGDB had an internal server error"),
            IGDBError::BadResponse => return write!(f, "IGDB returned an unparseable response"),
            IGDBError::BadClient => return write!(f, "IGDB rejected the provided client ID"),
            IGDBError::BadSecret => return write!(f, "IGDB rejected the provided client secret"),
            IGDBError::BadToken => return write!(f, "IGDB rejected the provided bearer token"),
            IGDBError::BadAuth => return write!(f, "IGDB rejected some part of the provided authentication"),
            IGDBError::UnknownError(err_string) => return write!(f, "{}", &err_string),
            _ => return write!(f, "IGDB API had an unknown error")
        }
    }
}

#[derive(Debug)]
pub enum SteamError {
    UnknownError(String), // Unhandled error
    ServerError, // Steam had an internal error
    BadResponse, // Steam returned an unparseable response
    BadWebkey, // The supplied webkey is wrong
    GamesListPrivate(u64), // The games list of the given steam id is unretrievable, probably due to game list privacy settings
    GamesListEmpty(u64), // Intersection failed, the user with the given steam id had no games
    FriendListPrivate, // The steam id passed to get_friend_list() has their friend list set to private
}

impl fmt::Display for SteamError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            SteamError::ServerError => return write!(f, "Steam had an internal server error"),
            SteamError::BadResponse => return write!(f, "Steam returned an unparseable response"),
            SteamError::BadWebkey => return write!(f, "Steam rejected the provided webkey"),
            SteamError::UnknownError(err_string) => return write!(f, "{}", &err_string),
            SteamError::GamesListPrivate(steamid) => return write!(f, "The user with Steam ID {} has their games list set to private", steamid),
            SteamError::GamesListEmpty(steamid) => return write!(f, "The user with the Steam ID {} has no games to intersect", steamid),
            SteamError::FriendListPrivate => return write!(f, "The given user has their friend list set to private"),
            _ => return write!(f, "Steam had an unknown error")
        }
    }
}

#[derive(Debug)]
pub enum WCWPError {
    SteamError(SteamError), // Steam API Error
    IGDBError(IGDBError), // IGDB API Error
}

impl fmt::Display for WCWPError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            WCWPError::SteamError(e) => return e.fmt(f),
            WCWPError::IGDBError(e) => return e.fmt(f),
        }
    }
}

impl From<SteamError> for WCWPError {
    fn from(e: SteamError) -> Self {
        return WCWPError::SteamError(e);
    }
}

impl From<IGDBError> for WCWPError {
    fn from(e: IGDBError) -> Self {
        return WCWPError::IGDBError(e);
    }
}