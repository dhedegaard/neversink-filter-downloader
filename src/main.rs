extern crate requests;
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
use requests::ToJson;

/// API URL for the latest release.
static LATEST_RELEASE_URL: &str = "https://api.github.com/repos/NeverSinkDev/NeverSink-Filter/releases/latest";

struct ReleaseInfo {
    tag_name: String,
    published_at: String,
    zip_url: String,
}
/// Determines and returns info about the latest release available.
fn determine_latest_release() -> ReleaseInfo {
    // Fetch the URL and parse the JSON.
    let data = requests::get(LATEST_RELEASE_URL).unwrap().json().unwrap();
    ReleaseInfo {
        tag_name: data["tag_name"].as_str().unwrap().to_string(),
        published_at: data["published_at"].as_str().unwrap().to_string(),
        zip_url: data["zipball_url"].as_str().unwrap().to_string(),
    }
}

/// Fetches the given URL, returning the body as a string.
fn fetch_url_to_buffer(url: &str) -> Vec<u8> {
    requests::get(url).unwrap().content().to_owned()
}

/// Determines and returns a path object pointing to the PoE configuration
/// directory.
fn determine_poe_dir() -> Result<path::PathBuf, Box<Error>> {
    let homedir = env::home_dir();
    if homedir.is_none() {
        return Err(Box::new(io::Error::new(io::ErrorKind::NotFound, "Unable to find the homedir for the user.")));
    }
    let homedir = homedir.unwrap();

    let poedir = homedir.join("Documents")
        .join("My Games")
        .join("Path of Exile");

    if !poedir.exists() {
        return Err(Box::new(io::Error::new(io::ErrorKind::NotFound, format!("The expected PoE directory does not exist: {}", poedir.to_str().unwrap()))));
    }

    Ok(poedir)
}

/// Reads and returns the version value from the filename specified.
fn read_filter_version_from_string(filename: path::PathBuf) -> Result<String, Box<Error>> {
    let mut f = fs::File::open(filename)?;
    let mut content = String::new();
    f.read_to_string(&mut content)?;
    if let Some(version_line) = content.split("\n")
        .filter(|line| line.contains("# VERSION:")).next() {

        return Ok(version_line
            .split_whitespace()
            .last().unwrap()
            .to_owned());

    }
    Err(Box::new(io::Error::new(io::ErrorKind::InvalidData, "Unable to fetch the version line in the filter")))
}

/// Fetches and returns an existing filter version (if there are any existing
/// filter files).
fn fetch_existing_filter_version() -> Result<String, Box<Error>> {
    for path in fs::read_dir(determine_poe_dir()?)? {
        let path = path?;
        if let Some(filename) = path.file_name().to_str() {
            if filename.contains("NeverSink") && filename.contains(".filter") {
                return read_filter_version_from_string(path.path());
            }
        }
    }
    Err(Box::new(io::Error::new(io::ErrorKind::Other, "No existing filters found")))
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

fn fetch_and_extract_new_version(local_dir: &str, latest_release: ReleaseInfo) -> Result<(), Box<Error>> {
    // Fetch and parse the zipfile.
    println!("Fetching zip-file... ");
    let zipfile = fetch_url_to_buffer(&latest_release.zip_url);
    println!("Fetched {} bytes, extracting filters...", zipfile.len());

    // Initialize reading the zipfile.
    let reader = Cursor::new(zipfile);
    let mut zipfile = zip::ZipArchive::new(reader)?;

    // This logic'll probably improve as the library improves :)
    let empty_path = path::Path::new("");
    for i in 0..zipfile.len() {
        let mut file = zipfile.by_index(i)?;

        // Skip files that are not .filter, or not in the root of the zipfile.
        let filename = file.name().to_string();
        let path = path::Path::new(&filename);
        if path.extension().is_none() || path.extension().unwrap() != "filter" ||
            path.parent().unwrap().parent().unwrap() != empty_path
        {
            continue;
        }

        // Determine the filename on the local filesystem and create the file.
        let filename = path.file_name().unwrap();
        let local_filename = path::PathBuf::from(local_dir).join(filename);
        let mut local_file = fs::File::create(local_filename)?;

        // Copy from the zipfile to the local filesystem, notifying the user.
        let bytes = io::copy(&mut file, &mut local_file).unwrap();
        if let Some(filename) = filename.to_str() {
            println!("  Wrote {} ({} bytes)", filename, bytes);
        }
    }
    Ok(())
}

fn update_filter() -> Result<(), Box<Error>> {
    // Determine the directory on the filesystem, where PoE filters should live.
    let local_dir = determine_poe_dir()?.into_os_string();
    println!(
        "PoE configuration directory is: \"{}\"",
        local_dir.to_str().unwrap()
    );

    // Look for existing neversink filter files.
    let current_version = match fetch_existing_filter_version() {
        Ok(version) => version,
        Err(err) => format!("<Err: {}>", err),
    };

    // Fetch and parse info about the latest release.
    println!("Fetching info about the latest release from Github...");
    let latest_release = determine_latest_release();
    println!("Current tagname:   {}", current_version);
    println!("Latest tagname:    {}", latest_release.tag_name);
    println!("Published at:      {}", latest_release.published_at);
    println!("");

    if current_version == latest_release.tag_name {
        println!("Latest version is already installed, doing nothing...");
    } else {
        println!("Removing existing filters...");
        if let Err(err) = remove_existing_filters(local_dir.to_str().unwrap()) {
            println!("Error: unable to remove existing filter files: {}", err);
        }
        println!("Fetching and extracting new filters.");
        fetch_and_extract_new_version(local_dir.to_str().unwrap(), latest_release)?;
    }

    println!("All done, press enter to close :)");
    Ok(())
}

fn main() {
    if let Err(err) = update_filter() {
        println!("Error updating filter: {}", err);
        process::exit(1);
    }

    if cfg!(windows) {
        // Let the user read the output before closing, for cmd on windows :)
        let stdin = io::stdin();
        let mut line = String::new();
        stdin.lock().read_line(&mut line).unwrap_or_default();
    }
}
