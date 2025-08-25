// Legacy pixel-based UI - DEPRECATED
// This file contains the old minifb-based pixel rendering UI
// Use modern_browser.rs instead for the new egui-based interface

// Moved to browser_ui_legacy.rs to keep as reference
// TODO: Remove this file completely after migration is confirmed stable

#[deprecated = "Use ModernGenesisBrowser instead"]
pub struct GenesisBrowserUI;

impl GenesisBrowserUI {
    #[deprecated = "Use ModernGenesisBrowser::run() instead"]
    pub fn new(_genesis_node_url: String) -> Result<Self, Box<dyn std::error::Error>> {
        Err("Legacy UI deprecated. Use ModernGenesisBrowser instead.".into())
    }
    
    #[deprecated = "Use ModernGenesisBrowser::run() instead"]  
    pub async fn run(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        Err("Legacy UI deprecated. Use ModernGenesisBrowser::run() instead.".into())
    }
}