use serde_json::{ Map, Value };
use crate::event::{ Event, EventStyle::Bookmark };
use crate::demos::tf2_class_converter;
use crate::settings::RecordToggle;

// Record all highlights
pub fn all_highlights_from_res(res: Value) -> Vec<Vec<Event>> {
  let mut events_list: Vec<Vec<Event>> = vec![];

  let binding = res.clone();
  let users = binding["data"]["users"].as_object().unwrap();
  let demo_name = res.clone()["header"]["demo_name"].as_str().unwrap().to_string();
  let settings = crate::settings::load_settings();

  for user in users {
    let lives = res["data"]["player_lives"][user.0.to_string()].clone();

    let highlights = player_highlights(&demo_name, user.1.clone(), lives, &settings);

    if highlights.len() == 0 {
      continue;
    }

    events_list.push(highlights);
  }

  set_demo_names(&mut events_list, demo_name)
}

// Record all highlights
pub fn player_highlights_from_res(res: Value, steam_id: String) -> Vec<Event> {
  let binding = res.clone();
  let users = binding["data"]["users"].as_object().unwrap();
  let demo_name = res.clone()["header"]["demo_name"].as_str().unwrap().to_string();
  let settings = crate::settings::load_settings();

  let (user_info, user_lives) = get_player_info_from_steam_id(res, users, steam_id);

  player_highlights(&demo_name, user_info, user_lives, &settings)
}

fn append_label(label: String, append: String) -> String {
  if label == "" {
    return append;
  }

  format!("{} {}", label, append)
}

fn player_highlights(
  demo_name: &String,
  user: Value,
  lives: Value,
  settings: &Value
) -> Vec<Event> {
  let automation_settings = &settings["automation"];
  let airshot_settings = &settings["advanced"]["airshots"];

  let mut events = vec![];

  if lives.is_null() {
    return events;
  }

  for life in lives.as_array().unwrap() {
    let record_whole_life =
      ((life["med_picks"].as_array().unwrap().len() > 0 &&
        automation_settings["med_picks"].as_bool().unwrap()) ||
        (life["killstreak_pointers"].as_array().unwrap().len() > 0 &&
          automation_settings["killstreaks"].as_bool().unwrap())) &&
      automation_settings["whole_life"].as_bool().unwrap();

    if record_whole_life {
      let mut life_events = life_to_bookmarks(demo_name.clone(), user.clone(), life.clone());

      events.append(&mut life_events);
      continue;
    }

    let mut kills_to_bookmark: Vec<i64> = vec![];

    if automation_settings["airshots"].as_bool().unwrap() {
      for airshot in life["airshots"].as_array().unwrap() {
        let kill = life["kills"][airshot["kill_index"].as_i64().unwrap() as usize].clone();

        let killer_toggle = RecordToggle::from_str(
          airshot_settings["killer"]
            .as_object()
            .unwrap()
            [kill["killer_class"].as_str().unwrap()].as_str()
            .unwrap()
        );
        let victim_toggle = RecordToggle::from_str(
          airshot_settings["victim"]
            .as_object()
            .unwrap()
            [kill["victim_class"].as_str().unwrap()].as_str()
            .unwrap()
        );

        let mut should_record = airshot_settings["default"].as_bool().unwrap();
        should_record = should_record_airshot(killer_toggle, &kill, should_record);
        should_record = should_record_airshot(victim_toggle, &kill, should_record);

        if should_record {
          kills_to_bookmark.push(airshot["kill_index"].as_i64().unwrap());
        }
      }
    }

    if automation_settings["med_picks"].as_bool().unwrap() {
      for med_pick in life["med_picks"].as_array().unwrap() {
        let kill_index = med_pick["kill_index"].as_i64().unwrap();

        if kills_to_bookmark.contains(&kill_index) {
          continue;
        }

        kills_to_bookmark.push(kill_index);
      }
    }

    if automation_settings["killstreaks"].as_bool().unwrap() {
      for killstreak in life["killstreak_pointers"].as_array().unwrap() {
        for kill_index in killstreak["kills"].as_array().unwrap() {
          if kills_to_bookmark.contains(&kill_index.as_i64().unwrap()) {
            continue;
          }

          kills_to_bookmark.push(kill_index.as_i64().unwrap());
        }
      }
    }

    kills_to_bookmark.sort();

    for kill_index in kills_to_bookmark {
      let kill = life["kills"][kill_index as usize].clone();

      let mut bookmark_label = String::new();

      if kill["is_airborne"].as_bool().unwrap() {
        bookmark_label = append_label(bookmark_label, "AS".to_string());
      }

      if kill["is_killstreak"].as_bool().unwrap() {
        bookmark_label = append_label(bookmark_label, "KS".to_string());
      }

      if kill["penetration"].as_bool().unwrap() {
        bookmark_label = append_label(bookmark_label, "PN".to_string());
      }

      if kill["victim_class"].as_str().unwrap() == "medic" {
        bookmark_label = append_label(bookmark_label, "MP".to_string());
      }

      let event = Event {
        event: "".to_string(),
        demo_name: demo_name.clone(),
        tick: kill["tick"].as_i64().unwrap(),
        value: Bookmark(format!("{} spec {}", bookmark_label, user["steamId64"].as_str().unwrap())),
        notes: format!(
          "{} - {}",
          user["name"].as_str().unwrap(),
          kill["killer_class"].as_str().unwrap()
        ),
      };

      events.push(event);
    }
  }

  events
}

