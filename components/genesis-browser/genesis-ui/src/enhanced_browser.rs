// Enhanced Browser UI with full Servo integration
// This module provides the complete browser interface with tabs, navigation, and Servo WebView

use std::cell::{Cell, RefCell};
use std::collections::VecDeque;

use tracing::info;

// Tab structure for multi-tab browsing
#[derive(Clone, Debug)]
pub struct BrowserTab {
    pub id: String,
    pub title: String,
    pub url: String,
    pub favicon: Option<String>,
    pub is_loading: bool,
    pub can_go_back: bool,
    pub can_go_forward: bool,
    pub is_genesis_domain: bool,
    pub load_progress: f32,
}

impl BrowserTab {
    pub fn new(url: &str) -> Self {
        let id = uuid::Uuid::new_v4().to_string();
        Self {
            id,
            title: "New Tab".to_string(),
            url: url.to_string(),
            favicon: None,
            is_loading: false,
            can_go_back: false,
            can_go_forward: false,
            is_genesis_domain: Self::check_genesis_domain(url),
            load_progress: 0.0,
        }
    }
    
    fn check_genesis_domain(url: &str) -> bool {
        let genesis_tlds = [".genesis", ".free", ".web", ".defi", ".dao", "genesis://"];
        genesis_tlds.iter().any(|tld| url.contains(tld))
    }
}

/// Browser UI State Management
pub struct BrowserUIState {
    // Tab management
    pub tabs: RefCell<Vec<BrowserTab>>,
    pub active_tab_index: Cell<usize>,
    
    // Navigation
    pub url_input: RefCell<String>,
    pub search_input: RefCell<String>,
    pub navigation_history: RefCell<VecDeque<String>>,
    
    // UI panels visibility
    pub show_dev_tools: Cell<bool>,
    pub show_downloads: Cell<bool>,
    pub show_bookmarks: Cell<bool>,
    pub show_history: Cell<bool>,
    pub show_settings: Cell<bool>,
    pub show_sidebar: Cell<bool>,
    
    // Browser features
    pub private_mode: Cell<bool>,
    pub javascript_enabled: Cell<bool>,
    pub images_enabled: Cell<bool>,
    pub webgl_enabled: Cell<bool>,
    pub adblock_enabled: Cell<bool>,
    
    // Genesis features
    pub genesis_connected: Cell<bool>,
    pub genesis_node_status: RefCell<String>,
    
    // Collections
    pub bookmarks: RefCell<Vec<Bookmark>>,
    pub downloads: RefCell<Vec<Download>>,
    pub history: RefCell<Vec<HistoryEntry>>,
    pub passwords: RefCell<Vec<SavedPassword>>,
}

#[derive(Clone, Debug)]
pub struct Bookmark {
    pub id: String,
    pub title: String,
    pub url: String,
    pub folder: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Clone, Debug)]
pub struct Download {
    pub id: String,
    pub filename: String,
    pub url: String,
    pub size: u64,
    pub downloaded: u64,
    pub status: DownloadStatus,
}

#[derive(Clone, Debug)]
pub enum DownloadStatus {
    Pending,
    InProgress(f32),
    Completed,
    Failed(String),
    Cancelled,
}

#[derive(Clone, Debug)]
pub struct HistoryEntry {
    pub url: String,
    pub title: String,
    pub visit_count: u32,
    pub last_visit: chrono::DateTime<chrono::Utc>,
}

#[derive(Clone, Debug)]
pub struct SavedPassword {
    pub domain: String,
    pub username: String,
    pub encrypted_password: Vec<u8>,
}

