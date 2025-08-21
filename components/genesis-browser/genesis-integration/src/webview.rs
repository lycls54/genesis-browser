use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use anyhow::{Result, Context};
use tracing::{info, error, debug, warn};
use url::Url;
use tokio::sync::{mpsc, oneshot};

#[cfg(feature = "servo-integration")]
use {
    // Servo WebView API
    servo::{WebViewId, Servo},
    base::id::BrowsingContextId,
    compositing_traits::CompositorMsg,
    constellation_traits::{LoadData, LoadOrigin},
    servo_url::ServoUrl,
};

use crate::servo_engine::{ServoEngine, ServoConfig};
use genesis_dns::GenesisDnsResolver;

/// Genesis Browser WebView implementation
pub struct GenesisWebView {
    /// Unique ID for this webview
    id: WebViewId,
    
    /// Servo engine instance
    engine: Arc<Mutex<ServoEngine>>,
    
    /// Current URL
    current_url: Option<Url>,
    
    /// Page title
    title: String,
    
    /// Loading state
    is_loading: bool,
    
    /// History
    history: Vec<Url>,
    history_index: usize,
    
    /// JavaScript enabled
    javascript_enabled: bool,
    
    /// User agent
    user_agent: String,
    
    /// Genesis DNS resolver
    dns_resolver: Arc<tokio::sync::RwLock<GenesisDnsResolver>>,
    
    /// Event channel
    event_sender: mpsc::Sender<WebViewEvent>,
    event_receiver: mpsc::Receiver<WebViewEvent>,
}

/// WebView events
#[derive(Debug, Clone)]
pub enum WebViewEvent {
    LoadStarted(Url),
    LoadFinished(Url),
    LoadError(String),
    TitleChanged(String),
    NavigationRequested(Url),
    NewWindowRequested(Url),
    CloseRequested,
    ProgressUpdate(f32),
    SecurityStateChanged(SecurityState),
    ConsoleMessage(ConsoleMessage),
}

/// Security state of the page
#[derive(Debug, Clone)]
pub enum SecurityState {
    Secure,       // HTTPS or Genesis blockchain verified
    Insecure,     // HTTP
    Broken,       // Mixed content
    Genesis,      // Genesis blockchain domain
}

/// Console message from the page
#[derive(Debug, Clone)]
pub struct ConsoleMessage {
    pub level: ConsoleLevel,
    pub message: String,
    pub source: String,
    pub line: u32,
}

#[derive(Debug, Clone)]
pub enum ConsoleLevel {
    Log,
    Info,
    Warn,
    Error,
}

/// WebView manager for multiple tabs
pub struct WebViewManager {
    /// All webviews
    webviews: HashMap<WebViewId, GenesisWebView>,
    
    /// Active webview
    active_id: Option<WebViewId>,
    
    /// Servo engine (shared)
    engine: Arc<Mutex<ServoEngine>>,
    
    /// Next webview ID
    next_id: u32,
    
    /// Configuration
    config: WebViewConfig,
}

/// WebView configuration
#[derive(Debug, Clone)]
pub struct WebViewConfig {
    pub javascript_enabled: bool,
    pub images_enabled: bool,
    pub webgl_enabled: bool,
    pub user_agent: String,
    pub developer_tools: bool,
    pub private_mode: bool,
}

impl Default for WebViewConfig {
    fn default() -> Self {
        Self {
            javascript_enabled: true,
            images_enabled: true,
            webgl_enabled: true,
            user_agent: "Genesis Browser/1.0 (Servo; Decentralized Web)".to_string(),
            developer_tools: false,
            private_mode: false,
        }
    }
}

impl GenesisWebView {
    /// Create new webview
    pub async fn new(
        id: WebViewId,
        engine: Arc<Mutex<ServoEngine>>,
        dns_resolver: Arc<tokio::sync::RwLock<GenesisDnsResolver>>,
    ) -> Result<Self> {
        let (event_sender, event_receiver) = mpsc::channel(100);
        
        Ok(Self {
            id,
            engine,
            current_url: None,
            title: String::from("New Tab"),
            is_loading: false,
            history: Vec::new(),
            history_index: 0,
            javascript_enabled: true,
            user_agent: "Genesis Browser/1.0".to_string(),
            dns_resolver,
            event_sender,
            event_receiver,
        })
    }
    
    /// Navigate to URL
    pub async fn navigate(&mut self, url: &str) -> Result<()> {
        info!("🔍 WebView {} navigating to: {}", self.id.0, url);
        
        // Parse URL
        let parsed_url = self.parse_and_resolve_url(url).await?;
        
        // Send load started event
        self.event_sender.send(WebViewEvent::LoadStarted(parsed_url.clone())).await?;
        self.is_loading = true;
        
        // Navigate using Servo engine
        #[cfg(feature = "servo-integration")]
        {
            let mut engine = self.engine.lock().unwrap();
            engine.navigate(&parsed_url.to_string()).await?;
        }
        
        // Update state
        self.current_url = Some(parsed_url.clone());
        self.add_to_history(parsed_url.clone());
        
        // Send load finished event (in real implementation, this would be async)
        self.event_sender.send(WebViewEvent::LoadFinished(parsed_url)).await?;
        self.is_loading = false;
        
        Ok(())
    }
    
