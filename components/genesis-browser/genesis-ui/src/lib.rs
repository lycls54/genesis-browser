// Genesis UI - User interface components for Genesis Browser

pub mod browser_ui;
pub mod enhanced_browser;
pub mod components;

// Modern UI with egui
#[cfg(feature = "modern-ui")]
pub mod modern_browser;

// Re-export main types
pub use browser_ui::GenesisBrowserUI; // Compatibility wrapper
pub use enhanced_browser::{BrowserUIState, BrowserTab, Bookmark, Download, HistoryEntry};

// Legacy components - deprecated
#[deprecated = "Components are handled internally by modern UI"]
pub use components::{
    NavigationBarResponse, TabBarResponse, BookmarksPanelResponse,
    DownloadsPanelResponse, SettingsPanelResponse, DevToolsPanelResponse,
};

#[cfg(feature = "modern-ui")]
pub use modern_browser::ModernGenesisBrowser;