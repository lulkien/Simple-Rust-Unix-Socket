use std::{env, ops::Add};

use serde::{Deserialize, Serialize};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::UnixStream;

#[derive(Debug, Clone, Deserialize, Serialize)]
struct Workspace {
    id: u32,
    name: String,
    monitor: String,
    #[serde(rename = "monitorID")]
    monitor_id: u32,
    windows: u32,
    hasfullscreen: bool,
    lastwindow: String,
    lastwindowtitle: String,
}

impl Workspace {
    fn new() -> Self {
        Workspace {
            id: 0,
            name: String::new(),
            monitor: String::new(),
            monitor_id: 0,
            windows: 0,
            hasfullscreen: false,
            lastwindow: String::new(),
            lastwindowtitle: String::new(),
        }
    }
}

#[tokio::main]
async fn main() {
    let hyprland_instance_signature = match env::var("HYPRLAND_INSTANCE_SIGNATURE") {
        Ok(var) => var,
        Err(_) => {
            eprintln!("HYPRLAND_INSTANCE_SIGNATURE not set! (is hyprland running?)");
            return;
        }
    };

    let hypr_socket = "/tmp/hypr/"
        .to_string()
        .add(hyprland_instance_signature.as_str())
        .add("/.socket.sock");

    let mut stream = match UnixStream::connect(&hypr_socket).await {
        Ok(stream) => {
            println!("Connected to socket {hypr_socket}");
            stream
        }
        Err(e) => {
            eprintln!("Failed to connect to socket {hypr_socket}, Error: {e}");
            return;
        }
    };

    if let Err(e) = stream.write_all(b"workspaces").await {
        eprintln!("Failed to write to socket: {}", e);
        return;
    }

    let mut response = String::new();
    let mut buffer: [u8; 8192] = [0; 8192];
    loop {
        match stream.read(&mut buffer).await {
            Ok(size) if size > 0 => {
                let chunk = String::from_utf8_lossy(&buffer[..size]);
                response.push_str(&chunk);
            }
            Ok(_) | Err(_) => break, // Break on end of response or error
        }
    }

    println!("{response}");

    let workspaces: Vec<Workspace> = parse_reply(&response);
    let json_result = serde_json::to_string_pretty(&workspaces).unwrap();

    println!("{}", json_result);
}

fn parse_reply(reply: &str) -> Vec<Workspace> {
    let mut workspaces = Vec::new();
    let mut current_workspace = Workspace::new();

    for line in reply.lines() {
        if line.starts_with("workspace ID") {
            if current_workspace.id != 0 {
                workspaces.push(current_workspace.clone());
            }

            let parts: Vec<&str> = line.split_whitespace().collect();
            current_workspace.id = parts[2].parse().unwrap();
            current_workspace.name = parts[4].trim_end_matches('(').to_string();
            continue;
        }

        if line.starts_with("monitor") {
            let parts: Vec<&str> = line.split_whitespace().collect();
            current_workspace.monitor = parts[3].to_string();
            current_workspace.monitor_id = parts[5].parse().unwrap();
            continue;
        }

        if line.starts_with("windows") {
            let parts: Vec<&str> = line.split_whitespace().collect();
            current_workspace.windows = parts[1].parse().unwrap();
            continue;
        }

        if line.starts_with("hasfullscreen") {
            let parts: Vec<&str> = line.split_whitespace().collect();
            current_workspace.hasfullscreen = parts[1] == "1";
            continue;
        }

        if line.starts_with("lastwindow") {
            let parts: Vec<&str> = line.split_whitespace().collect();
            current_workspace.lastwindow = parts[1].to_string();
            continue;
        }

        if line.starts_with("lastwindowtitle") {
            current_workspace.lastwindowtitle =
                line.trim_start_matches("lastwindowtitle: ").to_string();
        }
    }

    if current_workspace.id != 0 {
        workspaces.push(current_workspace);
    }

    workspaces
}
