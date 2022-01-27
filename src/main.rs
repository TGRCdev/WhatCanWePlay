#[macro_use] extern crate rocket;

use rocket::{
    figment::{
        Figment,
        providers::{ Json, Format, Env },
    },
    fairing::AdHoc,
    fs::FileServer,
    Rocket, Build,
};
use rocket_dyn_templates::{
    Template,
};
use std::borrow::Cow;

pub mod structs;
use crate::structs::{ WCWPConfig, CachedFileServer };

pub mod routes;
use crate::routes::routes;

pub mod api;
use api::steam::backend::SteamClient;

pub mod serde_utils;

const DEFAULT_CONFIG_PATH: &str = "wcwp_config.json";

// This is the main function
#[launch]
async fn launch() -> Rocket<Build> {
    let mut figment = Figment::from(rocket::Config::default()) // Default Rocket config
        .merge(Env::prefixed("WCWP_").global()); // Load any variable starting with WCWP_ into config
    
    let mut config_path = Cow::Borrowed(DEFAULT_CONFIG_PATH);
    // Handle alternate config path
    if let Ok(path) = figment.extract_inner("config_path")
    {
        config_path = path;
    }
    figment = figment
        .join(Json::file(&*config_path)); // Env variables still take priority over config

    rocket::custom(figment)
        .mount("/", routes())
        .mount("/static", CachedFileServer::from("static"))
        .attach(AdHoc::config::<WCWPConfig>())
        
        .mount("/api", api::routes())
        .register("/api", catchers![api::minimal_catcher])
        .attach(SteamClient::fairing())

        .attach(Template::fairing())
}