    /// Parse and resolve URL (including Genesis domains)
    async fn parse_and_resolve_url(&self, url: &str) -> Result<Url> {
        let url_str = if !url.starts_with("http://") && !url.starts_with("https://") {
            format!("http://{}", url)
        } else {
            url.to_string()
        };
        
        let mut parsed_url = Url::parse(&url_str)?;
        
        // Check if it's a Genesis domain
        if let Some(host) = parsed_url.host_str() {
            if self.is_genesis_domain(host) {
                parsed_url = self.resolve_genesis_domain(parsed_url).await?;
            }
        }
        
        Ok(parsed_url)
    }
    
    /// Check if domain is Genesis
    fn is_genesis_domain(&self, host: &str) -> bool {
        let genesis_tlds = [".genesis", ".free", ".web", ".defi", ".dao"];
        genesis_tlds.iter().any(|tld| host.ends_with(tld))
    }
    
    /// Resolve Genesis domain
    async fn resolve_genesis_domain(&self, mut url: Url) -> Result<Url> {
        if let Some(host) = url.host_str() {
            info!("🌐 Resolving Genesis domain: {}", host);
            
            let mut resolver = self.dns_resolver.write().await;
            match resolver.resolve(host).await {
                Ok(dns_result) => {
                    // Update security state for Genesis domain
                    self.event_sender.send(WebViewEvent::SecurityStateChanged(
                        SecurityState::Genesis
                    )).await?;
                    
                    if let Some(ip) = dns_result.ip_address {
                        info!("✅ Resolved to IP: {}", ip);
                        url.set_host(Some(&ip.to_string()))?;
                    } else if let Some(content_hash) = dns_result.content_hash {
                        info!("📦 Resolved to IPFS: {}", content_hash);
                        url = Url::parse(&format!("https://ipfs.io/ipfs/{}", content_hash))?;
                    }
                },
                Err(e) => {
                    warn!("⚠️ Failed to resolve Genesis domain: {:?}", e);
                    self.event_sender.send(WebViewEvent::LoadError(
                        format!("Failed to resolve Genesis domain: {}", e)
                    )).await?;
                }
            }
        }
        
        Ok(url)
    }
    
    /// Go back in history
    pub async fn go_back(&mut self) -> Result<()> {
        if self.can_go_back() {
            self.history_index -= 1;
            let url = self.history[self.history_index].clone();
            self.navigate(&url.to_string()).await?;
        }
        Ok(())
    }
    
    /// Go forward in history
    pub async fn go_forward(&mut self) -> Result<()> {
        if self.can_go_forward() {
            self.history_index += 1;
            let url = self.history[self.history_index].clone();
            self.navigate(&url.to_string()).await?;
        }
        Ok(())
    }
    
    /// Reload current page
    pub async fn reload(&mut self) -> Result<()> {
        if let Some(url) = &self.current_url {
            let url_str = url.to_string();
            self.navigate(&url_str).await?;
        }
        Ok(())
    }
    
    /// Stop loading
    pub fn stop(&mut self) {
        self.is_loading = false;
        // Send stop message to Servo
    }
    
    /// Execute JavaScript
    #[cfg(feature = "servo-integration")]
    pub async fn execute_script(&mut self, script: &str) -> Result<String> {
        if !self.javascript_enabled {
            return Err(anyhow::anyhow!("JavaScript is disabled"));
        }
        
        debug!("📜 Executing JavaScript in WebView {}", self.id.0);
        
        // Execute script via Servo
        // This would involve sending a message to the script thread
        
        Ok(String::new())
    }
    
    /// Set user agent
    pub fn set_user_agent(&mut self, user_agent: String) {
        self.user_agent = user_agent;
    }
    
    /// Enable/disable JavaScript
    pub fn set_javascript_enabled(&mut self, enabled: bool) {
        self.javascript_enabled = enabled;
    }
    
    /// Get current URL
    pub fn current_url(&self) -> Option<&Url> {
        self.current_url.as_ref()
    }
    
    /// Get title
    pub fn title(&self) -> &str {
        &self.title
    }
    
    /// Check if can go back
    pub fn can_go_back(&self) -> bool {
        self.history_index > 0
    }
    
    /// Check if can go forward
    pub fn can_go_forward(&self) -> bool {
        self.history_index < self.history.len() - 1
    }
    
    /// Add URL to history
    fn add_to_history(&mut self, url: Url) {
        // Remove forward history if we're not at the end
        if self.history_index < self.history.len() - 1 {
            self.history.truncate(self.history_index + 1);
        }
        
        self.history.push(url);
        self.history_index = self.history.len() - 1;
    }
    