impl Default for BrowserUIState {
    fn default() -> Self {
        // Create default tabs
        let welcome_tab = BrowserTab::new("genesis://welcome");
        
        Self {
            tabs: RefCell::new(vec![welcome_tab]),
            active_tab_index: Cell::new(0),
            url_input: RefCell::new("genesis://welcome".to_string()),
            search_input: RefCell::new(String::new()),
            navigation_history: RefCell::new(VecDeque::with_capacity(100)),
            show_dev_tools: Cell::new(false),
            show_downloads: Cell::new(false),
            show_bookmarks: Cell::new(false),
            show_history: Cell::new(false),
            show_settings: Cell::new(false),
            show_sidebar: Cell::new(false),
            private_mode: Cell::new(false),
            javascript_enabled: Cell::new(true),
            images_enabled: Cell::new(true),
            webgl_enabled: Cell::new(true),
            adblock_enabled: Cell::new(false),
            genesis_connected: Cell::new(false),
            genesis_node_status: RefCell::new("Connecting...".to_string()),
            bookmarks: RefCell::new(Self::default_bookmarks()),
            downloads: RefCell::new(Vec::new()),
            history: RefCell::new(Vec::new()),
            passwords: RefCell::new(Vec::new()),
        }
    }
}

impl BrowserUIState {
    fn default_bookmarks() -> Vec<Bookmark> {
        vec![
            Bookmark {
                id: uuid::Uuid::new_v4().to_string(),
                title: "Genesis Home".to_string(),
                url: "genesis://home".to_string(),
                folder: Some("Genesis".to_string()),
                created_at: chrono::Utc::now(),
            },
            Bookmark {
                id: uuid::Uuid::new_v4().to_string(),
                title: "Freedom Web".to_string(),
                url: "freedom.free".to_string(),
                folder: Some("Genesis".to_string()),
                created_at: chrono::Utc::now(),
            },
            Bookmark {
                id: uuid::Uuid::new_v4().to_string(),
                title: "DeFi Exchange".to_string(),
                url: "exchange.defi".to_string(),
                folder: Some("DeFi".to_string()),
                created_at: chrono::Utc::now(),
            },
            Bookmark {
                id: uuid::Uuid::new_v4().to_string(),
                title: "DAO Governance".to_string(),
                url: "governance.dao".to_string(),
                folder: Some("DAO".to_string()),
                created_at: chrono::Utc::now(),
            },
        ]
    }
    
    /// Create a new tab
    pub fn create_tab(&self, url: &str) -> String {
        let new_tab = BrowserTab::new(url);
        let tab_id = new_tab.id.clone();
        
        self.tabs.borrow_mut().push(new_tab);
        let new_index = self.tabs.borrow().len() - 1;
        self.active_tab_index.set(new_index);
        
        info!("📑 Created new tab: {} -> {}", tab_id, url);
        tab_id
    }
    
    /// Close a tab
    pub fn close_tab(&self, index: usize) -> bool {
        let mut tabs = self.tabs.borrow_mut();
        
        if tabs.len() > 1 && index < tabs.len() {
            tabs.remove(index);
            
            // Adjust active tab if needed
            let current = self.active_tab_index.get();
            if current >= tabs.len() {
                self.active_tab_index.set(tabs.len() - 1);
            } else if current > index {
                self.active_tab_index.set(current - 1);
            }
            
            info!("📑 Closed tab at index {}", index);
            true
        } else {
            false
        }
    }
    
    /// Switch to a specific tab
    pub fn switch_to_tab(&self, index: usize) {
        let tabs = self.tabs.borrow();
        if index < tabs.len() {
            self.active_tab_index.set(index);
            if let Some(tab) = tabs.get(index) {
                *self.url_input.borrow_mut() = tab.url.clone();
                info!("📑 Switched to tab {}: {}", index, tab.title);
            }
        }
    }
    
    /// Get current active tab
    pub fn get_active_tab(&self) -> Option<BrowserTab> {
        let tabs = self.tabs.borrow();
        let index = self.active_tab_index.get();
        tabs.get(index).cloned()
    }
    
