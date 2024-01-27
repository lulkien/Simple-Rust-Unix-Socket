mod hyprvisor;
mod protocols;
mod server;

use server::Server;

#[tokio::main]
async fn main() {
    let mut server = Server::new("/tmp/hyprvisor.sock".to_string()).await;
    server.prepare().await;
    server.start().await;
}
