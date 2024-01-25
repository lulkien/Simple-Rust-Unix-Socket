use std::collections::HashMap;
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
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
    Invalid,
}

struct HyprvisorState {
    data: HyprvisorData,
    subscribers: HashMap<Subscription, Vec<UnixStream>>,
}

#[tokio::main]
async fn main() {
    let socket_path = "/tmp/hyprvisor.sock";

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
    let mut buffer = [0; 1];
    if let Ok(_) = stream.read_exact(&mut buffer).await {
        let subscription = match buffer[0] {
            1 => Subscription::Workspace,
            // 2 => Subscription::Window,
            // 3 => Subscription::AudioSinkVolume,
            // 4 => Subscription::AudioSourceVolume,
            _ => {
                eprint!("Invalid subscription type...");
                return;
            }
        };

        let mut state = server_state.lock().await;
        state.subscribers.entry(subscription).or_insert(Vec::new());

        println!("New subscriber with ID: {}", buffer[0]);

        let response_message = "I got u, bae";
        stream.write_all(response_message.as_bytes()).await.unwrap();

        state
            .subscribers
            .get_mut(&subscription)
            .unwrap()
            .push(stream);
    }
}
