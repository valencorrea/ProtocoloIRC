//! Modulo que se centra en las funcionalidades referentes a funciones varias
use std::{
    collections::HashMap,
    fmt::Display,
    str::FromStr,
    sync::{Arc, Mutex},
};

use crate::irc::message::utils::validate_name_invalid_none;

pub fn deserialize_bool(b: &str) -> Result<bool, String> {
    match b {
        "0" => Ok(false),
        "1" => Ok(true),
        _ => Err(deserialize_err("Invalid bool value")),
    }
}

pub fn deserialize_num<T>(n: &str) -> Result<T, String>
where
    T: FromStr,
{
    match n.parse::<T>() {
        Ok(v) => Ok(v),
        Err(_) => Err(deserialize_err("Invalid number")),
    }
}

fn deserialize_list<T>(l: &str, f: &dyn Fn(&str) -> Result<T, String>) -> Result<Vec<T>, String> {
    let list = l.split(';').collect::<Vec<&str>>();

    let mut r = Vec::new();
    for entry in list {
        match f(entry) {
            Ok(e) => r.push(e),
            Err(e) => return Err(e),
        }
    }
    Ok(r)
}

pub fn deserialize_err(s: &str) -> String {
    format!("[SERVER - DESERIALIZE] Error: {}", s)
}

pub fn mt<T>(c: T) -> Arc<Mutex<T>> {
    Arc::new(Mutex::new(c))
}

pub fn deseriaze_usernames(l: &str) -> Result<HashMap<String, bool>, String> {
    let nicks = deserialize_list(l, &|e| {
        if validate_name_invalid_none(Some(e.as_bytes())).is_err() {
            return Err(deserialize_err("Invalid nicks"));
        }
        Ok(e.to_owned())
    })?;
    let mut r = HashMap::new();
    for nick in nicks {
        r.insert(nick, false);
    }
    Ok(r)
}

pub fn serialize_option<T>(o: &Option<T>) -> String
where
    T: Display,
{
    if let Some(o) = o {
        o.to_string()
    } else {
        "".to_owned()
    }
}

pub fn serialize_bool(b: bool) -> String {
    if b {
        "1".to_owned()
    } else {
        "0".to_owned()
    }
}

pub fn serialize_list<T>(l: &[T]) -> String
where
    T: Display,
{
    l.iter()
        .map(|e| e.to_string())
        .collect::<Vec<String>>()
        .join(";")
}
