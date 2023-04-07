extern crate serde;

use serde_derive::Deserialize;
use std::error::Error;
use std::fs;
use std::io;
use std::io::Read;
use std::path;

#[derive(Debug, Clone, Deserialize)]
pub struct ReleaseInfo {
    pub tag_name: String,
    pub published_at: String,
    pub zipball_url: String,
}

/// Useragent for the github API, as it crashes when no header exists.
const USER_AGENT: &str = "neversink-filter-downloader";

/// API URL for the latest release.
const LATEST_RELEASE_URL: &str =
    "https://api.github.com/repos/NeverSinkDev/NeverSink-Filter/releases/latest";

/// Determines and returns info about the latest release available.
pub async fn determine_latest_release() -> Result<ReleaseInfo, Box<dyn Error>> {
    // Fetch the URL and parse the JSON.
    Ok(reqwest::Client::new()
        .get(LATEST_RELEASE_URL)
        .header(reqwest::header::USER_AGENT, USER_AGENT)
        .send()
        .await?
        .error_for_status()?
        .json::<ReleaseInfo>()
        .await?)
}

/// Fetches the given URL, returning the body as a string.
pub async fn fetch_url_to_buffer(url: &str) -> Result<Vec<u8>, Box<dyn Error>> {
    Ok(reqwest::Client::new()
        .get(url)
        .header(reqwest::header::USER_AGENT, USER_AGENT)
        .send()
        .await?
        .bytes()
        .await?
        .to_vec())
}

pub fn determine_documents_dir() -> std::path::PathBuf {
    match dirs::document_dir() {
        Some(documents) => documents,
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

/// Reads and returns the version value from the filename specified.
pub fn read_filter_version_from_string(filename: path::PathBuf) -> Result<String, Box<dyn Error>> {
    let mut f = fs::File::open(filename)?;
    let mut content = String::new();
    f.read_to_string(&mut content)?;
    if let Some(version_line) = content.split('\n').find(|line| line.contains("# VERSION:")) {
        if let Some(version_str) = version_line.split_whitespace().last() {
            return Ok(version_str.to_owned());
        }
    }
    Err(Box::new(io::Error::new(
        io::ErrorKind::InvalidData,
        "Unable to fetch the version line in the filter",
    )))
}

pub fn remove_existing_filters(local_dir: &str) -> io::Result<()> {
    for path in fs::read_dir(local_dir)? {
        let path = path?;
        if let Some(filename) = path.file_name().to_str() {
            if filename.contains("NeverSink") && filename.contains(".filter") {
                fs::remove_file(path.path())?;
            }
        }
    }
    Ok(())
}

#[derive(Debug, Clone)]
pub struct FetchExistingFilterVersionResult {
    pub poedir: String,
    pub created_directory: bool,
}
/// Fetches and returns an existing filter version (if there are any existing
/// filter files).
pub fn fetch_existing_filter_version() -> Result<FetchExistingFilterVersionResult, Box<dyn Error>> {
    let poedir = determine_poe_dir()?;
    for path in fs::read_dir(poedir.poedir)? {
        let path = path?;
        if let Some(filename) = path.file_name().to_str() {
            if filename.contains("NeverSink") && filename.contains(".filter") {
                return match read_filter_version_from_string(path.path()) {
                    Ok(result) => Ok(FetchExistingFilterVersionResult {
                        poedir: result,
                        created_directory: poedir.directory_created,
                    }),
                    Err(e) => Err(e),
                };
            }
        }
    }
    Err(Box::new(io::Error::new(
        io::ErrorKind::Other,
        "No existing filters found",
    )))
}
