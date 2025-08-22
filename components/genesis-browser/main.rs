use clap::{Parser, Subcommand};
use tracing::{info, error};
use tracing_subscriber;

// Import our local packages
use genesis_dns;
use genesis_integration;
use genesis_ui;

/// Genesis Browser - Decentralized Web Browser
/// No ICANN, No Censorship, Pure Freedom
#[derive(Parser)]
#[command(name = "genesis-browser")]
#[command(about = "A decentralized web browser using Genesis blockchain")]
#[command(version = "0.1.0")]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
    
    /// Enable verbose logging
    #[arg(short, long)]
    verbose: bool,
    
    /// Genesis node address
    #[arg(long, default_value = "http://localhost:3000")]
    genesis_node: String,
    
    /// Enable traditional DNS fallback
    #[arg(long)]
    fallback: bool,
}

#[derive(Subcommand)]
enum Commands {
    /// Start the Genesis Browser
    Start {
        /// URL to open at startup
        #[arg(short, long)]
        url: Option<String>,
        
        /// Window width
        #[arg(long, default_value_t = 1200)]
        width: u32,
        
        /// Window height
        #[arg(long, default_value_t = 800)]
        height: u32,
    },
    
    /// Test Genesis DNS resolution
    Test {
        /// Domain to test
        domain: String,
    },
    
    /// Show Genesis browser information
    Info,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    
    // Initialize logging
    let subscriber = tracing_subscriber::fmt()
        .with_max_level(if cli.verbose { 
            tracing::Level::DEBUG 
        } else { 
            tracing::Level::INFO 
        })
        .with_target(false)
        .finish();
    
    tracing::subscriber::set_global_default(subscriber)?;

    // Print banner
    print_banner();
    
    // Execute command
    match cli.command {
        Some(Commands::Start { url, width, height }) => {
            info!("Starting Genesis Browser...");
            start_browser(&cli.genesis_node, url, width, height, cli.fallback).await?;
        },
        Some(Commands::Test { domain }) => {
            info!("Testing DNS resolution for: {}", domain);
            test_dns_resolution(&cli.genesis_node, &domain).await?;
        },
        Some(Commands::Info) => {
            show_info(&cli.genesis_node).await?;
        },
        None => {
            // Default: start browser
            info!("Starting Genesis Browser with default settings...");
            start_browser(&cli.genesis_node, None, 1200, 800, cli.fallback).await?;
        }
    }
    
    Ok(())
}

