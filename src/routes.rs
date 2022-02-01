use rocket::{
    Route, State,
    get,
    http::{ CookieJar, Cookie, Status },
    response::{ Redirect, Responder },
};
use rocket_dyn_templates::Template;
use rocket_dyn_templates::tera::Context;

use crate::WCWPConfig;

#[get("/")]
async fn index(config: &State<WCWPConfig>, cookies: &CookieJar<'_>) -> Template {
    Template::render("index", &**config)
}

#[get("/privacy")]
async fn privacy(context: &State<WCWPConfig>) -> Template {
    Template::render("privacy", context.inner())
}

#[get("/logout")]
async fn logout(cookies: &CookieJar<'_>) -> Redirect {
    cookies.remove(Cookie::named("loggedIn"));

    Redirect::to("/")
}

#[get("/favicon.ico")]
async fn favicon<'r>() -> impl Responder<'r, 'static> {
    // TODO
    Status::NoContent
}

pub fn routes() -> Vec<Route> {
    routes![index, privacy, logout, favicon]
}