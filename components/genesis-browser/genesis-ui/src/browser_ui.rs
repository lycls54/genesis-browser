// Modern Genesis Browser UI - Compatibility wrapper
// Redirects to modern egui-based interface for better performance and visuals

use tracing::info;

/// Genesis Browser UI - Compatibility wrapper that launches modern UI
pub struct GenesisBrowserUI {
    genesis_node_url: String,
}

impl GenesisBrowserUI {
    /// Create new Genesis Browser UI (redirects to modern interface)
    pub fn new(genesis_node_url: String) -> Result<Self, Box<dyn std::error::Error>> {
        info!("🔄 GenesisBrowserUI wrapper - redirecting to modern egui interface");
        
        Ok(Self {
            genesis_node_url,
        })
    }

    /// Run the browser (launches modern UI)
    pub async fn run(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        info!("🚀 Launching Modern Genesis Browser UI with egui...");
        info!("✨ Features: 144 FPS, GPU acceleration, beautiful fonts");
        
        #[cfg(feature = "modern-ui")]
        {
            // Launch modern egui interface
            crate::ModernGenesisBrowser::run()?;
            Ok(())
        }
        
        #[cfg(not(feature = "modern-ui"))]
        {
            Err("Modern UI not available. Please enable 'modern-ui' feature.".into())
        }
    }
}