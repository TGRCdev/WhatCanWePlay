use rocket::{
    Route, State,
    get,
    form::Form,
    response::Redirect,
};
use rocket_dyn_templates::Template;

use crate::WCWPConfig;

#[get("/")]
#[inline(always)]
async fn index(context: &State<WCWPConfig>) -> Template {
    Template::render("home", context.inner())
}

#[get("/privacy")]
#[inline(always)]
async fn privacy(context: &State<WCWPConfig>) -> Template {
    Template::render("privacy", context.inner())
}

#[inline(always)]
pub fn routes() -> Vec<Route> {
    routes![index, privacy]
}