use serde::{
    Serialize, Deserialize,
    de::{ Visitor, Unexpected },
};

#[derive(Serialize, Default)]
pub struct EmailProtector {
    pub user_reversed: String,
    pub domain_reversed: String,
    pub email: String,
}

#[allow(dead_code)]
impl EmailProtector {
    pub fn new(email: &str) -> Self {
        let (user, domain) = email.split_once('@')
            .unwrap_or_else(|| panic!("Invalid email given to EmailProtector ({})", email));
        
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

struct EmailProtectorVisitor;

impl Visitor<'_> for EmailProtectorVisitor {
    type Value = EmailProtector;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("a string containing '@'")
    }

    fn visit_str<E>(self, email: &str) -> Result<Self::Value, E>
    where
            E: serde::de::Error, {
            let (user, domain) = email.split_once('@').ok_or_else( ||
                E::invalid_value(
                    Unexpected::Str(email),
                    &"string containing '@'"
                )
            )?;
            let (user, domain): (String, String) = (
                user.chars().rev().collect(),
                domain.chars().rev().collect(),
            );
            
            Ok(Self::Value {
                user_reversed: user,
                domain_reversed: domain,
                email: email.to_string(),
            })
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