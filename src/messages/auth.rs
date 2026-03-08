#[derive(Debug, Clone)]
pub enum AuthMessage {
    // Steam
    SteamLogin,
    SteamLoginComplete(Result<String, String>),
    // GOG
    GogLogin,
    GogCodeChanged(String),
    GogSubmitCode,
    GogLoginComplete(Result<(String, String), String>),
    GogTokenRefreshed(Result<(String, String), String>),
    // Epic
    EpicLogin,
    EpicCodeChanged(String),
    EpicSubmitCode,
    EpicLoginComplete(Result<crate::epic::auth::EpicLoginResult, String>),
    EpicTokenRefreshed(Result<crate::epic::auth::EpicLoginResult, String>),
}
