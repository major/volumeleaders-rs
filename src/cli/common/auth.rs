use crate::{Client, ClientError, Session};

use crate::cli::error::client_error;

/// Browser cookie domain used for VolumeLeaders authentication.
pub const VL_DOMAIN: &str = "volumeleaders.com";

/// Build a VolumeLeaders client from browser cookies and a fresh page XSRF token.
pub async fn make_client_from_browser(domain: &str) -> Result<Client, i32> {
    let session = match crate::session_from_browser(domain) {
        Ok(session) => session,
        Err(err) => return Err(client_error(&err)),
    };

    let bootstrap_client = match Client::new(session.clone()) {
        Ok(client) => client,
        Err(err) => return Err(client_error(&err)),
    };

    let xsrf_token = match crate::extract_xsrf_token(&bootstrap_client).await {
        Ok(token) => token,
        Err(err) => return Err(client_error(&err)),
    };

    let refreshed_session = Session::new(session.cookies().to_vec(), xsrf_token);
    match Client::new(refreshed_session) {
        Ok(client) => Ok(client),
        Err(err) => Err(client_error(&err)),
    }
}

/// Build a VolumeLeaders client for the default browser cookie domain.
pub async fn make_client() -> Result<Client, i32> {
    make_client_from_browser(VL_DOMAIN).await
}

/// Convert API errors into CLI exit codes and messages.
pub fn handle_api_error(err: ClientError) -> i32 {
    client_error(&err)
}
