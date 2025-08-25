// UI Components for Genesis Browser - LEGACY
// These were used by the old pixel-based UI
// Modern egui UI handles responses internally
// TODO: Remove after confirming no usage

#[deprecated = "Use modern egui UI responses instead"]

/// Navigation bar response without egui
#[derive(Default)]
pub struct NavigationBarResponse {
    pub go_back: bool,
    pub go_forward: bool,
    pub reload: bool,
    pub stop: bool,
    pub go_home: bool,
    pub navigate: bool,
    pub new_url: Option<String>,
}

/// Tab bar response without egui
#[derive(Default)]
pub struct TabBarResponse {
    pub switch_to: Option<usize>,
    pub close_tab: Option<usize>,
    pub new_tab: bool,
}

/// Bookmarks panel response without egui
#[derive(Default)]
pub struct BookmarksPanelResponse {
    pub navigate_to: Option<String>,
    pub delete_bookmark: Option<String>,
    pub add_current_page: bool,
}

/// Downloads panel response without egui
#[derive(Default)]
pub struct DownloadsPanelResponse {
    pub open_file: Option<String>,
    pub pause_download: Option<String>,
    pub resume_download: Option<String>,
    pub retry_download: Option<String>,
    pub cancel_download: Option<String>,
    pub clear_all: bool,
}

/// Settings panel response without egui
#[derive(Default)]
pub struct SettingsPanelResponse {
    pub clear_browsing_data: bool,
    pub test_genesis_connection: bool,
}

/// Developer tools panel response without egui
#[derive(Default)]
pub struct DevToolsPanelResponse {
    pub active_tab: usize,
}

// Simple component structures for basic UI
pub struct NavigationBar<'a> {
    pub can_go_back: bool,
    pub can_go_forward: bool,
    pub url: &'a mut String,
    pub is_loading: bool,
    pub is_genesis_domain: bool,
}

pub struct TabBar<'a> {
    pub tabs: &'a [(String, String, bool)], // (id, title, is_genesis)
    pub active_index: usize,
}

pub struct StatusBar<'a> {
    pub status_text: &'a str,
    pub is_loading: bool,
    pub load_progress: f32,
    pub is_secure: bool,
    pub is_private: bool,
    pub fps: f32,
    pub memory_mb: f64,
}

pub struct BookmarksPanel<'a> {
    pub bookmarks: &'a [(String, String, String)], // (id, title, url)
    pub search: &'a mut String,
}

pub struct DownloadsPanel<'a> {
    pub downloads: &'a [(String, String, f32, String)], // (id, filename, progress, status)
}

pub struct SettingsPanel<'a> {
    pub javascript_enabled: &'a mut bool,
    pub images_enabled: &'a mut bool,
    pub webgl_enabled: &'a mut bool,
    pub private_mode: &'a mut bool,
    pub adblock_enabled: &'a mut bool,
    pub genesis_node_url: &'a mut String,
}

pub struct DevToolsPanel {
    pub console_output: Vec<String>,
    pub network_requests: Vec<(String, String, u64)>, // (method, url, size)
}