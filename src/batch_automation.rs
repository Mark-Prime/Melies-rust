use std::process::Command;
use serde_json::{ self, json, Value };

use crate::tf2::is_tf2_running;

pub fn get_tab_settings(settings: &Value, tab: i64) -> Value {
  let mut defaults = settings["hlae"].clone();

  if tab == 0 {
    return defaults;
  }

  let alt_installs = settings["alt_installs"].as_array().unwrap();
  let tab_install = alt_installs[(tab - 1) as usize].clone();

  for key in tab_install.as_object().unwrap().keys() {
    defaults[key] = tab_install[key].clone();
  }

  return defaults;
}


pub fn before_batch(settings: &Value, tab: i64) -> Value {
  let tab_settings = get_tab_settings(settings, tab);

  match tab_settings["before_batch"].as_str().unwrap() {
    "nothing" => {
      return json!({});
    }
    "open" => {
      let _ = Command::new("explorer")
        .arg(settings["output"]["folder"].as_str().unwrap())
        .spawn()
        .unwrap();
    }
    "run" => {
      let _ = Command::new(settings["hlae"]["before_batch_path"].as_str().unwrap())
        .spawn()
        .unwrap();
    }
    _ => {
      return json!({});
    }
  }
  json!({})
}

pub fn after_batch(settings: &Value, tab: i64) -> Value {
  if is_tf2_running() {
    return json!({});
  }
  
  let tab_settings = get_tab_settings(settings, tab);

  match tab_settings["after_batch"].as_str().unwrap() {
    "nothing" => {
      return json!({});
    }
    "open" => {
      let _ = Command::new("explorer")
        .arg(settings["output"]["folder"].as_str().unwrap())
        .spawn()
        .unwrap();
    }
    "shutdown" => {
      let _ = Command::new("shutdown /s /t 0").spawn();
    }
    "run" => {
      let _ = Command::new(settings["hlae"]["after_batch_path"].as_str().unwrap()).spawn().unwrap();
    }
    _ => {
      return json!({});
    }
  }
  json!({})
}
