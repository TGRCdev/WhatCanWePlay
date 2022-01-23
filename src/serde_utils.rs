use serde::{
    Deserializer, Deserialize,
    Serializer,
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
            if let Some(val) = val.as_u64()
            {
                Ok(val)
            }
            else if let Some(val) = val.as_i64()
            {
                Err(Error::invalid_type(
                    Unexpected::Signed(val),
                    &"u64"
                ))
            }
            else
            {
                let val = val.as_f64().unwrap();
                Err(Error::invalid_type(
                    Unexpected::Float(val),
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

pub fn value_to_string<T, S>(value: &T, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
    T: ToString,
{
    let valstr = value.to_string();
    serializer.serialize_str(valstr.as_str())
}