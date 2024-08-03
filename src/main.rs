use log::{error, info};
use floem::{reactive::*, App, Cmd, View, Widget};
use rspotify::{prelude::*, scopes, AuthCodeSpotify, Config, Credentials, OAuth, Token};
use notion::chrono::format;
use notion::ids::DatabaseId;
use notion::NotionApi;
use sled::Db;
use serde::{Deserialize, Serialize};

// structures //

#[derive(Clone)]
struct ManicScrobbler {
    spotify: AuthCodeSpotify,
    notion: NotionApi,
    db: Db,
    settings: Option<Settings>,
    current_view: Signal<CurrentView>,
}

//settings
#[derive(Serialize, Deserialize)]
struct Settings {
    spotify_client_id: String,
    spotify_client_secret: String,
    spotify_redirect_uri: String,
    notion_api_token: String,
    notion_database_id: String,
}

// app
impl app for ManicScrobbler {
    fn new() -> (Self, Cmd) {

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
                current_view: Signal::new(CurrentView::Main),
                db,
                settings,
                spotify,
                notion: NotionApi::new("placeholder_token".to_string()),

        };
        (app,Cmd::none())

        }
    }

    //UI//
    fn view (&self) -> View {
        // render main view or 
        if self.current_view.get() == CurrentView::Main {
            self.view_main()
        } else if self.current_view.get() == CurrentView::Settings {
            self.view_settings()
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
                }

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
                                        let spotify_client_id = world.get_widget::<TextInput>(0).text().clone();
                                        let spotify_client_secret = world.get_widget::<TextInput>(1).text().clone();
                                        let spotify_redirect_uri = world.get_widget::<TextInput>(2).text().clone();
                                        let notion_api_token = world.get_widget::<TextInput>(3).text().clone();
                                        let notion_database_id = world.get_widget::<TextInput>(4).text().clone();
                                        let settings = Settings {
                                            spotify_client_id,
                                            spotify_client_secret,
                                            spotify_redirect_uri,
                                            notion_api_token,
                                            notion_database_id,
                                        };
                                        self.save_settings(&self.db, &settings).unwrap();
                                        // Optionally show a success message or feedback
                                        Cmd::none();

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
                                        .push(TextInput::new().placeholder("Spotify Client ID").text(
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
                                        .push(TextInput::new().placeholder("Spotify Client Secret").text(
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
                                        .push(TextInput::new().placeholder("Spotify Redirect URI").text(
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
                                            .push(TextInput::new().placeholder("Notion API Token").text(
                                                if let Some(settings) = &self.settings {
                                                    settings.notion_api_token.clone()
                                                } else {
                                                    "".to_string()
                                                },
                                            )),
                                        )
                                        .push(
                                            Row::new()
                                                .spacing(20.0)
                                                .padding(20.0)
                                            .push(TextInput::new().placeholder("Notion Database ID").text(
                                                if let Some(settings) = &self.settings {
                                                    settings.notion_database_id.clone()
                                                } else {
                                                    "".to_string()
                                                },
                                            )),
                                        )
                                    )
                            )
                    );

                }
            }

//functions//

fn update_notion() {
    ManicScrobbler::run();
}


fn save_settings(&self, db: &Db, settings: &Settings) -> Result<(), sled::Error> {
    let encrypted_settings = encrypt_settings(settings);
    let encoded_settings = bincode::serialize(&encrypted_settings).unwrap();
    db.insert("settings", &encoded_settings)?;
    db.flush()?;
    Ok(())
}

// create logic to load settings at startup
fn load_settings(db: &Db) -> Option<Settings> {
    if let Some(ivec) = db.get("settings").unwrap() {
        let settings = bincode::deserialize(&ivec).unwrap();
        Some(decrypt_settings(&decoded))
    } else {
        None
    }


}
