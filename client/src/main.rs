use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::UnixStream;

#[tokio::main]
async fn main() {
    let socket_path = "/tmp/hyprvisor.sock";

    // Connect to the Unix socket
    let mut stream = match UnixStream::connect(socket_path).await {
        Ok(stream) => stream,
        Err(e) => {
            eprintln!(
                "Failed to connect to Unix socket: {} | Error: {}",
                socket_path, e
            );
            return;
        }
    };

    // Subscribe to Workspace updates (change the number accordingly based on your protocol)
    let subscription_type = 1;
    stream
        .write_all(&[subscription_type])
        .await
        .expect("Failed to write subscription type");

    // Continuously listen for responses from the server
    loop {
        let mut response_buffer = [0; 256]; // Adjust the buffer size based on your expected message size
        let bytes_received = match stream.read(&mut response_buffer).await {
            Ok(bytes) => bytes,
            Err(e) => {
                eprintln!("Error reading from server: {}", e);
                break;
            }
        };

        if bytes_received == 0 {
            eprintln!("Server closed the connection");
            break;
        }

        let response_message = String::from_utf8_lossy(&response_buffer[..bytes_received]);
        println!("Received from server: {}", response_message);

        // Add your logic to process the received message from the server
    }
}
