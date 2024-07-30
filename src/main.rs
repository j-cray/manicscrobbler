use log::{error, info};
use floem::{reactive::*, App, Cmd, View, Widget};
use notion::chrono::format;
use rspotify::{prelude::*, scopes, AuthCodeSpotify, Config, Credentials, OAuth, Token};
use notion::ids::DatabaseId;
use notion::NotionApi;
use sled::Db;
use serde::{Deserialize, Serialize};
use libsodium::{crypto_secretbox_easy, crypto_secretbox_open_easy, randombytes_buf, crypto_secretbox_KEYBYTES};
use arboard::Clipboard;

#[derive(Serialize, Deserialize)]

enum KeyGenState {
    KeyGenerated([u8; crypto_secretbox_KEYBYTES]),
    KeyGenerationFailed,
    KeyCopied,
}
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
    key_gen_state:Signal<KeyGenState>
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
        let spotify = if let Some(settings) = &settings {
            let creds = Credentials {
                id: settings.spotify_client_id.clone(),
                secret: Some(settings.spotify_client_secret.clone()),
            };
            let oauth = if let Some(settings) = &settings {
                // use redirect uri from settings
                OAuth::from_env(Some(&settings.spotify_redirect_uri))
            } else {
                //fallback to default or handle differently
                OAuth::from_env(Some("http://localhost:8888/callback"))
            };
            AuthCodeSpotify::with_config(creds, oauth, Config::default())
        } else {
            //handle cases where settings are not loaded
            AuthCodeSpotify::with_config(
                Credentials {
                    id: "YOUR_SPOTIFY_CLIENT_ID".to.string(), //Placeholder
                    secret: Some("YOUR_SPOTIFY_CLIENT_SECRET".to.string()), //Placeholder
                },
                OAuth::from_env(Some("http://localhost:8888/callback")),
                Config::default(),
            )
        };

        let app = Self {
                count: reactive(0),
                db,
                settings,
                spotify,
                notion: NotionApi::new("placeholder_token".to_string()),
        };
        (app,Cmd::none())

        }
    }

    fn update(&mut self, _msg:  ()) -> Cmd {
        Cmd::none()
    }

    // main window
    fn view (&self) -> View {
        View::new(self).content(
            Column::new()
                .push(
                    Text::new(format!("Count: {}", self.count.get()))
                        .size(30)
                ),
        )
    }

    //modal dialogue for key copying
    fn show_key_modal(&self, key: [u8; crypto_secretbox_KEYBYTES]) {
        let modal = Modal::new(self).content(
            Column::new()
                .width(Length::Fill)
                .padding(20.0)
                .spacing(20,0)
                .push(Text::new(format!("{:?}", key))) //display the key
                .push(
                    Row::new().spacing(20.0).push(
                        Button::new()
                        .text("Copy") //replace text with icon
                        .on_click(move |_| {
                            //implement clipboard functionality
                            let mut clipboard = Clipboard::new().unwrap();
                            match clipboard.set_text(format!("{:?}", key)) {
                                Ok(_) => { info!("Key copied to clipboard"); self.key_gen_state.set(KeyGenState::KeyCopied); }, //replace copy icon with check mark to indicate success
                                Err(e) => error!("Failed to copy key to clipboard: {}", e), //replace copy icon with error icon maybe?
                            };
                            Cmd::none()
                        }),
                    )
                )

        );

fn main() {
    ManicScrobbler::run();
}

//generate key for encryption and decryption of settingss stored in the database
//make this be activated by a button in settings, and send a notification to the user that they should click the button
fn generate_key(&self) -> [u8; crypto_secretbox_KEYBYTES] {
    let mut key = [0u8; crypto_secretbox_KEYBYTES];
    randombytes_buf(&mut key);
    self.key_gen_state.set(KeyGenState::KeyGenerated(key));
        let mut key = [0u8; crypto_secretbox_KEYBYTES];
    if randombytes_buf(&mut key).is_err() {
        self.key_gen_state.set(KeyGenState::KeyGenerationFailed);
        return key; //is this line necessary
    } else {
    self.key_gen_state.set(KeyGenState::KeyGenerated(key));
    };

    //show popup with key displayed so user can copy it
    self.show_key_modal(key);
    //function may need to pause here until user has copied the key
    Cmd::mew(move |_| {
        while let KeyGenState::KeyGenerated(_) = self.key_gen_state.get() {
        //might need to yield control here to prevent blocking ui
        }
        if let KeyGenState::KeyGenerationFailed = self.key_gen_state.get() {
            error!("Key generation failed"); //expand on error handling
        } else if let KeyGenState::KeyCopied = self.key_gen_state.get() {
            info!("Key copied"); //is this line necessary if KeyGenState::KeyCopied is set by the modal dialogue?
        } else {
            error!("Unknown key generation state");
        }
    });
    key
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