fn should_record_airshot(toggle: RecordToggle, kill: &Value, default: bool) -> bool {
  if toggle == RecordToggle::Never {
    return false;
  }

  return match toggle {
    RecordToggle::Never => false,
    RecordToggle::CriticalHit => kill["crit_type"].as_i64().unwrap() == 2,
    RecordToggle::AnyCritHit => kill["crit_type"].as_i64().unwrap() > 0,
    RecordToggle::Always => true,
    RecordToggle::Passive => default,
  };
}

// Record all lives but skip deaths
pub fn all_lives_from_res(res: Value, include_empty_lives: bool) -> Vec<Vec<Event>> {
  let mut events_list: Vec<Vec<Event>> = vec![];
  let users = res["data"]["users"].as_object().unwrap();
  let demo_name = res["header"]["demo_name"].as_str().unwrap().to_string();

  for user in users {
    let lives = player_lives(
      demo_name.clone(),
      user.1.clone(),
      res["data"]["player_lives"][user.0.to_string()].clone(),
      include_empty_lives
    );

    if lives.len() == 0 {
      continue;
    }

    events_list.push(lives);
  }

  set_demo_names(&mut events_list, demo_name)
}

fn set_demo_names(events_list: &mut Vec<Vec<Event>>, demo_name: String) -> Vec<Vec<Event>> {
  let events_length = events_list.len();

  if [0, 1].contains(&events_list.len()) {
    return events_list.to_vec();
  }

  events_list.sort_by(|a, b| b.len().cmp(&a.len()));

  let settings = crate::settings::load_settings();

  let mut new_events_list: Vec<Vec<Event>> = vec![
    vec![Event {
      event: "".to_string(),
      demo_name: demo_name.clone(),
      tick: settings["recording"]["start_delay"].as_i64().unwrap(),
      value: Bookmark(format!("mls_load_vdm {}~0", demo_name)),
      notes: "".to_string(),
    }]
  ];

  for (index, events) in &mut events_list.iter_mut().enumerate() {
    for event in events.iter_mut() {
      event.demo_name = format!("{}~{}", demo_name, index);
    }

    if index < events_length - 1 {
      let last_event = events[events.len() - 1].clone();

      events.push(
        Event {
          event: "".to_string(),
          demo_name: format!("{}~{}", demo_name, index),
          tick: last_event.tick + settings["recording"]["start_delay"].as_i64().unwrap(),
          value: Bookmark(format!("mls_load_vdm {}~{}", demo_name, index + 1)),
          notes: "".to_string(),
        }
      );
    }

    new_events_list.push(events.clone());
  }

  new_events_list
}

