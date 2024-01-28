use std::collections::HashMap;
use std::fs;
use std::sync::Arc;
use std::time::Duration;
use tokio::io::AsyncWriteExt;
use tokio::net::{UnixListener, UnixStream};
use tokio::sync::Mutex;
use tokio::time::sleep;

use crate::hyprvisor::HyprvisorState;
use crate::protocols::{SubscriptionID, SubscriptionInfo};

pub struct Server {
    socket: String,
    is_ready: Option<bool>,
    state: Arc<Mutex<HyprvisorState>>,
}

impl Server {
    pub async fn new(socket: String) -> Self {
        Server {
            socket,
            is_ready: None,
            state: Arc::new(Mutex::new(HyprvisorState::new())),
        }
    }

    pub async fn prepare(&mut self) {
        if fs::metadata(&self.socket).is_err() {
            println!("No running server binded on socket {}", self.socket);
            self.is_ready = Some(true);
            return;
        };

        match UnixStream::connect(&self.socket).await {
            Ok(_) => {
                eprintln!("There is a running server bind on {}", self.socket);
                self.is_ready = Some(false);
                return;
            }
            _ => match fs::remove_file(&self.socket) {
                Ok(_) => {
                    println!("Remove old socket {}", self.socket);
                    self.is_ready = Some(true);
                    return;
                }
                Err(e) => {
                    println!(
                        "Failed to remove old socket path {} | Error: {}",
                        self.socket, e
                    );
                    self.is_ready = Some(false);
                    return;
                }
            },
        }
    }

    pub async fn start(&mut self) {
        if self.is_ready.is_none() {
            eprintln!("Error: Prepare server before run!");
            return;
        }

        if Some(false) == self.is_ready {
            eprintln!("Error: Cannot prepare server!");
            return;
        }

        let listener = match UnixListener::bind(&self.socket) {
            Ok(unix_listener) => unix_listener,
            Err(e) => {
                eprintln!("Fail to bind on socket: {} | Error: {}", self.socket, e);
                return;
            }
        };

        println!(
            "Hyprvisor daemon is listening for connection on {}",
            self.socket
        );

        let broadcast_state_ref = Arc::clone(&self.state);
        tokio::spawn(broadcast_data(broadcast_state_ref));

        // Main loop
        while let Ok((stream, _)) = listener.accept().await {
            let state_ref = Arc::clone(&self.state);
            tokio::spawn(handle_new_connection(stream, state_ref));
        }
    }
}

async fn handle_new_connection(mut stream: UnixStream, state: Arc<Mutex<HyprvisorState>>) {
    // Handle new connection
    let mut buffer: [u8; 1024] = [0; 1024];
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
                "workspace" => SubscriptionID::WORKSPACE,
                "window" => SubscriptionID::WINDOW,
                "sink_volume" => SubscriptionID::SINKVOLUME,
                "source_volume" => SubscriptionID::SOURCEVOLUME,
                _ => {
                    eprintln!("Invalid subscription");
                    return;
                }
            };

            let mut state = state.lock().await;
            state
                .subscribers
                .entry(subscription_id)
                .or_insert(HashMap::new());

            println!(
                "New client with PID {} subscribed to {}",
                info.pid, info.name
            );

            let response_message = "From server with love".to_string();
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

async fn broadcast_data(server_state: Arc<Mutex<HyprvisorState>>) {
    loop {
        println!("Send message to client");
        {
            // Lock server state
            let mut state = server_state.lock().await;

            for (_, subscribers) in state.subscribers.iter_mut() {
                let mut disconnected_pid: Vec<u32> = Vec::new();
                for (pid, stream) in subscribers.iter_mut() {
                    let msg = "Test connection".to_string();
                    match stream.write_all(msg.as_bytes()).await {
                        Ok(_) => {
                            println!("Client {} is alive.", pid);
                        }
                        Err(e) => {
                            println!("Client {} is no longer alive. Error: {}", pid, e);
                            disconnected_pid.push(pid.clone());
                        }
                    }
                }

                // Remove disconnected_pid
                for pid in disconnected_pid {
                    subscribers.remove(&pid);
                }
            }
            // Release server state
        }

        sleep(Duration::from_secs(2)).await;
    }
}
