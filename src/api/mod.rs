use rocket::{
    Route, Request,
    http::Status,
};

pub mod steam;

pub fn routes() -> Vec<Route>
{
    return routes![
        steam::get_steam_users_info
    ]
}

// Minimize the amount of data sent from API errors
#[catch(default)]
pub fn minimal_catcher(status: Status, _: &Request) -> &'static str {
    status.reason_lossy()
}