fn get_player_info_from_steam_id(
  res: Value,
  users: &Map<String, Value>,
  steam_id: String
) -> (Value, Value) {
  let mut user_info = Value::Null;
  let mut user_lives = Value::Null;

  for user in users {
    if user.1["steamId64"].as_str().unwrap().to_string() != steam_id {
      continue;
    }

    user_info = user.1.clone();
    user_lives = res["data"]["player_lives"][user.0.to_string()].clone();
  }

  (user_info, user_lives)
}

// Record every life from a specific player but skip deaths
pub fn player_lives_from_res(
  res: Value,
  steam_id: String,
  include_empty_lives: bool
) -> Vec<Event> {
  let binding = res.clone();
  let users = binding["data"]["users"].as_object().unwrap();
  let demo_name = res.clone()["header"]["demo_name"].as_str().unwrap().to_string();

  let (user_info, user_lives) = get_player_info_from_steam_id(res, users, steam_id);

  player_lives(demo_name.clone(), user_info.clone(), user_lives, include_empty_lives)
}

fn player_lives(
  demo_name: String,
  user: Value,
  lives: Value,
  include_empty_lives: bool
) -> Vec<Event> {
  let mut events: Vec<Event> = vec![];

  if lives.is_null() {
    return events;
  }

  for life in lives.as_array().unwrap() {
    if
      life["kills"].as_array().unwrap().len() > 0 ||
      life["assists"].as_array().unwrap().len() > 0 ||
      include_empty_lives
    {
      events.append(&mut life_to_bookmarks(demo_name.clone(), user.clone(), life.clone()));
    }
  }

  events
}

pub fn life_to_bookmarks(demo_name: String, user: Value, life: Value) -> Vec<Event> {
  let class_string = life["classes"]
    .as_array()
    .unwrap()
    .iter()
    .map(|class| class.as_str().unwrap())
    .collect::<Vec<_>>()
    .join(", ");

  let start = Event {
    event: "".to_string(),
    demo_name: demo_name.clone(),
    tick: life["start"].as_i64().unwrap(),
    value: Bookmark(format!("clip_start spec {}", user["steamId64"].as_str().unwrap())),
    notes: format!("{} - {}", user["name"].as_str().unwrap(), class_string),
  };

  let end = Event {
    event: "".to_string(),
    demo_name: demo_name.clone(),
    tick: life["end"].as_i64().unwrap(),
    value: Bookmark("clip_end".to_string()),
    notes: "".to_string(),
  };

  vec![start, end]
}

// Record the POV of all players
pub fn all_pov_from_res(res: Value) -> Vec<Event> {
  let mut events: Vec<Event> = vec![];
  let users = res["data"]["users"].as_object().unwrap();
  let demo_name = res["header"]["demo_name"].as_str().unwrap().to_string();

  for user in users {
    let steam_id = user.1["steamId64"].as_str().unwrap().to_string();
    let player_classes = user.1["classes"].as_object().unwrap().keys().collect::<Vec<_>>();

    if player_classes.len() < 1 {
      continue;
    }

    let player_class_string = get_class_string(player_classes);

    let mut event = player_pov(demo_name.clone(), steam_id.clone());
    event.notes = format!(
      "{} - {} - Full POV",
      user.1["name"].as_str().unwrap(),
      player_class_string
    );

    events.push(event);
  }

  events
}

// Record a specific players POV from start to end
pub fn player_pov(demo_name: String, steam_id: String) -> Event {
  let settings = crate::settings::load_settings();

  Event {
    event: "".to_string(),
    demo_name: demo_name.clone(),
    tick: settings["recording"]["start_delay"].as_i64().unwrap(),
    value: Bookmark(format!("mls_rec_demo spec {}", steam_id)),
    notes: "Full POV".to_string(),
  }
}

fn get_class_string(player_classes: Vec<&String>) -> String {
  player_classes
    .iter()
    .map(|x| tf2_class_converter(x.to_string()))
    .collect::<Vec<_>>()
    .join(", ")
}