    /// Handle page title change
    pub async fn on_title_changed(&mut self, title: String) -> Result<()> {
        self.title = title.clone();
        self.event_sender.send(WebViewEvent::TitleChanged(title)).await?;
        Ok(())
    }
    
    /// Handle console message
    pub async fn on_console_message(&mut self, message: ConsoleMessage) -> Result<()> {
        self.event_sender.send(WebViewEvent::ConsoleMessage(message)).await?;
        Ok(())
    }
    
    /// Get next event
    pub async fn next_event(&mut self) -> Option<WebViewEvent> {
        self.event_receiver.recv().await
    }
}

impl WebViewManager {
    /// Create new webview manager
    pub async fn new(engine: Arc<Mutex<ServoEngine>>, config: WebViewConfig) -> Result<Self> {
        Ok(Self {
            webviews: HashMap::new(),
            active_id: None,
            engine,
            next_id: 1,
            config,
        })
    }
    
    /// Create new webview tab
    pub async fn create_webview(&mut self) -> Result<WebViewId> {
        let id = WebViewId(BrowsingContextId::new());
        self.next_id += 1;
        
        let dns_resolver = Arc::new(tokio::sync::RwLock::new(
            GenesisDnsResolver::new("http://localhost:3000".to_string(), true)
        ));
        
        let webview = GenesisWebView::new(id, self.engine.clone(), dns_resolver).await?;
        
        self.webviews.insert(id, webview);
        self.active_id = Some(id);
        
        info!("📑 Created new WebView with ID: {}", id.0);
        Ok(id)
    }
    
    /// Close webview
    pub fn close_webview(&mut self, id: WebViewId) -> Result<()> {
        self.webviews.remove(&id);
        
        if self.active_id == Some(id) {
            self.active_id = self.webviews.keys().next().cloned();
        }
        
        info!("📑 Closed WebView with ID: {}", id.0);
        Ok(())
    }
    
    /// Get active webview
    pub fn active_webview(&mut self) -> Option<&mut GenesisWebView> {
        self.active_id.and_then(move |id| self.webviews.get_mut(&id))
    }
    
    /// Set active webview
    pub fn set_active(&mut self, id: WebViewId) -> Result<()> {
        if self.webviews.contains_key(&id) {
            self.active_id = Some(id);
            Ok(())
        } else {
            Err(anyhow::anyhow!("WebView {} not found", id.0))
        }
    }
    
    /// Get all webview IDs
    pub fn webview_ids(&self) -> Vec<WebViewId> {
        self.webviews.keys().cloned().collect()
    }
    
    /// Navigate in active webview
    pub async fn navigate(&mut self, url: &str) -> Result<()> {
        if let Some(webview) = self.active_webview() {
            webview.navigate(url).await
        } else {
            Err(anyhow::anyhow!("No active webview"))
        }
    }
    
    /// Go back in active webview
    pub async fn go_back(&mut self) -> Result<()> {
        if let Some(webview) = self.active_webview() {
            webview.go_back().await
        } else {
            Err(anyhow::anyhow!("No active webview"))
        }
    }
    
    /// Go forward in active webview
    pub async fn go_forward(&mut self) -> Result<()> {
        if let Some(webview) = self.active_webview() {
            webview.go_forward().await
        } else {
            Err(anyhow::anyhow!("No active webview"))
        }
    }
    
    /// Reload active webview
    pub async fn reload(&mut self) -> Result<()> {
        if let Some(webview) = self.active_webview() {
            webview.reload().await
        } else {
            Err(anyhow::anyhow!("No active webview"))
        }
    }
}

// Use Servo's WebViewId instead of defining our own

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_webview_creation() {
        let config = ServoConfig::default();
        let engine = Arc::new(Mutex::new(
            ServoEngine::new(config).await.unwrap()
        ));
        
        let dns_resolver = Arc::new(tokio::sync::RwLock::new(
            GenesisDnsResolver::new("http://localhost:3000".to_string(), true)
        ));
        
        let webview = GenesisWebView::new(
            WebViewId(1),
            engine,
            dns_resolver,
        ).await;
        
        assert!(webview.is_ok());
    }
    
    #[tokio::test]
    async fn test_genesis_domain_detection() {
        let config = ServoConfig::default();
        let engine = Arc::new(Mutex::new(
            ServoEngine::new(config).await.unwrap()
        ));
        
        let dns_resolver = Arc::new(tokio::sync::RwLock::new(
            GenesisDnsResolver::new("http://localhost:3000".to_string(), true)
        ));
        
        let webview = GenesisWebView::new(
            WebViewId(1),
            engine,
            dns_resolver,
        ).await.unwrap();
        
        assert!(webview.is_genesis_domain("test.genesis"));
        assert!(webview.is_genesis_domain("freedom.free"));
        assert!(!webview.is_genesis_domain("google.com"));
    }
}