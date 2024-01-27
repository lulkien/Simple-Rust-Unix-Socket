use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::sync::Arc;
use tokio::io::AsyncWriteExt;
use tokio::net::{UnixListener, UnixStream};
use tokio::sync::Mutex;

enum WorkspaceState {
    Active = 0,
    Occupied = 1,
    Empty = 2,
}

struct HyprvisorData {
    workspace_info: Vec<WorkspaceState>,
    window_title: String,
    sink_volume: Option<u32>,   // None -> muted, Some -> Volume
    source_volume: Option<u32>, // None -> muted, Some -> Volume
}

#[derive(Debug, Eq, PartialEq, Hash, Clone, Copy)]
enum Subscription {
    Workspace,
    Window,
    AudioSinkVolume,
    AudioSourceVolume,
}

#[derive(Serialize, Deserialize)]
struct SubscriptionInfo {
    pid: u32,
    name: String,
}

struct HyprvisorState {
    data: HyprvisorData,
    subscribers: HashMap<Subscription, HashMap<u32, UnixStream>>,
}

#[tokio::main]
async fn main() {
    let socket_path = "/tmp/hyprvisor.sock";

    // Try remove old socket first
    let _ = fs::remove_file(socket_path);

    let listener = match UnixListener::bind(socket_path) {
        Ok(unix_listener) => unix_listener,
        Err(e) => {
            eprintln!("Fail to bind on socket: {} | Error: {}", socket_path, e);
            return;
        }
    };

    let server_state = Arc::new(Mutex::new(HyprvisorState {
        data: HyprvisorData {
            workspace_info: vec![
                WorkspaceState::Active,
                WorkspaceState::Occupied,
                WorkspaceState::Empty,
                WorkspaceState::Empty,
                WorkspaceState::Empty,
                WorkspaceState::Empty,
                WorkspaceState::Empty,
                WorkspaceState::Empty,
                WorkspaceState::Empty,
                WorkspaceState::Empty,
            ],
            window_title: "Hyprland".to_string(),
            sink_volume: Some(50),
            source_volume: None,
        },
        subscribers: HashMap::new(),
    }));

    println!("Server is listening on {}", socket_path);

    // Handle client connection
    while let Ok((stream, _)) = listener.accept().await {
        let server_state_ref = Arc::clone(&server_state);
        tokio::spawn(handle_new_connection(stream, server_state_ref));
    }
}

async fn handle_new_connection(mut stream: UnixStream, server_state: Arc<Mutex<HyprvisorState>>) {
    let mut buffer = [0; 1024];
    let bytes_received = match stream.try_read(&mut buffer) {
        Ok(message_len) => message_len,
        Err(e) => {
            eprintln!("Failed to read data from client. | Error: {}", e);
            return;
        }
    };

    if bytes_received < 2 {
        eprintln!("Invalid message.");
        return;
    }

    let subscription_info: Result<SubscriptionInfo, serde_json::Error> =
        serde_json::from_slice(&buffer[0..bytes_received].to_vec());

    match subscription_info {
        Ok(info) => {
            let subscription_id = match info.name.as_str() {
                "workspace" => Subscription::Workspace,
                "window" => Subscription::Window,
                "sink_volume" => Subscription::AudioSinkVolume,
                "source_volume" => Subscription::AudioSourceVolume,
                _ => {
                    eprintln!("Invalid subscription");
                    return;
                }
            };

            let mut state = server_state.lock().await;
            state
                .subscribers
                .entry(subscription_id)
                .or_insert(HashMap::new());

            println!(
                "New client with PID {} subscribed to {}",
                info.pid, info.name
            );

            let response_message = "From server with love";
            stream.write_all(response_message.as_bytes()).await.unwrap();

            state
                .subscribers
                .get_mut(&subscription_id)
                .unwrap()
                .insert(info.pid, stream);
        }

        Err(e) => {
            eprintln!("Failed to parse subscription message. | Error: {}", e);
            return;
        }
    }
}
