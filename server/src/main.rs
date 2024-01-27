mod hyprvisor;

use hyprvisor::run_server;

#[tokio::main]
async fn main() {
    run_server().await;
}
