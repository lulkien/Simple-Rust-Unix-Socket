#[allow(dead_code)]
#[derive(Debug)]
pub enum WorkspaceState {
    Active = 0,
    Occupied = 1,
    Empty = 2,
}

#[derive(Debug)]
pub struct HyprvisorData {
    pub workspace_info: Vec<WorkspaceState>,
    pub window_title: String,
    pub sink_volume: Option<u32>,
    pub source_volume: Option<u32>,
}

impl HyprvisorData {
    pub fn new() -> Self {
        HyprvisorData {
            workspace_info: vec![
                WorkspaceState::Active,
                WorkspaceState::Empty,
                WorkspaceState::Empty,
                WorkspaceState::Empty,
                WorkspaceState::Empty,
                WorkspaceState::Empty,
                WorkspaceState::Empty,
                WorkspaceState::Empty,
                WorkspaceState::Empty,
                WorkspaceState::Empty,
            ],
            window_title: "".to_string(),
            sink_volume: None,
            source_volume: None,
        }
    }
}
