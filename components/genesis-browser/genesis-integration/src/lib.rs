// Genesis Integration - Servo engine and blockchain integration layer

pub mod servo_engine;
pub mod servo_integration;
pub mod webview;

// Re-export main types
pub use servo_engine::{ServoEngine, ServoConfig};
pub use servo_integration::{GenesisBrowserEngine, BrowserConfig};
pub use webview::{
    GenesisWebView, WebViewManager, WebViewConfig, WebViewEvent, 
    SecurityState, ConsoleMessage, ConsoleLevel
};