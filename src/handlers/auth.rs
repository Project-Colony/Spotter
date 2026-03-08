use iced::Task;

use crate::app::{Message, Spotter};
use crate::messages::AuthMessage;

impl Spotter {
    pub(crate) fn handle_auth(&mut self, msg: AuthMessage) -> Task<Message> {
        match msg {
            AuthMessage::SteamLogin => return self.handle_steam_login(),
            AuthMessage::SteamLoginComplete(result) => {
                return self.handle_steam_login_complete(result);
            }
            AuthMessage::GogLogin => return self.handle_gog_login(),
            AuthMessage::GogCodeChanged(code) => {
                self.gog_code_input = code;
            }
            AuthMessage::GogSubmitCode => return self.handle_gog_submit_code(),
            AuthMessage::GogLoginComplete(result) => {
                return self.handle_gog_login_complete(result);
            }
            AuthMessage::GogTokenRefreshed(result) => match result {
                Ok((new_token, new_refresh)) => {
                    eprintln!("[app] GOG token refreshed proactively at startup");
                    self.profile.gog_token = new_token;
                    self.profile.gog_refresh_token = new_refresh;
                    return self.save_profile_task();
                }
                Err(e) => {
                    eprintln!(
                        "[app] GOG token refresh failed (will retry on import): {}",
                        e
                    );
                }
            },
            AuthMessage::EpicLogin => return self.handle_epic_login(),
            AuthMessage::EpicCodeChanged(code) => {
                self.epic_code_input = code;
            }
            AuthMessage::EpicSubmitCode => return self.handle_epic_submit_code(),
            AuthMessage::EpicLoginComplete(result) => {
                return self.handle_epic_login_complete(result);
            }
            AuthMessage::EpicTokenRefreshed(result) => match result {
                Ok((new_token, new_refresh, new_account_id, new_display_name)) => {
                    eprintln!("[app] Epic token refreshed proactively at startup");
                    self.profile.epic_token = new_token;
                    self.profile.epic_refresh_token = new_refresh;
                    self.profile.epic_account_id = new_account_id;
                    if !new_display_name.is_empty() {
                        self.profile.epic_display_name = new_display_name;
                    }
                    return self.persist_profile();
                }
                Err(e) => {
                    eprintln!(
                        "[app] Epic token refresh failed (will retry on import): {}",
                        e
                    );
                }
            },
        }
        Task::none()
    }
}
