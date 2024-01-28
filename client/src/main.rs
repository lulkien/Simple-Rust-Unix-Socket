use serde::{Deserialize, Serialize};
use serde_json::Error;
use std::{env, process};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::UnixStream;

#[derive(Serialize, Deserialize)]
struct SubscriptionInfo {
    pid: u32,
    name: String,
}

#[tokio::main]
async fn main() {
    // Get subscription id from arguments
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: {} <subscription_id>", args[0]);
        return;
    }

    let client_pid: u32 = process::id();
    let subscription_name: String = match validate_subscription_id(&args[1]) {
        Some(id) => id,
        None => {
            eprintln!("Error: Invalid sunscription id!");
            eprintln!("Allow IDs: workspace, window, sink_volume, source_volume");
            return;
        }
    };

    let subscription_message = match prepare_subscription_message(client_pid, subscription_name) {
        Ok(msg) => msg,
        Err(e) => {
            eprintln!("Failed to construct subscription message | Error: {}", e);
            return;
        }
    };

    // Connect to the Unix socket
    const SOCKET_PATH: &str = "/tmp/hyprvisor.sock";
    let mut stream = match UnixStream::connect(SOCKET_PATH).await {
        Ok(stream) => stream,
        Err(e) => {
            eprintln!(
                "Failed to connect to Unix socket: {} | Error: {}",
                SOCKET_PATH, e
            );
            return;
        }
    };

    stream
        .write_all(subscription_message.as_bytes())
        .await
        .expect("Failed to write subscription type");
    // println!("Send: {}", subscription_message);

    // Continuously listen for responses from the server
    loop {
        let mut response_buffer: [u8; 1024] = [0; 1024]; // Adjust the buffer size based on your expected message size
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
        println!("{}", response_message);
    }
}

fn validate_subscription_id(id: &String) -> Option<String> {
    let allow_id: Vec<String> = vec![
        "workspace".to_string(),
        "window".to_string(),
        "sink_volume".to_string(),
        "source_volume".to_string(),
    ];

    if allow_id.contains(id) {
        Some(id.clone())
    } else {
        None
    }
}

fn prepare_subscription_message(pid: u32, name: String) -> Result<String, Error> {
    let subscription_info = SubscriptionInfo { pid, name };
    serde_json::to_string(&subscription_info)
}
