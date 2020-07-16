extern crate serde;

use std::error::Error;
use std::fs;
use std::io::Read;
use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct ReleaseInfo {
    pub tag_name: String,
    pub published_at: String,
    pub zipball_url: String,
}

/// API URL for the latest release.
const LATEST_RELEASE_URL: &'static str =
    "https://api.github.com/repos/NeverSinkDev/NeverSink-Filter/releases/latest";


/// Determines and returns info about the latest release available.
pub fn determine_latest_release() -> Result<ReleaseInfo, Box<dyn Error>> {
    // Fetch the URL and parse the JSON.
    reqwest::get(LATEST_RELEASE_URL)?
        .json()
        .map_err(|e| e.into())
}

/// Fetches the given URL, returning the body as a string.
pub fn fetch_url_to_buffer(url: &str) -> Result<Vec<u8>, Box<dyn Error>> {
    let mut result = vec![];
    reqwest::get(url)?.read_to_end(&mut result)?;
    Ok(result)
}

pub fn determine_documents_dir() -> std::path::PathBuf {
    match dirs::document_dir() {
        Some(documents) => documents.clone(),
        None => match dirs::home_dir() {
            Some(homedir) => homedir.join("Documents"),
            None => panic!("Unable to find homedir for user."),
        },
    }
}

pub struct DeterminePoeDirResult {
    pub poedir: String,
    pub directory_created: bool,
}
/// Determines and returns a path object pointing to the PoE configuration
/// directory.
pub fn determine_poe_dir() -> Result<DeterminePoeDirResult, Box<dyn Error>> {
    let documents = determine_documents_dir();

    let poedir = documents.join("My Games").join("Path of Exile");
    let mut created = false;
    if !poedir.exists() {
        fs::create_dir_all(&poedir)?;
        created = true;
    }

    Ok(DeterminePoeDirResult {
        poedir: poedir.to_str().unwrap().to_owned(),
        directory_created: created,
    })
}
