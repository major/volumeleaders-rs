#[tokio::main]
async fn main() {
    std::process::exit(volumeleaders_agent::run().await);
}
