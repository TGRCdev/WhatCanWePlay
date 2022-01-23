use rocket::{
    Route, Request,
    http::Status,
};

pub mod steam;
pub mod testing;

pub fn routes() -> Vec<Route>
{
    steam::routes()
}

// Minimize the amount of data sent from API errors
#[catch(default)]
pub fn minimal_catcher(status: Status, _: &Request) -> &'static str {
    status.reason_lossy()
}