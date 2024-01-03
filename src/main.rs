use std::io::Write;
use std::process::{self, exit, Command};
use std::{fs, io};

use git2::build::{CheckoutBuilder, RepoBuilder};
use git2::{FetchOptions, Progress, RemoteCallbacks};
use std::cell::RefCell;
use std::path::{Path, PathBuf};

use reqwest;
use serde::Deserialize;

static REPO_PATH: &str = "./Not_So_Serious_Company";
static REPO_URL: &str = "https://github.com/Vylkatis/Not_So_Serious_Company";
static CONFIG_PATH: &str = "./BepInEx/config";
static PLUGIN_PATH: &str = "./BepInEx/plugins";
static LAST_COMMIT_LOG: &str = "./last_not_so_serious_company_commit.log";
static LAST_COMMIT_API_URL: &str =
    "https://api.github.com/repos/Vylkatis/Not_So_Serious_Company/commits?per_page=1";

struct State {
    progress: Option<Progress<'static>>,
    total: usize,
    current: usize,
    path: Option<PathBuf>,
    newline: bool,
}

#[derive(Deserialize, Debug)]
struct GithubCommit {
    sha: String,
}

fn print(state: &mut State) {
    let stats = state.progress.as_ref().unwrap();
    let network_pct = (100 * stats.received_objects()) / stats.total_objects();
    if stats.received_objects() == stats.total_objects() {
        if !state.newline {
            println!();
            state.newline = true;
        }
        print!(
            "Resolving deltas {}/{}\r",
            stats.indexed_deltas(),
            stats.total_deltas()
        );
    } else {
        let prog = (network_pct + 1) / 5;
        print!(
            "progress: [{}] {}%\r",
            [" "; 20]
                .iter()
                .enumerate()
                .map(|(i, _)| if i < prog { "#" } else { " " })
                .collect::<Vec<&str>>()
                .join(""),
            network_pct + 1
        )
    }
    io::stdout().flush().unwrap();
}

fn clone() -> Result<(), git2::Error> {
    let state = RefCell::new(State {
        progress: None,
        total: 0,
        current: 0,
        path: None,
        newline: false,
    });
    let mut cb = RemoteCallbacks::new();
    cb.transfer_progress(|stats| {
        let mut state = state.borrow_mut();
        state.progress = Some(stats.to_owned());
        print(&mut *state);
        true
    });

    let mut co = CheckoutBuilder::new();
    co.progress(|path, cur, total| {
        let mut state = state.borrow_mut();
        state.path = path.map(|p| p.to_path_buf());
        state.current = cur;
        state.total = total;
        print(&mut *state);
    });

    let mut fo = FetchOptions::new();
    fo.remote_callbacks(cb);
    RepoBuilder::new()
        .fetch_options(fo)
        .with_checkout(co)
        .clone(REPO_URL, Path::new(REPO_PATH))?;
    println!();

    Ok(())
}

fn copy_dir_all(src: impl AsRef<Path>, dst: impl AsRef<Path>) -> io::Result<()> {
    fs::create_dir_all(&dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        if ty.is_dir() {
            copy_dir_all(entry.path(), dst.as_ref().join(entry.file_name()))?;
        } else {
            fs::copy(entry.path(), dst.as_ref().join(entry.file_name()))?;
        }
    }
    Ok(())
}

#[tokio::main]
async fn main() -> io::Result<()> {
    fs::remove_dir_all(REPO_PATH).unwrap_or_default();
    let last_commit_log = String::from(
        fs::read_to_string(LAST_COMMIT_LOG)
            .unwrap_or(String::new())
            .trim(),
    );

    let client = reqwest::Client::new();

    let response = client
        .get(LAST_COMMIT_API_URL)
        .header("User-Agent", "rust")
        .send()
        .await
        .unwrap();

    let commits: Vec<GithubCommit> = response.json().await.unwrap();

    if commits.first().unwrap().sha == last_commit_log {
        return Ok(());
    }

    match clone() {
        Ok(()) => (),
        _ => panic!("Git clone failed"),
    };

    let config_files = fs::read_dir(format!("{}/{}", REPO_PATH, CONFIG_PATH).as_str())?
        .map(|res| res.map(|file| file.path()))
        .collect::<Result<Vec<_>, io::Error>>()?;
    let plugin_files = fs::read_dir(format!("{}/{}", REPO_PATH, PLUGIN_PATH).as_str())?
        .map(|res| res.map(|file| file.path()))
        .collect::<Result<Vec<_>, io::Error>>()?;

    fs::create_dir_all(CONFIG_PATH).unwrap_or_default();

    for config_file in config_files {
        println!(
            "Copying {:?} to {:?}",
            config_file.as_os_str(),
            format!(
                "{}/{}",
                CONFIG_PATH,
                config_file.as_path().file_name().unwrap().to_str().unwrap()
            )
        );
        fs::copy(
            config_file.as_os_str(),
            format!(
                "{}/{}",
                CONFIG_PATH,
                config_file.as_path().file_name().unwrap().to_str().unwrap()
            ),
        )?;
    }

    fs::remove_dir_all(PLUGIN_PATH).unwrap_or_default();
    fs::create_dir_all(PLUGIN_PATH)?;

    for plugin_file in plugin_files {
        match plugin_file.as_path().is_dir() {
            true => {
                copy_dir_all(
                    plugin_file.as_os_str(),
                    format!(
                        "{}/{}",
                        PLUGIN_PATH,
                        plugin_file.as_path().file_name().unwrap().to_str().unwrap()
                    ),
                )?;
            }
            false => {
                println!(
                    "Copying {:?} to {:?}",
                    plugin_file.as_os_str(),
                    format!(
                        "{}/{}",
                        PLUGIN_PATH,
                        plugin_file.as_path().file_name().unwrap().to_str().unwrap()
                    )
                );
                fs::copy(
                    plugin_file.as_os_str(),
                    format!(
                        "{}/{}",
                        PLUGIN_PATH,
                        plugin_file.as_path().file_name().unwrap().to_str().unwrap()
                    ),
                )?;
            }
        };
    }

    fs::remove_dir_all(REPO_PATH)?;

    fs::write(LAST_COMMIT_LOG, &commits.first().unwrap().sha)?;

    if cfg!(target_os = "windows") {
        // Start the game
        let _ = Command::new("Lethal Company.exe")
            .stdin(process::Stdio::null())
            .stdout(process::Stdio::null())
            .stderr(process::Stdio::null())
            .spawn();
    };

    exit(0)
}
