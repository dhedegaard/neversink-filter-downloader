#[macro_use]
extern crate serde_derive;
extern crate dirs;

extern crate chrono;
extern crate reqwest;
extern crate serde;
extern crate serde_json;
extern crate term_painter;
extern crate zip;

use std::env;
use std::error::Error;
use std::fs;
use std::io;
use std::io::BufRead;
use std::io::Cursor;
use std::io::Read;
use std::path;
use std::process;

use term_painter::Color::BrightWhite;
use term_painter::ToStyle;

/// API URL for the latest release.
const LATEST_RELEASE_URL: &'static str =
    "https://api.github.com/repos/NeverSinkDev/NeverSink-Filter/releases/latest";

#[derive(Debug, Deserialize)]
struct ReleaseInfo {
    tag_name: String,
    published_at: String,
    zipball_url: String,
}

/// Determines and returns info about the latest release available.
fn determine_latest_release() -> Result<ReleaseInfo, Box<dyn Error>> {
    // Fetch the URL and parse the JSON.
    reqwest::get(LATEST_RELEASE_URL)?
        .json()
        .map_err(|e| e.into())
}

/// Fetches the given URL, returning the body as a string.
fn fetch_url_to_buffer(url: &str) -> Result<Vec<u8>, Box<dyn Error>> {
    let mut result = vec![];
    reqwest::get(url)?.read_to_end(&mut result)?;
    Ok(result)
}

fn determine_documents_dir() -> std::path::PathBuf {
    match dirs::document_dir() {
        Some(documents) => documents.clone(),
        None => match dirs::home_dir() {
            Some(homedir) => homedir.join("Documents"),
            None => panic!("Unable to find homedir for user."),
        },
    }
}

/// Determines and returns a path object pointing to the PoE configuration
/// directory.
fn determine_poe_dir() -> Result<String, Box<dyn Error>> {
    let documents = determine_documents_dir();

    let poedir = documents.join("My Games").join("Path of Exile");
    if !poedir.exists() {
        fs::create_dir_all(&poedir)?;
        println!(
            "The expected PoE directory did not exist, so it was created: {}",
            &poedir.to_str().unwrap()
        );
    }

    Ok(poedir.to_str().unwrap().to_owned())
}

/// Reads and returns the version value from the filename specified.
fn read_filter_version_from_string(filename: path::PathBuf) -> Result<String, Box<dyn Error>> {
    let mut f = fs::File::open(filename)?;
    let mut content = String::new();
    f.read_to_string(&mut content)?;
    if let Some(version_line) = content
        .split("\n")
        .filter(|line| line.contains("# VERSION:"))
        .next()
    {
        if let Some(version_str) = version_line.split_whitespace().last() {
            return Ok(version_str.to_owned());
        }
    }
    Err(Box::new(io::Error::new(
        io::ErrorKind::InvalidData,
        "Unable to fetch the version line in the filter",
    )))
}

/// Fetches and returns an existing filter version (if there are any existing
/// filter files).
fn fetch_existing_filter_version() -> Result<String, Box<dyn Error>> {
    for path in fs::read_dir(determine_poe_dir()?)? {
        let path = path?;
        if let Some(filename) = path.file_name().to_str() {
            if filename.contains("NeverSink") && filename.contains(".filter") {
                return read_filter_version_from_string(path.path());
            }
        }
    }
    Err(Box::new(io::Error::new(
        io::ErrorKind::Other,
        "No existing filters found",
    )))
}

