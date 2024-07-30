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

// structures //

#[derive(Clone)]
struct ManicScrobbler {
    spotify: AuthCodeSpotify,
    notion: NotionApi,
    db: Db,
    settings: Option<Settings>,
    key_gen_state:Signal<KeyGenState>,
    current_view: Signal<CurrentView>,
    // view_main: Signal::new(CurrentView::Main),
    // view_settings: Signal::new(CurrentView::Settings),
    modal_visible: Signal<bool>,
}

//settings
#[derive(Serialize, Deserialize)]
struct Settings {
    spotify_client_id: String,
    spotify_client_secret: String,
    notion_api_token: String,
}

// app
impl app for ManicScrobbler {
    fn new() -> (Self, Cmd) {

        enum KeyGenState {
            KeyGenerated([u8; crypto_secretbox_KEYBYTES]),
            KeyGenerationFailed,
            KeyCopied,
        }
        //view switching
        enum CurrentView {
            Main,
            Settings,
        }
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
                key_gen_state: Signal::new(KeyGenState::KeyGenerationFailed),
                current_view: Signal::new(CurrentView::Main), //if this is set, will it force main view always on?
                db,
                settings,
                spotify,
                notion: NotionApi::new("placeholder_token".to_string()),

        };
        (app,Cmd::none())

        }
    }

    fn update(&mut self, _msg:  ()) -> Cmd { //what does this snippet do? lol
        Cmd::none()
    }

    //UI//
    fn view (&self) -> View {
        if self.current_view.get() == CurrentView::Main {} //render main view
        else if self.current_view.get() == CurrentView::Settings {}//render settings view
        if self.modal_visible.get() {
            self.show_key_modal(self.key_gen_state.get().unwrap()); // Assuming KeyGenerated holds the key
}

        }

            //main view
            fn view_main(&self) -> View {
                View::new(self).content(
                    Column::new()
                            .width(Length::Fill)
                            .padding(20.0)
                            .spacing(20,0)
                            .push(
                                Button::new()
                                    .text("Update Notion")
                                    .on_click(move |world| {
                                        // Call the update_notion function
                                        let cmd = world.get::<ManicScrobbler>().update_notion(); // Or just self.update_notion() if it's a method
                                        // Dispatch the command if necessary
                                        if let Some(cmd) = cmd {
                                            world.dispatch(cmd); 
                                        }
                                    }),
                            )
                            .push(
                                Column::new()
                                    .width(Length::Fill)
                                    .padding(20.0)
                                    .spacing(20,0)
                                    .push(
                                        Button::new()
                                            .text("Settings") // replace with icon
                                            .on_click(move |world| {
                                                self.current_view.set(CurrentView::Settings)
                                            }),
                                        ),
                                ),
                    );
                };

            // settings view
            fn view_settings(&self) -> View {
                View::new(self).content(
                    Column::new()
                        .width(Length::Fill)
                        .padding(20.0)
                        .spacing(20,0),
                        Row::new()
                            .spacing(20.0)
                            .padding(20.0)
                            .push(
                                // Back button
                                Button::new()
                                    .text("Back")
                                    .on_click(move |world| {
                                        self.current_view.set(CurrentView::Main)
                                    }),
                                ),
                        Row::new()
                            .spacing(20.0)
                            .padding(20.0)
                            .push(
                                // Save button
                                Button::new()
                                    .text("Save")
                                    .on_click(move |world| {
                                        // Access the TextInput values and create a new Settings instance
                                        // ...
                                        // Save the settings (using self.save_settings or a similar method)
                                        // ...
                                        // Optionally show a success message or feedback
                                        // ...
                                        Cmd::none()
                                    }),
                                )
                        .push(
                            Column::new()
                                .width(Length::Fill)
                                .padding(20.0)
                                .spacing(20,0)
                                // spotify settings
                                .push(
                                    Row::new()
                                        .spacing(20.0)
                                        .padding(20.0)
                                        .push(
                                            Text::new("Spotify")
                                            .size(20)
                                        )
                                    )    
                                .push(
                                    Row::new()
                                        .spacing(20.0)
                                        .padding(20.0)
                                        .push(TextInput::new().placeholder("Client ID").text(
                                            if let Some(settings) = &self.settings {
                                                settings.spotify_client_id.clone()
                                            } else {
                                                "".to_string()
                                            },
                                        ))
                                    )
                                .push(
                                    Row::new()
                                        .spacing(20.0)
                                        .padding(20.0)
                                        .push(TextInput::new().placeholder("Client Secret").text(
                                            if let Some(settings) = &self.settings {
                                                settings.spotify_client_secret.clone()
                                            } else {
                                                "".to_string()
                                            },
                                        ))
                                    )
                                .push(
                                    Row::new()
                                        .spacing(20.0)
                                        .padding(20.0)
                                        .push(TextInput::new().placeholder("Redirect URI").text(
                                            if let Some(settings) = &self.settings {
                                                settings.spotify_redirect_uri.clone()
                                            } else {
                                                "".to_string()
                                            },
                                        )),
                                    )
                                .push(
                                    // Notion settings
                                    Row::new()
                                        .padding(20.0)
                                        .spacing(10.0)
                                        .push(Text::new("Notion").size(20.0))
                                        .push(
                                            Row::new()
                                                .spacing(20.0)
                                                .padding(20.0)
                                            .push(TextInput::new().placeholder("API Token").text(
                                                if let Some(settings) = &self.settings {
                                                    settings.notion_api_token.clone()
                                                } else {
                                                    "".to_string()
                                                },
                                            )),
                                        )
                                    )
                                .push(
                                    // Key generation
                                    Row::new()
                                        .spacing(20.0)
                                        .padding(20.0)
                                        .push(
                                            Button::new()
                                                .text("Generate Key")
                                                .on_click(move |world| {
                                                    // Set modal_visible to true
                                                    world.get::<ManicScrobbler>().modal_visible.set(true);
                                                    // Show the modal dialog
                                                    // world.get::<ManicScrobbler>().show_key_modal(); 
                                                    Cmd::none()
                                                }),
                                            ),
                                    )
                            )
                    );
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
                                    //use clipboard functionality
                                    let mut clipboard = Clipboard::new().unwrap();
                                    match clipboard.set_text(format!("{:?}", key)) {
                                        Ok(_) => { info!("Key copied to clipboard"); self.key_gen_state.set(KeyGenState::KeyCopied); }, //replace copy icon with check mark to indicate success
                                        Err(e) => error!("Failed to copy key to clipboard: {}", e), //replace copy icon with error icon maybe?
                                    };
                                    Cmd::none()
                                }),
                            )
                        )
                        .push(
                            Row::new().spacing(20.0).push(
                                Button::new()
                                .text("Continue") // replace text with icon
                                .enabled(self.key_gen_state.map(|state| *state == KeyGenState::KeyCopied)) //enable only when key has been copied
                                .on_click(move |_| {
                                    Cmd::new(move |world| {
                                        world.get::<ManicScrobbler>().modal_visible.set(false); 
                                        Cmd::none()
                                    })
                                }),
                            )
                        )
                    );
                }
      }


//functions//

fn update_notion() {
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
        return key;
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
