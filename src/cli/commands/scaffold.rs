use std::future::Future;
use std::io;
use std::pin::Pin;

use crate::{Client, ClientError};

use crate::cli::common::auth::{handle_api_error, make_client};
use crate::cli::output::finish_output;

pub(crate) async fn run_client_command<T, Fetch, Render>(fetch: Fetch, render: Render) -> i32
where
    Fetch: for<'a> FnOnce(&'a Client) -> Pin<Box<dyn Future<Output = Result<T, ClientError>> + 'a>>,
    Render: FnOnce(T) -> io::Result<()>,
{
    let client = match make_client().await {
        Ok(client) => client,
        Err(code) => return code,
    };

    let value = match fetch(&client).await {
        Ok(value) => value,
        Err(err) => return handle_api_error(err),
    };

    finish_output(render(value))
}
