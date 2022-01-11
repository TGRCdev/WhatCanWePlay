use rocket::{
    Route, State,
    get,
};
use rocket_dyn_templates::Template;

use crate::WCWPConfig;

#[get("/")]
#[inline(always)]
async fn index(context: &State<WCWPConfig>) -> Template {
    Template::render("base_page", context.inner())
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