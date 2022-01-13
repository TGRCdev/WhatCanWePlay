use serde::{
    Deserializer, Deserialize,
    de::{ Error, Unexpected },
};
use serde_json::Value;

pub fn u64_or_parse_str<'de, D>(deserializer: D) -> Result<u64, D::Error>
where
    D: Deserializer<'de>
{
    let val: Value = Deserialize::deserialize(deserializer)?;
    match val {
        Value::Null => Err(Error::invalid_type(
            Unexpected::Other("null"),
            &"u64"
        )),
        Value::Bool(val) => Err(Error::invalid_type(
            Unexpected::Bool(val),
            &"u64",
        )),
        Value::Number(val) => {
            if let Some(val) = val.as_f64()
            {
                Err(Error::invalid_type(
                    Unexpected::Float(val),
                    &"u64"
                ))
            }
            else if let Some(val) = val.as_u64()
            {
                Ok(val)
            }
            else
            {
                let val = val.as_i64().unwrap();
                Err(Error::invalid_type(
                    Unexpected::Signed(val),
                    &"u64"
                ))
            }
        },
        Value::String(val) => {
            if let Ok(val) = val.parse()
            {
                Ok(val)
            }
            else {
                Err(Error::invalid_type(
                    Unexpected::Str(&val),
                    &"u64"
                ))
            }
        },
        Value::Array(_val) => Err(Error::invalid_type(
            Unexpected::Seq,
            &"u64",
        )),
        Value::Object(_val) => Err(Error::invalid_type(
            Unexpected::Map,
            &"u64",
        )),
    }
}