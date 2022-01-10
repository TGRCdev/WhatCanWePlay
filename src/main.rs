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
use std::borrow::Cow;

use serde::{
    Serialize, Deserialize,
    de::{
        Visitor, Unexpected,
    },
};

const DEFAULT_CONFIG_PATH: &'static str = "wcwp_config.json";

#[derive(Serialize)]
struct EmailProtector {
    pub user_reversed: String,
    pub domain_reversed: String,
    pub email: String,
}

#[allow(dead_code)]
impl EmailProtector {
    pub fn new(email: &str) -> Self {
        let (user, domain) = email.split_once('@')
            .expect(format!("Invalid email given to EmailProtector ({})", email).as_str());
        
        Self {
            user_reversed: user.chars().rev().collect(),
            domain_reversed: domain.chars().rev().collect(),
            email: email.to_string(),
        }
    }

    pub fn get_email(&self) -> String {
        return self.user_reversed.chars().rev()
            .chain(['@'].into_iter())
            .chain(self.domain_reversed.chars().rev())
            .collect();
    }

    pub fn get_user(&self) -> String {
        return self.user_reversed.chars().rev().collect();
    }

    pub fn get_domain(&self) -> String {
        return self.domain_reversed.chars().rev().collect();
    }
}

impl Default for EmailProtector {
    fn default() -> Self {
        Self { user_reversed: Default::default(), domain_reversed: Default::default(), email: Default::default() }
    }
}

struct EmailProtectorVisitor;

impl Visitor<'_> for EmailProtectorVisitor {
    type Value = EmailProtector;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("a string containing '@'")
    }

    fn visit_str<E>(self, email: &str) -> Result<Self::Value, E>
    where
            E: serde::de::Error, {
            let (user, domain) = email.split_once('@').ok_or(
                E::invalid_value(
                    Unexpected::Str(email),
                    &"string containing '@'"
                )
            )?;
            let (user, domain): (String, String) = (
                user.chars().rev().collect(),
                domain.chars().rev().collect(),
            );
            return Ok(Self::Value {
                user_reversed: user,
                domain_reversed: domain,
                email: email.to_string(),
            });
    }
}

impl<'de> Deserialize<'de> for EmailProtector
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de> {
        deserializer.deserialize_string(EmailProtectorVisitor)
    }
}

#[derive(Serialize, Deserialize)]
#[serde(default)]
struct WCWPConfig {
    donate_url: String,
    source_url: String,
    contact_email: EmailProtector,
    privacy_email: EmailProtector,

    #[serde(skip_deserializing)]
    commit: &'static str,
}

impl Default for WCWPConfig {
    fn default() -> Self {
        Self {
            donate_url: Default::default(),
            source_url: Default::default(),
            contact_email: Default::default(),
            commit: env!("VERGEN_GIT_SHA_SHORT"),
            privacy_email: Default::default(),
        }
    }
}

#[get("/")]
async fn index(context: &State<WCWPConfig>) -> Template {
    Template::render("base_page", context.inner())
}

#[get("/privacy")]
async fn privacy(context: &State<WCWPConfig>) -> Template {
    Template::render("privacy", context.inner())
}

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
        .mount("/", routes![index, privacy])
        .mount("/static", FileServer::from("static"))
        .attach(Template::fairing())
        .manage(wcwp_config)
}