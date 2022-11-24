// get json from url and return serde_json::Value
use serde_json;
pub fn get_json(url: String) -> serde_json::Value {
    reqwest::blocking::get(url).unwrap().json::<serde_json::Value>().unwrap()
}