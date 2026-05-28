#[tokio::main]
async fn main() {
    std::process::exit(rusty_volumeleaders::cli::run().await);
}