    /// Update tab info
    pub fn update_tab(&self, index: usize, title: Option<String>, url: Option<String>, is_loading: Option<bool>) {
        let mut tabs = self.tabs.borrow_mut();
        if let Some(tab) = tabs.get_mut(index) {
            if let Some(title) = title {
                tab.title = title;
            }
            if let Some(url) = url {
                tab.url = url.clone();
                tab.is_genesis_domain = BrowserTab::check_genesis_domain(&url);
            }
            if let Some(loading) = is_loading {
                tab.is_loading = loading;
                if !loading {
                    tab.load_progress = 1.0;
                }
            }
        }
    }
    
    /// Add bookmark
    pub fn add_bookmark(&self, title: String, url: String, folder: Option<String>) {
        let bookmark = Bookmark {
            id: uuid::Uuid::new_v4().to_string(),
            title,
            url: url.clone(),
            folder,
            created_at: chrono::Utc::now(),
        };
        
        self.bookmarks.borrow_mut().push(bookmark);
        info!("⭐ Added bookmark: {}", url);
    }
    
    /// Remove bookmark
    pub fn remove_bookmark(&self, id: &str) {
        self.bookmarks.borrow_mut().retain(|b| b.id != id);
        info!("⭐ Removed bookmark: {}", id);
    }
    
    /// Add to history
    pub fn add_to_history(&self, url: String, title: String) {
        let mut history = self.history.borrow_mut();
        
        // Check if URL already exists
        if let Some(entry) = history.iter_mut().find(|e| e.url == url) {
            entry.visit_count += 1;
            entry.last_visit = chrono::Utc::now();
        } else {
            history.push(HistoryEntry {
                url: url.clone(),
                title,
                visit_count: 1,
                last_visit: chrono::Utc::now(),
            });
            
            // Limit history size
            if history.len() > 1000 {
                history.drain(0..100);
            }
        }
        
        // Add to navigation history
        self.navigation_history.borrow_mut().push_back(url);
        if self.navigation_history.borrow().len() > 100 {
            self.navigation_history.borrow_mut().pop_front();
        }
    }
    
    /// Clear browsing data
    pub fn clear_browsing_data(&self, clear_history: bool, clear_downloads: bool, clear_passwords: bool) {
        if clear_history {
            self.history.borrow_mut().clear();
            self.navigation_history.borrow_mut().clear();
            info!("🧹 Cleared browsing history");
        }
        
        if clear_downloads {
            self.downloads.borrow_mut().clear();
            info!("🧹 Cleared downloads");
        }
        
        if clear_passwords {
            self.passwords.borrow_mut().clear();
            info!("🧹 Cleared saved passwords");
        }
    }
    
    /// Add download
    pub fn add_download(&self, filename: String, url: String, size: u64) -> String {
        let download = Download {
            id: uuid::Uuid::new_v4().to_string(),
            filename,
            url,
            size,
            downloaded: 0,
            status: DownloadStatus::Pending,
        };
        
        let id = download.id.clone();
        self.downloads.borrow_mut().push(download);
        info!("⬇ Started download: {}", id);
        id
    }
    
    /// Update download progress
    pub fn update_download(&self, id: &str, downloaded: u64, status: DownloadStatus) {
        let mut downloads = self.downloads.borrow_mut();
        if let Some(download) = downloads.iter_mut().find(|d| d.id == id) {
            download.downloaded = downloaded;
            download.status = status;
        }
    }
    
    /// Toggle UI panel
    pub fn toggle_panel(&self, panel: &str) {
        match panel {
            "dev_tools" => self.show_dev_tools.set(!self.show_dev_tools.get()),
            "downloads" => self.show_downloads.set(!self.show_downloads.get()),
            "bookmarks" => self.show_bookmarks.set(!self.show_bookmarks.get()),
            "history" => self.show_history.set(!self.show_history.get()),
            "settings" => self.show_settings.set(!self.show_settings.get()),
            "sidebar" => self.show_sidebar.set(!self.show_sidebar.get()),
            _ => {}
        }
    }
}