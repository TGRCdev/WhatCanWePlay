#[macro_use] extern crate rocket;

use rocket::{
    figment::{
        Figment,
        providers::{ Json, Format, Env },
    },
    fs::FileServer,
    Rocket, Build, 
    State,
};
use rocket_dyn_templates::{
    Template,
};

use serde::{ Serialize, Deserialize };

#[derive(Serialize, Deserialize)]
#[serde(default)]
struct WCWPConfig {
    donate_url: String,
    source_url: String,
    contact_email_user_reversed: String,
    contact_email_domain_reversed: String,
    commit: String,
}

impl Default for WCWPConfig {
    fn default() -> Self {
        Self {
            donate_url: Default::default(),
            source_url: Default::default(),
            contact_email_user_reversed: Default::default(),
            contact_email_domain_reversed: Default::default(),
            commit: env!("VERGEN_GIT_SHA_SHORT").to_string(),
        }
    }
}

#[get("/")]
async fn index(context: &State<WCWPConfig>) -> Template {
    Template::render("base_page", context.inner())
}

// This is the main function
#[launch]
async fn launch() -> Rocket<Build> {
    let mut figment = Figment::from(rocket::Config::default())
        .merge(Json::file("wcwp_config.json"))
        .merge(Env::prefixed("WCWP_").global());
    
    match figment
        .find_value("contact_email")
    {
        Ok(email) => {
            let email = email.into_string().unwrap();
            let (user, domain) = email.split_once('@').unwrap();
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
        .mount("/", routes![index])
        .mount("/static", FileServer::from("static"))
        .attach(Template::fairing())
        .manage(wcwp_config)
}