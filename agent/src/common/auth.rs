use volumeleaders_client::{Client, ClientError, Session};

/// Browser cookie domain used for VolumeLeaders authentication.
pub const VL_DOMAIN: &str = "volumeleaders.com";

/// Build a VolumeLeaders client from browser cookies and a fresh page XSRF token.
pub async fn make_client_from_browser(domain: &str) -> Result<Client, i32> {
    let session = match volumeleaders_client::session_from_browser(domain) {
        Ok(session) => session,
        Err(err) => {
            eprintln!("auth error: {err}");
            return Err(2);
        }
    };

    let bootstrap_client = match Client::new(session.clone()) {
        Ok(client) => client,
        Err(err) => {
            eprintln!("client error: {err}");
            return Err(1);
        }
    };

    let xsrf_token = match volumeleaders_client::extract_xsrf_token(&bootstrap_client).await {
        Ok(token) => token,
        Err(err) => {
            if err.is_auth_error() {
                eprintln!("auth error: {err}");
                return Err(2);
            }
            eprintln!("client error: {err}");
            return Err(1);
        }
    };

    let refreshed_session = Session::new(session.cookies().to_vec(), xsrf_token);
    match Client::new(refreshed_session) {
        Ok(client) => Ok(client),
        Err(err) => {
            eprintln!("client error: {err}");
            Err(1)
        }
    }
}

/// Build a VolumeLeaders client for the default browser cookie domain.
pub async fn make_client() -> Result<Client, i32> {
    make_client_from_browser(VL_DOMAIN).await
}

/// Convert API errors into CLI exit codes and messages.
pub fn handle_api_error(err: ClientError) -> i32 {
    if err.is_auth_error() {
        eprintln!("auth error: {err}");
        2
    } else {
        eprintln!("API error: {err}");
        1
    }
}
