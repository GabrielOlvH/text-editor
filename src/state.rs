use std::fs::{File, OpenOptions};
use std::io::{Read, Write};
use serde::{Deserialize, Serialize};
use serde_json::to_string_pretty;

#[derive(Serialize, Deserialize)]
pub struct State {
    pub data_dir: String,
    pub background_image_path: Option<String>,
    pub last_open_file: Option<String>,
    pub theme: String
}

impl State {
    pub fn new() -> Self {
        State {
            data_dir: "cache".to_string(),
            background_image_path: None,
            last_open_file: None,
            theme: "Default".to_string()
        }
    }

    pub fn read(self) -> Result<Self, ()> {
        let mut content = String::new();
        let file = File::open("state.json");
        if !file.is_ok() {
            return Ok(self);
        }
        file.unwrap().read_to_string(&mut content).expect("Failed to load state");

        let state: State = serde_json::from_str(&content).unwrap();
        return Ok(state);
    }

    pub fn save(&self) -> Result<(), ()> {
        let json = to_string_pretty(self);
        let mut file = OpenOptions::new()
            .write(true)
            .truncate(true)
            .create(true)
            .open("state.json").expect("aaaaa");

        writeln!(&mut file, "{}", json.unwrap()).expect("caralho");
        println!("Saved state");
        Ok(())
    }
}
