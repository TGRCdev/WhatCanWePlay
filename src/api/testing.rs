// TODO: Make a proc_macro_attribute somehow

use serde::Serialize;

#[derive(Serialize)]
#[serde(rename_all = "lowercase")]
pub enum APITestArgType {
    String,
    Int,
    Bool,
    #[serde(rename = "csl:int")]
    CSLOfInt,
    #[serde(rename = "csl:string")]
    CSLOfString,
}

#[derive(Serialize)]
#[serde(untagged)]
pub enum APITestArgDefault {
    String(&'static str),
    Int(i64),
    Bool(bool),
    CSLOfInt(&'static [i64]),
    CSLOfString(&'static [&'static str])
}

#[derive(Serialize)]
pub struct APITestArgument {
    pub name: &'static str,
    #[serde(rename = "type")]
    pub arg_type: APITestArgType,

    pub default: Option<APITestArgDefault>,
}

#[derive(Serialize)]
pub struct APITestInfo {
    #[serde(rename(serialize = "api_function_name"))]
    pub func_name: &'static str,

    #[serde(rename(serialize = "api_function_params"))]
    pub func_args: Vec<APITestArgument>,
}

/// Spawn an API test page route function 
macro_rules! api_test {
    ($fn_name:ident, $api_route:literal, $context:expr) => {
        #[get($api_route)]
        fn $fn_name() -> rocket_dyn_templates::Template {
            rocket_dyn_templates::Template::render("api_test", $context)
        }
    }
}

pub(crate) use api_test;