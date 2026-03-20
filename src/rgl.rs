use std::{fs::{File, read_to_string}};

use serde_json::{ json, Value };
use reqwest;

use crate::settings::load_settings;

pub fn get_users(steam_ids: Vec<String>) -> Value {
  let mut users: Value = json!({});
  let resp = reqwest::blocking::Client
    ::new()
    .post("https://api.rgl.gg/v0/profile/getmany")
    .header("Content-Type", "application/json")
    .body(serde_json::to_string(&steam_ids).unwrap())
    .send();

  if resp.is_err() {
    return users;
  }

  let resp = resp.unwrap();

  // println!("RGL response: {}", resp.text().unwrap());

  users["response"] = resp.json().unwrap();

  let response = users["response"].clone();

  if response["error"].as_str().is_some() {
    return users;
  }

  for user in response.as_array().unwrap() {
    let steam_id = user["steamId"].as_str().unwrap();
    users[steam_id] = user.clone();

    // The RGL website wont let me add () to my name
    // Drastic times call for drastic measures
    if steam_id == "76561198045517514" {
      users[steam_id]["name"] = serde_json::Value::String("Maven (famous)".to_string());
    }
  }

  users
}

fn read_lines(filename: &str) -> Vec<String> {
    read_to_string(filename)
        .unwrap()  // panic on possible file-reading errors
        .lines()  // split the string into an iterator of string slices
        .map(String::from)  // make each slice into a string
        .collect()  // gather them together into a vector
}

pub fn save_users_to_cfg(users: Value) {
  let settings = load_settings();

  if settings["features"]["demo_scanner"]["rgl_force_rename"].as_bool().unwrap() == false {
    return;
  }

  let file_path = format!("{}\\cfg\\mls_rgl_aliases.cfg", settings["tf_folder"].as_str().unwrap());
  
  let mut lines: Vec<String> = match File::open(&file_path) {
    Ok(_) => read_lines(file_path.as_str()),
    Err(_) => vec![],
  };
  
  use std::io::Write;

  for user_id in users.as_object().unwrap().keys() {
    if user_id == "response" {
      continue;
    }

    let name = &users[user_id]["name"];
    let line = format!("ce_playeraliases_add {} {};", user_id, name);

    if !lines.contains(&line) {
      lines.push(line);
    }
  }

  let mut cfg_file = File::create(&file_path).unwrap();

  cfg_file.write_all(lines.join("\n").as_bytes()).unwrap();
}
