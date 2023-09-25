// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::path::PathBuf;
use std::path::Path;
use serde::{Serialize, Deserialize};
use tauri::api::dialog;
use tauri::{CustomMenuItem, Menu, Submenu, Manager};
use merkle_hash::{bytes_to_hex, Algorithm, MerkleTree, anyhow::Error};
use std::fs::{File, self, create_dir_all};
use std::io::{Read, Write};
use std::io;
use reqwest::blocking::multipart::*;

#[derive(Serialize, Deserialize)]
struct LocalCADFile {
    path: String,
    size: u64,
    hash: String
}

#[derive(Serialize, Deserialize)]
struct S3FileLink {
    path: String,
    url: String
}

#[tauri::command]
fn download_s3_file(app_handle: tauri::AppHandle, link: S3FileLink) {
    println!("download");
    let mut resp = reqwest::blocking::get(link.url).unwrap();
    let path = get_project_dir(app_handle) + link.path.as_str();
    let p: &Path = std::path::Path::new(&path);
    let prefix = p.parent().unwrap();
    println!("prefix: {}", &prefix.display());
    fs::create_dir_all(prefix).unwrap();

    let mut f = File::create(&path).expect("Unable to create file");
    io::copy(&mut resp, &mut f).expect("Unable to copy data");
    println!("finish");
    println!("loc: {}", path);
}

#[tauri::command]
fn upload_changes(app_handle: tauri::AppHandle, files: Vec<LocalCADFile>, commit: u64, server_url: String) -> Result<(), ()> {
    let client = reqwest::blocking::Client::new();
    let url = server_url.to_string() + "/ingest";
    //let url = "http://localhost:5000/ingest";
    println!("commit # {}; server url {}", commit, &url);

    let project_dir = get_project_dir(app_handle);
    println!("{}", project_dir);

    for file in files {
        let path: String = file.path;
        let relative_path = path.replace(&project_dir, "");

        println!("uploading {}", path);
        println!("relative {}", relative_path);
        let form = reqwest::blocking::multipart::Form::new()
            .text("commit", commit.to_string())
            .text("path", relative_path)
            .text("size", file.size.to_string())
            .text("hash", file.hash)
            .file("key", path).unwrap();
        
        let res = client.post(url.to_string())
            .multipart(form)
            .send().unwrap();

        println!("{:?}", res);

    }
    println!("upload done");
    Ok(())
}

#[tauri::command]
fn update_server_url(app_handle: tauri::AppHandle, new_url: String) {
    let appdir = app_handle.path_resolver().app_local_data_dir().unwrap();
    let path = appdir.join("server_url.txt");

    let _ = fs::write(path, new_url);
}

#[tauri::command]
fn get_server_url(app_handle: tauri::AppHandle) -> String {
    let appdir = app_handle.path_resolver().app_local_data_dir().unwrap();
    let path = appdir.join("server_url.txt");
    let output: String = match fs::read_to_string(path) {
        Ok(contents) => return contents,
        Err(_err) => "http://localhost:5000".to_string(),
    };
    return output;
}

#[tauri::command]
fn get_project_dir(app_handle: tauri::AppHandle) -> String {
    let appdir = app_handle.path_resolver().app_local_data_dir().unwrap();
    let path = appdir.join("project_dir.txt");
    let output: String = match fs::read_to_string(path) {
        Ok(contents) => return contents,
        Err(_err) => "set project directory please!!!".to_string(),
    };
    return output;
}

fn update_project_dir(app_handle: tauri::AppHandle, dir: PathBuf) {
    println!("new project dir: {}", dir.display());

    let appdir = app_handle.path_resolver().app_local_data_dir().unwrap();
    let mut path = appdir.join("project_dir.txt");

    let _ = fs::write(path, pathbuf_to_string(dir));

    // update base.json
    path = appdir.join("base.json");
    hash_dir(app_handle, &pathbuf_to_string(path));
}

fn pathbuf_to_string(path: PathBuf) -> String {
    let output: String = path.into_os_string().into_string().unwrap();
    return output;
}

// Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
#[tauri::command]
fn greet(name: &str) -> String {
    let output: String = format!("Hello, {}! You've been greeted from Rust!", name);
    println!("helasdfasdfasdfsdf");
    return output;
}

#[tauri::command]
fn hash_dir(app_handle: tauri::AppHandle, results_path: &str) {
    let path: String = get_project_dir(app_handle);
    if path == "no lol" {
        return;
    }
    
    let mut files: Vec<LocalCADFile> = Vec::new();

    let do_steps = || -> Result<(), Error> {
        let tree = MerkleTree::builder(path)
        .algorithm(Algorithm::Blake3)
        .hash_names(true)
        .build()?;

        for item in tree {
            let pathbuf = item.path.absolute.into_string();
            let s_hash = bytes_to_hex(item.hash);

            if pathbuf.as_str() == "" {
                continue;
            }
            let metadata = std::fs::metadata(pathbuf.as_str())?;
            let isthisfile = metadata.is_file();
            let filesize = metadata.len();

            if !isthisfile {
                continue;
            }

            println!("{}: {}", pathbuf, s_hash);
            let file = LocalCADFile {
                hash: s_hash,
                path: pathbuf,
                size: filesize,
            };
            files.push(file);
        }

        let json = serde_json::to_string(&files)?;

        let mut file = File::create(results_path)?;
        file.write_all(json.as_bytes())?;
        Ok(())
    };
    let _ = do_steps();

    println!("fn hash_dir done");
}

fn main() {
    let set_dir = CustomMenuItem::new("Set Project Directory".to_string(), "Set Project Directory");
    let file_menu = Submenu::new("Configure", Menu::new().add_item(set_dir));
    let menu = Menu::new()
      .add_submenu(file_menu);

    tauri::Builder::default()
        .menu(menu)
        .on_menu_event(|event| match event.menu_item_id() {
        "Set Project Directory" => {
            dialog::FileDialogBuilder::default()
            .pick_folder(move |path_buf| match path_buf {
                Some(p) => {
                    println!("{}", p.display());
                    update_project_dir(event.window().app_handle(), p);
                }
                _ => {}
            });
        }
        _ => {}
        })
        .invoke_handler(tauri::generate_handler![hash_dir, greet, get_project_dir, upload_changes, update_server_url, get_server_url, download_s3_file])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
