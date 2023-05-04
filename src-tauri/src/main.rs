// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use anyhow::Result;
use dcspkg::config::{DcspkgConfig, DCSPKG_DIR};
use dcspkg::Package;
use dcspkg::{install_package, list_all_packages, run_package};
use dcspkg::util::list_installed_packages;

lazy_static::lazy_static! {
    static ref DCSPKG_CONFIG: Result<DcspkgConfig, String> = match dcspkg::config::DcspkgConfig::get() {
        Ok(c) => Ok(c),
        Err(e) => {
            println!("{:#?}", e);
            Err("Failed to get or create config file".to_string())
        },
    };
}

// Forwarding functions to connect the JS code to dcspkg.
#[tauri::command]
fn get_all_games() -> Result<Vec<Package>, String> {
    // https://www.reddit.com/r/rust/comments/pejdat/weird_syntax_using_lazy_static_am_i_doing/
    let c = (*DCSPKG_CONFIG).clone()?;
    match list_all_packages(c.server.url) {
        Ok(v) => Ok(v),
        Err(e) => {
            println!("{:#?}", e);
            Err("Failed to get games list".to_string())
        },
    }
}

#[tauri::command]
fn get_installed_games() -> Result<Vec<Package>, String> {
    let c = (*DCSPKG_CONFIG).clone()?;
    match list_installed_packages(&c.registry.registry_file) {
        Ok(v) => Ok(v),
        Err(e) => {
            println!("{:#?}", e);
            Err("Failed to get installed games list".to_string())
        },
    }
}

#[tauri::command(async)]
fn install_game(pkgname: &str) -> Result<(), String> {
    let c = (*DCSPKG_CONFIG).clone()?;
    let pkgname_str = String::from(pkgname);
    let pkgname_str2 = pkgname_str.clone();
    match std::thread::spawn(move || {
        install_package(
            &pkgname_str,
            c.server.url,
            c.registry.install_dir,
            c.registry.bin_dir,
            c.registry.registry_file,
        )
    })
    .join()
    {
        Ok(_) => Ok(()),
        Err(e) => {
            println!("{:#?}", e);
            Err(format!(
                "Failed to install {}",
                pkgname_str2
            ))
        },
    }
}

#[tauri::command]
fn run_game(pkgname: &str) -> Result<(), String> {
    let c = (*DCSPKG_CONFIG).clone()?;
    let pkgname_str = String::from(pkgname);
    let pkgname_str2 = pkgname_str.clone();
    match run_package(&c.registry.registry_file, c.registry.install_dir, &pkgname_str) {
        Ok(_) => Ok(()),
        Err(e) => {
            println!("{:#?}", e);
            Err(format!("Failed to run {}", pkgname_str2))
        },
    }
}

fn main() {
    // Copied from the dcspkg binary initialisation.
    // This could be moved into the dcspkg library at
    // some point.

    //create the dcspkg directory
    std::fs::create_dir_all(&*DCSPKG_DIR)?;

    //load config
    let config = DcspkgConfig::get()?;

    //create registry file if not exist
    if !config.registry.registry_file.is_file() {
        std::fs::write(&config.registry.registry_file, "[]")
            .context("Could not create empty package registry file")?;
    }

    // Init Tauri
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            get_all_games,
            get_installed_games,
            install_game,
            run_game
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
