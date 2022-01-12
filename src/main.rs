#[macro_use] extern crate rocket;

use rocket::{
    figment::{
        Figment,
        providers::{ Json, Format, Env },
    },
    fs::FileServer,
    Rocket, Build,
};
use rocket_dyn_templates::{
    Template,
};
use std::borrow::Cow;

pub mod structs;
use crate::structs::WCWPConfig;

pub mod routes;
use crate::routes::routes;

pub mod serde_utils;
pub mod api;

const DEFAULT_CONFIG_PATH: &'static str = "wcwp_config.json";

// This is the main function
#[launch]
async fn launch() -> Rocket<Build> {
    let mut figment = Figment::from(rocket::Config::default()) // Default Rocket config
        .merge(Env::prefixed("WCWP_").global()); // Load any variable starting with WCWP_ into config
    
    let mut config_path: Cow<str> = Cow::Borrowed(DEFAULT_CONFIG_PATH);
    // Handle alternate config path
    if let Ok(path) = figment.extract_inner("config_path")
    {
        config_path = path;
    }
    figment = figment
        .join(Json::file(&*config_path)); // Env variables still take priority over config
    
    // Handle reversed contact email
    match figment
        .extract_inner::<Cow<str>>("contact_email")
    {
        Ok(email) => {
            let (user, domain) = email.split_once('@')
                .expect(format!("CONFIG ERROR: 'contact_email' does not have a '@' sign, and is not a valid email. Please modify '{}' and re-run.", DEFAULT_CONFIG_PATH).as_str());
            let (user, domain): (String, String) = (
                user.chars().rev().collect(),
                domain.chars().rev().collect(),
            );

            figment = figment
                .merge(("contact_email_user_reversed", user))
                .merge(("contact_email_domain_reversed", domain));
        },
        Err(err) => {
            println!("Config error: {}", err);
            std::process::exit(exitcode::CONFIG);
        }
    }

    let wcwp_config: WCWPConfig = figment.extract().unwrap();
    
    rocket::custom(figment)
        .mount("/", routes())
        .mount("/static", FileServer::from("static"))
        .attach(Template::fairing())
        .manage(wcwp_config)
}