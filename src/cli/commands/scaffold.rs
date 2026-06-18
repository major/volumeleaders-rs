use std::future::Future;
use std::io;
use std::pin::Pin;

use crate::{Client, ClientError};

use crate::cli::common::auth::make_client;
use crate::cli::error::CliExit;

pub(crate) async fn run_client_command<T, Fetch, Render>(
    fetch: Fetch,
    render: Render,
) -> Result<(), CliExit>
where
    Fetch: for<'a> FnOnce(&'a Client) -> Pin<Box<dyn Future<Output = Result<T, ClientError>> + 'a>>,
    Render: FnOnce(T) -> io::Result<()>,
{
    let client = make_client().await?;
    let value = fetch(&client).await?;
    Ok(render(value)?)
}
