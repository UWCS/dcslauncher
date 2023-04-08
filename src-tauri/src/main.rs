// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use anyhow::Context;
use dcspkg::config::DcspkgConfig;
use dcspkg::Package;
use dcspkg::{install_package, list_all_packages, list_installed_packages, run_package};

// TODO 
// lazy_static::lazy_static! {
//     static ref DCSPKG_CONFIG: dcspkg::config::DcspkgConfig = match dcspkg::config::DcspkgConfig::get() {
//         Ok(a) => a,
//         Err(_) => dcspkg::config::DcspkgConfig::default(),
//     };
// }

// This should be replaced with the lazy_static call above but I can't
// figure out how to get the types to work.
fn get_config() -> DcspkgConfig {
    DcspkgConfig::get()
        .context("Failed to get or create config file")
        .unwrap()
}


// Forwarding functions to connect the JS code to dcspkg.
#[tauri::command]
fn get_all_games() -> Vec<Package> {
    let packages = list_all_packages(get_config());

    match packages {
        Ok(p) => p,
        Err(_) => vec![],
    }
}

#[tauri::command]
fn get_installed_games() -> Vec<Package> {
    let packages = list_installed_packages(get_config());

    match packages {
        Ok(p) => p,
        Err(_) => vec![],
    }
}

#[tauri::command(async)]
fn install_game(pkgname: String) {
    println!("installing: {}", pkgname);
    std::thread::spawn(move || {
        install_package(get_config(), &pkgname)
    }).join();
}

#[tauri::command]
fn run_game(pkgname: String) {
    run_package(get_config(), &pkgname);
}

fn main() {
    tauri::Builder::default()
        .setup(|app| {
            
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_all_games,
            get_installed_games,
            install_game,
            run_game
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