fn remove_existing_filters(local_dir: &str) -> io::Result<()> {
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

fn fetch_and_extract_new_version(
    local_dir: &str,
    latest_release: ReleaseInfo,
) -> Result<(), Box<dyn Error>> {
    // Fetch and parse the zipfile.
    println!("{}", BrightWhite.bold().paint("Fetching zip-file... "));
    let zipfile = fetch_url_to_buffer(&latest_release.zipball_url)?;
    println!(
        "Fetched {} bytes, extracting filters...",
        BrightWhite.bold().paint(zipfile.len().to_string())
    );

    // Initialize reading the zipfile.
    let reader = Cursor::new(zipfile);
    let mut zipfile = zip::ZipArchive::new(reader)?;

    // This logic'll probably improve as the library improves :)
    let empty_path = path::Path::new("");
    for i in 0..zipfile.len() {
        let mut file = zipfile.by_index(i)?;

        // Skip files that are not .filter, or not in the root of the zipfile.
        let filename = file.name().to_owned();
        let path = path::Path::new(&filename);
        if path.extension().is_none()
            || path.extension().unwrap() != "filter"
            || path.parent().unwrap().parent().unwrap() != empty_path
        {
            continue;
        }

        // Determine the filename on the local filesystem and create the file.
        let filename = path.file_name();
        if filename.is_none() {
            continue;
        }
        let filename = filename.unwrap();
        let local_filename = path::PathBuf::from(local_dir).join(filename);
        let mut local_file = fs::File::create(local_filename)?;

        // Copy from the zipfile to the local filesystem, notifying the user.
        let bytes = io::copy(&mut file, &mut local_file)?;
        if let Some(filename) = filename.to_str() {
            println!("  Wrote {} ({} bytes)", filename, bytes);
        }
    }
    Ok(())
}

fn update_filter(force: bool) -> Result<(), Box<dyn Error>> {
    // Determine the directory on the filesystem, where PoE filters should live.
    let local_dir = determine_poe_dir()?;
    println!(
        "PoE configuration directory is: \"{}\"",
        BrightWhite.bold().paint(local_dir.to_string())
    );

    // Look for existing neversink filter files.
    let current_version = match fetch_existing_filter_version() {
        Ok(version) => version,
        Err(err) => format!("<{}>", err),
    };

    // Fetch and parse info about the latest release.
    println!("Fetching info about the latest release from Github...");
    let latest_release = determine_latest_release()?;
    // Extract the published date as RFC3339, convert it to the local timezone
    // and convert it to a pretty printable.
    let published_at = chrono::DateTime::parse_from_rfc3339(&latest_release.published_at)?;
    let published_at = published_at.with_timezone(&chrono::Local);
    let published_at = published_at.format("%Y-%m-%d %H:%M:%S");
    println!(
        "Current tagname:   {}",
        BrightWhite.bold().paint(&current_version)
    );
    println!(
        "Latest tagname:    {}",
        BrightWhite.bold().paint(&latest_release.tag_name)
    );
    println!(
        "Published at:      {}",
        BrightWhite.bold().paint(&published_at)
    );
    println!();

    // If the tag names are equal, then return.
    if current_version == latest_release.tag_name && !force {
        println!("Latest version is already installed, doing nothing...");
        return Ok(());
    }

    println!("Removing existing filters...");
    remove_existing_filters(&local_dir)?;

    println!("Fetching and extracting new filters.");
    fetch_and_extract_new_version(&local_dir, latest_release)?;

    println!("{}", BrightWhite.bold().paint("All done"));
    Ok(())
}

fn main() {
    let force = std::env::args().any(|e| e == "-f");

    if let Err(err) = update_filter(force) {
        println!("Error updating filter: {}", err);
        process::exit(1);
    }

    if cfg!(windows) {
        let args = env::args().skip(1).collect::<Vec<_>>();
        let has_quite_flag = args.iter().any(|e| e == "-q" || e == "--quite");
        if !has_quite_flag {
            // Let the user read the output before closing, for cmd on windows :)
            println!("{}", BrightWhite.bold().paint("Press enter to close :)"));
            let stdin = io::stdin();
            let mut line = String::new();
            stdin.lock().read_line(&mut line).unwrap_or_default();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_determine_documents_dir_should_return_something() {
        assert!(determine_documents_dir().to_string_lossy().len() > 0);
    }
}
