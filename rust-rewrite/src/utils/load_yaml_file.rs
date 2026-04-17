use std::fs;
use serde::de::DeserializeOwned;

pub fn load_file<T: DeserializeOwned>(file: &str) -> Result<T, Box<dyn std::error::Error>> {
    let contents = fs::read_to_string(file)?;

    let info: T = serde_yaml::from_str(&contents)?;

    Ok(info)
}
