// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use anyhow::Result;
use dcspkg::config::DcspkgConfig;
use dcspkg::Package;
use dcspkg::{install_package, list_all_packages, run_package};
use dcspkg::util::list_installed_packages;

// TODO
// lazy_static::lazy_static! {
//     static ref DCSPKG_CONFIG: dcspkg::config::DcspkgConfig = match dcspkg::config::DcspkgConfig::get() {
//         Ok(a) => a,
//         Err(_) => dcspkg::config::DcspkgConfig::default(),
//     };
// }

// This should be replaced with the lazy_static call above but I can't
// figure out how to get the types to work.
fn get_config() -> Result<DcspkgConfig, String> {
    match DcspkgConfig::get() {
        Ok(c) => Ok(c),
        Err(_) => Err("Failed to get or create config file".to_string()),
    }
}

// Forwarding functions to connect the JS code to dcspkg.
#[tauri::command]
fn get_all_games() -> Result<Vec<Package>, String> {
    let c = get_config()?;
    match list_all_packages(c.server.url) {
        Ok(v) => Ok(v),
        Err(e) => Err(format!("Failed to get games list\n\n{}", e.to_string())),
    }
}

#[tauri::command]
fn get_installed_games() -> Result<Vec<Package>, String> {
    let c = get_config()?;
    match list_installed_packages(&c.registry.registry_file) {
        Ok(v) => Ok(v),
        Err(e) => Err(format!(
            "Failed to get installed games list\n\n{}",
            e.to_string()
        )),
    }
}

#[tauri::command(async)]
fn install_game(pkgname: String) -> Result<(), String> {
    let c = get_config()?;
    let pkgname2 = pkgname.clone();
    match std::thread::spawn(move || {
        install_package(
            &pkgname,
            c.server.url,
            c.registry.install_dir,
            c.registry.bin_dir,
            c.registry.registry_file,
        )
    })
    .join()
    {
        Ok(_) => Ok(()),
        Err(_e) => Err(
            format!(
                "Failed to install {}",
                pkgname2,
                // e.as_ref().
            )
        ),
    }
}

#[tauri::command]
fn run_game(pkgname: String) -> Result<(), String> {
    let c = get_config()?;
    let pkgname2 = pkgname.clone();
    match run_package(&c.registry.registry_file, c.registry.install_dir, &pkgname) {
        Ok(_) => Ok(()),
        Err(e) => Err(format!("Failed to run {}\n\n{}", pkgname2, e.to_string())),
    }
}

fn main() {
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
