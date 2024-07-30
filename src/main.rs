use log::{error, info};
use floem::{reactive::*, App, Cmd, View, Widget};
use notion::chrono::format;
use rspotify::{prelude::*, scopes, AuthCodeSpotify, Config, Credentials, OAuth, Token,};
use notion::ids::DatabaseId;
use notion::NotionApi;
use sled::Db;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct Settings {
    spotify_client_id: String,
    spotify_client_secret: String,
    notion_api_token: String,
}

#[derive(Clone)]
struct ManicScrobbler {
    count: R<i32>,
    spotify: AuthCodeSpotify,
    notion: NotionApi,
    db: Db,
    settings: Option<Settings>,
}

struct Settings {
    spotify_client_id: String,
    spotify_client_secret: String,
    notion_api_token: String,

}
impl app for ManicScrobbler {
    fn new() -> (Self, Cmd) {

        let db: Db = sled::open("settings.db").unwrap();
        let settings = load_settings(&db);
        let app = Self {
                count: reactive(0),
                db,
                settings,
                spotify: AuthCodeSpotify::new(Config::default()),
                notion: NotionApi::new("placeholder_token".to_string()),
        };
        (app,Cmd::none())

        }
    }

    fn update(&mut self, _msg:  ()) -> Cmd {
        Cmd::none()
    }

    fn view (&self) -> View {
        View::new(self).content(
            Column::new()
                .push(
                    Text::new(format!("Count: {}", self.count.get()))
                        .size(30)
                ),
        )
    }


fn main() {
    ManicScrobbler::run();
}

fn save_settings(&self, db: &Db, settings: &Settings) -> Result<(), sled::Error> {
    let encrypted_settings = encrypt_settings(settings);
    let encoded_settings = bincode::serialize(&encrypted_settings).unwrap();
    db.insert("settings", &encoded_settings)?;
    db.flush()?;
    Ok(())
}

fn load_settings(db: &Db) -> Option<Settings> {
    if let Some(ivec) = db.get("settings").unwrap() {
        let settings = bincode::deserialize(&ivec).unwrap();
        Some(decrypt_settings(&decoded))
    } else {
        None
    }
}
