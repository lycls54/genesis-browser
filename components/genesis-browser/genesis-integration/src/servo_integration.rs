use std::sync::{Arc, Mutex};
use tokio::sync::RwLock;
use tracing::{info, error, debug};
use anyhow::Result;
use url::Url;

use crate::servo_engine::{ServoEngine, ServoConfig};
use crate::webview::{WebViewManager, WebViewConfig};
use base::id::WebViewId;
use genesis_dns::GenesisDnsResolver;

/// Servo-based browser engine integration for Genesis Browser
pub struct GenesisBrowserEngine {
    /// Servo engine instance
    servo_engine: Arc<Mutex<ServoEngine>>,
    /// WebView manager for tabs
    webview_manager: Arc<Mutex<WebViewManager>>,
    /// Genesis DNS resolver
    dns_resolver: Arc<RwLock<GenesisDnsResolver>>,
    /// Configuration
    config: BrowserConfig,
    /// Active webview ID
    active_webview: Option<WebViewId>,
}

/// Browser configuration
#[derive(Debug, Clone)]
pub struct BrowserConfig {
    pub enable_genesis_dns: bool,
    pub enable_traditional_fallback: bool,
    pub genesis_node_url: String,
    pub user_agent: String,
    pub enable_javascript: bool,
    pub enable_webgl: bool,
}

impl Default for BrowserConfig {
    fn default() -> Self {
        Self {
            enable_genesis_dns: true,
            enable_traditional_fallback: true,
            genesis_node_url: "http://localhost:3000".to_string(),
            user_agent: "Genesis Browser/1.0 (Servo; Decentralized Web)".to_string(),
            enable_javascript: true,
            enable_webgl: true,
        }
    }
}

impl GenesisBrowserEngine {
    /// Create new Genesis Browser Engine with Servo
    pub async fn new(config: BrowserConfig) -> Result<Self> {
        info!("🚀 Initializing Genesis Browser Engine with Servo");

        // Create Servo configuration
        let servo_config = ServoConfig {
            window_width: 1200,
            window_height: 800,
            enable_webgl: config.enable_webgl,
            enable_webrender: true,
            enable_javascript: config.enable_javascript,
            user_agent: config.user_agent.clone(),
            genesis_node_url: config.genesis_node_url.clone(),
            enable_dev_tools: false,
            multiprocess: false,
            ..Default::default()
        };

        // Initialize Servo engine
        let servo_engine = Arc::new(Mutex::new(
            ServoEngine::new(servo_config.title.clone())?
        ));

        // Initialize WebView manager
        let webview_config = WebViewConfig {
            javascript_enabled: config.enable_javascript,
            images_enabled: true,
            webgl_enabled: config.enable_webgl,
            user_agent: config.user_agent.clone(),
            developer_tools: false,
            private_mode: false,
        };
        
        let webview_manager = Arc::new(Mutex::new(
            WebViewManager::new(servo_engine.clone(), webview_config).await?
        ));

        // Initialize DNS resolver
        let dns_resolver = Arc::new(RwLock::new(
            GenesisDnsResolver::new(
                config.genesis_node_url.clone(),
                config.enable_traditional_fallback,
            )
        ));

        let engine = Self {
            servo_engine,
            webview_manager,
            dns_resolver,
            config,
            active_webview: None,
        };

        info!("✅ Genesis Browser Engine with Servo initialized successfully");
        Ok(engine)
    }

    /// Start the browser engine
    pub async fn start(&mut self) -> Result<()> {
        info!("🌟 Starting Genesis Browser Engine with Servo");

        // Start Servo engine
        {
            let mut engine = self.servo_engine.lock().unwrap();
            engine.start().await?;
        }

        // Create initial webview
        let webview_id = {
            let mut manager = self.webview_manager.lock().unwrap();
            manager.create_webview().await?
        };
        
        self.active_webview = Some(webview_id);

        info!("🎯 Genesis Browser Engine with Servo started successfully");
        Ok(())
    }

    /// Navigate to URL in active webview
    pub async fn navigate(&mut self, url: &str) -> Result<()> {
        info!("🔍 Navigating to: {}", url);

        // Navigate in active webview
        let mut manager = self.webview_manager.lock().unwrap();
        manager.navigate(url).await?;

        info!("✅ Navigation initiated: {}", url);
        Ok(())
    }

    /// Create new tab
    pub async fn new_tab(&mut self) -> Result<WebViewId> {
        let mut manager = self.webview_manager.lock().unwrap();
        let id = manager.create_webview().await?;
        self.active_webview = Some(id);
        info!("📑 Created new tab with ID: {:?}", id);
        Ok(id)
    }

    /// Close tab
    pub async fn close_tab(&mut self, id: WebViewId) -> Result<()> {
        let mut manager = self.webview_manager.lock().unwrap();
        manager.close_webview(id)?;
        
        if self.active_webview == Some(id) {
            self.active_webview = manager.webview_ids().first().cloned();
        }
        
        info!("📑 Closed tab with ID: {:?}", id);
        Ok(())
    }

    /// Switch to tab
    pub async fn switch_tab(&mut self, id: WebViewId) -> Result<()> {
        let mut manager = self.webview_manager.lock().unwrap();
        manager.set_active(id)?;
        self.active_webview = Some(id);
        info!("📑 Switched to tab with ID: {:?}", id);
        Ok(())
    }

    /// Go back in history
    pub async fn go_back(&mut self) -> Result<()> {
        let mut manager = self.webview_manager.lock().unwrap();
        manager.go_back().await?;
        Ok(())
    }

    /// Go forward in history
    pub async fn go_forward(&mut self) -> Result<()> {
        let mut manager = self.webview_manager.lock().unwrap();
        manager.go_forward().await?;
        Ok(())
    }

    /// Reload current page
    pub async fn reload(&mut self) -> Result<()> {
        let mut manager = self.webview_manager.lock().unwrap();
        manager.reload().await?;
        Ok(())
    }

    /// Run the browser event loop
    pub async fn run(&mut self) -> Result<()> {
        info!("🏃 Starting browser event loop");
        
        let mut engine = self.servo_engine.lock().unwrap();
        engine.run().await?;
        
        Ok(())
    }

    /// Get current browser status
    pub async fn get_status(&self) -> BrowserStatus {
        let engine = self.servo_engine.lock().unwrap();
        let mut manager = self.webview_manager.lock().unwrap();
        
        let current_url = if let Some(webview) = manager.active_webview() {
            webview.current_url().map(|u| u.to_string())
        } else {
            None
        };

        BrowserStatus {
            is_running: engine.is_running(),
            current_url,
            is_loading: false,
            window_size: (1200, 800),
        }
    }

    /// Stop the browser engine
    pub async fn stop(&mut self) -> Result<()> {
        info!("🛑 Stopping Genesis Browser Engine");

        let mut engine = self.servo_engine.lock().unwrap();
        engine.stop().await?;

        info!("✅ Genesis Browser Engine stopped");
        Ok(())
    }
}

/// Browser status information
#[derive(Debug)]
pub struct BrowserStatus {
    pub is_running: bool,
    pub current_url: Option<String>,
    pub is_loading: bool,
    pub window_size: (u32, u32),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_browser_engine_creation() {
        let config = BrowserConfig::default();
        let engine = GenesisBrowserEngine::new(config).await;
        assert!(engine.is_ok());
    }
}