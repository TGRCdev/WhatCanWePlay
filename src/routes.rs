use rocket::{
    Route, State,
    get,
    http::{ CookieJar, Cookie },
    response::Redirect,
};
use rocket_dyn_templates::Template;
use rocket_dyn_templates::tera::Context;

use crate::WCWPConfig;

#[get("/")]
#[inline(always)]
async fn index(config: &State<WCWPConfig>, cookies: &CookieJar<'_>) -> Template {
    let mut context = Context::from_serialize(&**config).unwrap();
    if cookies.get("loggedIn").is_some()
    {
        context.insert("logged_in", &true);
    }
    Template::render("app", context.into_json())
}

#[get("/privacy")]
#[inline(always)]
async fn privacy(context: &State<WCWPConfig>) -> Template {
    Template::render("privacy", context.inner())
}

#[get("/logout")]
#[inline(always)]
async fn logout(cookies: &CookieJar<'_>) -> Redirect {
    cookies.remove(Cookie::named("loggedIn"));

    Redirect::to("/")
}

#[inline(always)]
pub fn routes() -> Vec<Route> {
    routes![index, privacy, logout]
}