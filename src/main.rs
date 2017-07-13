extern crate requests;
extern crate zip;
use std::io;
use std::io::Cursor;
use std::path;
use std::env;
use std::process;
use std::fs;
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

fn main() {
    // Determine the directory on the filesystem, where PoE filters should live.
    let local_dir = env::home_dir()
        .unwrap()
        .join("Documents")
        .join("My Games")
        .join("Path of Exile");
    if !local_dir.exists() {
        println!(
            "Unable to find the PoE configration directory, tried: \"{}\"",
            local_dir.to_str().unwrap()
        );
        process::exit(1);
    }
    let local_dir = local_dir.into_os_string();
    println!(
        "PoE configuration directory is: \"{}\"",
        local_dir.to_str().unwrap()
    );

    // Fetch and parse info about the latest release.
    println!("Fetching info about the latest release...");
    let latest_release = determine_latest_release();
    println!("Latest tagname: {}", latest_release.tag_name);
    println!("Published at:   {}", latest_release.published_at);

    // Fetch and parse the zipfile.
    println!("Fetching zip-file... ");
    let zipfile = fetch_url_to_buffer(&latest_release.zip_url);
    println!("Fetched {} bytes, extracting filters...", zipfile.len());

    // Initialize reading the zipfile.
    let reader = Cursor::new(zipfile);
    let mut zipfile = zip::ZipArchive::new(reader).unwrap();

    // This logic'll probably improve as the library improves :)
    let empty_path = path::Path::new("");
    for i in 0..zipfile.len() {
        let mut file = zipfile.by_index(i).unwrap();

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
        let local_filename = path::PathBuf::from(local_dir.to_os_string()).join(filename);
        let mut local_file = fs::File::create(local_filename).unwrap();

        // Copy from the zipfile to the local filesystem, notifying the user.
        let bytes = io::copy(&mut file, &mut local_file).unwrap();
        println!("  Wrote {} ({} bytes)", filename.to_str().unwrap(), bytes);
    }

    println!("All done :)");
}