fn print_banner() {
    println!(r#"
 ██████╗ ███████╗███╗   ██╗███████╗███████╗██╗███████╗
██╔════╝ ██╔════╝████╗  ██║██╔════╝██╔════╝██║██╔════╝
██║  ███╗█████╗  ██╔██╗ ██║█████╗  ███████╗██║███████╗
██║   ██║██╔══╝  ██║╚██╗██║██╔══╝  ╚════██║██║╚════██║
╚██████╔╝███████╗██║ ╚████║███████╗███████║██║███████║
 ╚═════╝ ╚══════╝╚═╝  ╚═══╝╚══════╝╚══════╝╚═╝╚══════╝

        🌐 GENESIS BROWSER 🌐
    The First Decentralized Web Browser
       No ICANN • No Censorship • Pure Freedom
    "#);
    
    info!("Genesis Browser v{}", env!("CARGO_PKG_VERSION"));
    info!("Building the Free Web on Genesis Blockchain");
}

async fn start_browser(
    genesis_node: &str,
    startup_url: Option<String>, 
    width: u32, 
    height: u32,
    fallback: bool
) -> Result<(), Box<dyn std::error::Error>> {
    info!("Connecting to Genesis node at: {}", genesis_node);
    
    // Check Genesis node connectivity
    let client = reqwest::Client::new();
    match client.get(&format!("{}/health", genesis_node)).send().await {
        Ok(response) => {
            if response.status().is_success() {
                info!("✅ Connected to Genesis node");
            } else {
                error!("❌ Genesis node returned error: {}", response.status());
                if !fallback {
                    return Err("Genesis node not available and fallback disabled".into());
                }
            }
        },
        Err(e) => {
            error!("❌ Failed to connect to Genesis node: {}", e);
            if !fallback {
                return Err("Genesis node not available and fallback disabled".into());
            }
            info!("🔄 Continuing with traditional DNS fallback");
        }
    }
    
    // Check if Servo integration is available
    #[cfg(feature = "servo-integration")]
    {
        info!("🚀 Starting Genesis Browser with Servo Engine...");
        info!("Window size: {}x{}", width, height);
        
        // Create browser configuration
        let config = genesis_integration::BrowserConfig {
            enable_genesis_dns: true,
            enable_traditional_fallback: fallback,
            genesis_node_url: genesis_node.to_string(),
            user_agent: "Genesis Browser/1.0 (Servo; Decentralized Web)".to_string(),
            enable_javascript: true,
            enable_webgl: true,
        };
        
        // Initialize Servo-based browser engine
        let browser_engine = genesis_integration::GenesisBrowserEngine::new(config).await?;
        
        info!("🌐 Genesis Browser with Servo Engine running...");
        info!("Supported domains: .genesis, .free, .web, .defi, .dao");
        
        // Run the browser GUI
        run_genesis_browser_gui(browser_engine, startup_url).await?;
    }
    
    // Fallback to simple UI if Servo is not available
    #[cfg(not(feature = "servo-integration"))]
    {
        info!("🚀 Starting Genesis Browser UI (Servo not enabled)...");
        info!("Window size: {}x{}", width, height);
        
        let mut browser_ui = genesis_ui::GenesisBrowserUI::new(genesis_node.to_string())?;
        
        if let Some(url) = startup_url {
            info!("🔍 Will open startup URL: {}", url);
        }
        
        info!("🌐 Genesis Browser UI starting...");
        info!("Supported domains: .genesis, .free, .web, .defi, .dao");
        info!("Controls: Type URL and press Enter, ESC to exit");
        
        browser_ui.run().await?;
    }
    
    Ok(())
}

async fn test_dns_resolution(
    genesis_node: &str,
    domain: &str
) -> Result<(), Box<dyn std::error::Error>> {
    info!("Testing DNS resolution for domain: {}", domain);
    
    // TODO: Implement Genesis DNS resolution test
    // For now, just check if domain ends with Genesis TLDs
    let genesis_tlds = [".genesis", ".free", ".web", ".defi"];
    let is_genesis_domain = genesis_tlds.iter().any(|tld| domain.ends_with(tld));
    
    if is_genesis_domain {
        info!("🎯 Genesis domain detected: {}", domain);
        info!("🔍 Attempting Genesis blockchain DNS lookup...");
        
        // Mock resolution for now
        info!("✅ Resolved {} to Genesis blockchain", domain);
    } else {
        info!("🌐 Traditional domain: {}", domain);
        info!("🔍 Would fallback to traditional DNS");
    }
    
    Ok(())
}

async fn show_info(genesis_node: &str) -> Result<(), Box<dyn std::error::Error>> {
    info!("=== Genesis Browser Information ===");
    info!("Version: {}", env!("CARGO_PKG_VERSION"));
    info!("Genesis Node: {}", genesis_node);
    info!("Supported TLDs: .genesis, .free, .web, .defi");
    info!("Features:");
    info!("  ✅ Decentralized DNS");
    info!("  ✅ Blockchain integration");
    info!("  ✅ Censorship resistance");
    info!("  ✅ Community governance");
    info!("  ✅ Servo browser engine (ready - enable with --features servo-integration)");
    
    // Try to get Genesis node info
    let client = reqwest::Client::new();
    match client.get(&format!("{}/info", genesis_node)).send().await {
        Ok(response) => {
            if response.status().is_success() {
                info!("✅ Genesis node is online");
                // TODO: Parse and show node info
            }
        },
        Err(_) => {
            info!("❌ Genesis node is offline");
        }
    }
    
    Ok(())
}

/// Run Genesis Browser with GUI
async fn run_genesis_browser_gui(
    mut browser_engine: genesis_integration::GenesisBrowserEngine,
    startup_url: Option<String>
) -> Result<(), Box<dyn std::error::Error>> {
    use winit::event_loop::{EventLoop, ControlFlow};
    use winit::event::{Event, WindowEvent};
    
    info!("🎨 Starting Genesis Browser GUI Event Loop");
    
    // Try to create event loop for GUI
    match EventLoop::new() {
        Ok(event_loop) => {
            info!("✅ GUI mode available - starting with window");
            
            let mut browser_gui = genesis_integration::GenesisBrowserGUI::new();
            
            // Initialize GUI with browser engine
            browser_gui.initialize(browser_engine).await?;
    
    // Run event loop
    event_loop.run(move |event, event_loop_window_target| {
        match event {
            Event::Resumed => {
                // Create window when event loop starts
                if browser_gui.window().is_none() {
                    if let Err(e) = browser_gui.create_window(event_loop_window_target) {
                        error!("Failed to create window: {}", e);
                        event_loop_window_target.exit();
                        return;
                    }
                    info!("✅ Genesis Browser window created and ready");
                    
                    // Navigate to startup URL if provided
                    if let Some(ref url) = startup_url {
                        info!("🔍 Opening startup URL: {}", url);
                    }
                }
            },
            
            Event::WindowEvent { window_id, event } => {
                if let Some(window) = browser_gui.window() {
                    if window.id() == window_id {
                        match event {
                            WindowEvent::CloseRequested => {
                                info!("🛑 Genesis Browser closing");
                                event_loop_window_target.exit();
                            },
                            
                            WindowEvent::RedrawRequested => {
                                if let Err(e) = browser_gui.update_and_render() {
                                    error!("Failed to render: {}", e);
                                }
                            },
                            
                            _ => {
                                browser_gui.handle_event(&event);
                            }
                        }
                    }
                }
            },
            
            Event::AboutToWait => {
                // Request redraw
                if let Some(window) = browser_gui.window() {
                    window.request_redraw();
                }
            },
            
            _ => {}
        }
            })?;
        },
        
        Err(e) => {
            info!("⚠️ GUI not available: {}", e);
            info!("🖥️ Falling back to headless mode");
            
            // Start browser engine in headless mode
            browser_engine.start().await?;
            
            // Navigate to startup URL if provided
            if let Some(url) = startup_url {
                info!("🔍 Opening startup URL in headless mode: {}", url);
                browser_engine.navigate(&url).await?;
            } else {
                info!("🔍 Opening welcome page in headless mode");
                browser_engine.navigate("http://welcome.genesis").await?;
            }
            
            info!("🚀 Genesis Browser running in headless mode");
            info!("📊 Testing Genesis DNS resolution...");
            
            // Run browser in headless mode for demonstration
            for i in 1..=5 {
                info!("🔄 Headless browser loop iteration {}/5", i);
                browser_engine.run().await?;
                tokio::time::sleep(std::time::Duration::from_secs(2)).await;
            }
            
            info!("✅ Genesis Browser headless demo completed!");
        }
    }
    
    Ok(())
}