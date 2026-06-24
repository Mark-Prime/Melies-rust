use std::fs::{File};
use std::io::Write;
use chrono::{ DateTime, Local };
use regex::Regex;
use serde_json::{ Value, json };

use crate::{
  event::{ Event, EventStyle::{ Bookmark, Killstreak } },
  macros::extend,
  settings::load_settings,
  util::find_dir,
};

pub fn save_events(new_events: Value) -> Value {
  let new_events = new_events.as_array().unwrap();
  let mut events: Vec<Event> = vec![];

  for demo in new_events {

    for event in demo.as_array().unwrap() {
      let re = Regex::new("\\[(.*)\\] (.*) \\(\"(.*)\" at (\\d*)\\)(.*)").unwrap();

      if event["event"].as_str().is_none() {
        continue;
      }

      let events_regex = match re.captures(event["event"].as_str().unwrap()) {
        Some(val) => val,
        None => {
          println!("Failed to parse event: {}", event["event"].as_str().unwrap());
          continue;
        }
      };

      let original_event = Event::new(events_regex).unwrap();

      let is_killstreak = match &original_event.value {
        Bookmark(_) => true,
        Killstreak(_) => false,
      };

      if
        event["demo_name"].as_str().unwrap() != original_event.demo_name ||
        event["tick"].as_i64().unwrap() != original_event.tick ||
        event["notes"].as_str().unwrap_or("") != original_event.notes ||
        event["isKillstreak"].as_bool().unwrap() != is_killstreak
      {
        let built_event = build_event_from_json(event);
        events.push(built_event);
        continue;
      }

      match &original_event.value {
        Bookmark(bm) => {
          if bm.to_owned() != event["value"]["Bookmark"].as_str().unwrap() {
            let built_event = build_event_from_json(event);
            events.push(built_event);
            continue;
          }
        }
        Killstreak(ks) => {
          if ks.to_owned() != event["value"]["Killstreak"].as_i64().unwrap() {
            let built_event = build_event_from_json(event);
            events.push(built_event);
            continue;
          }
        }
      }

      events.push(original_event);
    }
  }

  write_events(events, false)
}

pub fn write_events(events: Vec<Event>, append: bool) -> Value {
  let mut new_events_text = String::new();

  let mut current_demo = "".to_string();

  for event in events.clone() {
    if event.demo_name != current_demo {
      extend!(new_events_text, "{}\n", ">");
      let demo_name = event.demo_name.clone();
      current_demo = demo_name;
    } else {
      extend!(new_events_text, "{}", "\n");
    }

    extend!(new_events_text, "{}", match event.value {
      Bookmark(val) => format!("[melies] {} (\"{}\" at {}) {}", val, event.demo_name, event.tick, event.notes),
      Killstreak(val) => format!("[melies] killstreak {} (\"{}\" at {}) {}", val, event.demo_name, event.tick, event.notes),
    });
  }

  let settings = load_settings();

  let dir;

  match find_dir(&settings) {
    Ok(directory) => {
      dir = directory;
    }
    Err(err) => {
      return json!({
          "code": 404,
          "err_text": err
        });
    }
  }

  let mut file = File::options().write(true).append(append).open(dir).unwrap();

  if let Err(e) = writeln!(&mut file, "{new_events_text}") {
      eprintln!("Couldn't write to file: {}", e);
  }

  return json!({
    "code": 200,
    "events": events
  });
}

fn build_event_from_json(event_json: &Value) -> Event {
  let sys_time: DateTime<Local> = Local::now();

  match event_json["isKillstreak"].as_bool().unwrap() {
    true => {
      return Event {
        event: format!(
          "[{}] Killstreak {} (\"{}\" at {}){}",
          sys_time.format("%Y/%m/%d %H:%M").to_string().replace("\"", ""),
          event_json["value"]["Killstreak"],
          event_json["demo_name"].as_str().unwrap(),
          event_json["tick"].as_i64().unwrap(),
          event_json["notes"].as_str().unwrap_or("").to_string()
        ),
        demo_name: event_json["demo_name"].as_str().unwrap().to_string(),
        tick: event_json["tick"].as_i64().unwrap(),
        value: Killstreak(event_json["value"]["Killstreak"].as_i64().unwrap()),
        notes: event_json["notes"].as_str().unwrap_or("").to_string(),
      };
    }
    false => {
      if event_json["value"]["Bookmark"] == "General" {
        return Event {
          event: format!(
            "[{}] Bookmark {} (\"{}\" at {}){}",
            sys_time.format("%Y/%m/%d %H:%M").to_string(),
            event_json["value"]["Bookmark"].as_str().unwrap(),
            event_json["demo_name"].as_str().unwrap(),
            event_json["tick"].as_i64().unwrap(),
            event_json["notes"].as_str().unwrap_or("").to_string()
          ),
          demo_name: event_json["demo_name"].as_str().unwrap().to_string(),
          tick: event_json["tick"].as_i64().unwrap(),
          value: Bookmark(event_json["value"]["Bookmark"].as_str().unwrap().to_string()),
          notes: event_json["notes"].as_str().unwrap_or("").to_string(),
        };
      }

      return Event {
        event: format!(
          "[{}] {} (\"{}\" at {}){}",
          sys_time.format("%Y/%m/%d %H:%M").to_string(),
          event_json["value"]["Bookmark"].as_str().unwrap(),
          event_json["demo_name"].as_str().unwrap(),
          event_json["tick"].as_i64().unwrap(),
          event_json["notes"].as_str().unwrap_or("").to_string()
        ),
        demo_name: event_json["demo_name"].as_str().unwrap().to_string(),
        tick: event_json["tick"].as_i64().unwrap(),
        value: Bookmark(event_json["value"]["Bookmark"].as_str().unwrap().to_string()),
        notes: event_json["notes"].as_str().unwrap_or("").to_string(),
      };
    }
  }
}
