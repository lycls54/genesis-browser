use minifb::{Key, Window, WindowOptions};
use std::time::{Duration, Instant};
use tracing::{info, error, debug};
use url::Url;
use html_escape;

const WIDTH: usize = 1200;
const HEIGHT: usize = 800;

/// Genesis Browser UI using minifb
pub struct GenesisBrowserUI {
    window: Window,
    buffer: Vec<u32>,
    current_url: Option<String>,
    address_bar: String,
    is_loading: bool,
    dns_resolver: genesis_dns::GenesisDnsResolver,
    page_content: String,
}

impl GenesisBrowserUI {
    /// Create new Genesis Browser UI
    pub fn new(genesis_node_url: String) -> Result<Self, Box<dyn std::error::Error>> {
        info!("🎨 Creating Genesis Browser UI ({}x{})", WIDTH, HEIGHT);

        let mut window = Window::new(
            "Genesis Browser - Decentralized Web",
            WIDTH,
            HEIGHT,
            WindowOptions::default(),
        )?;

        // Limit update rate
        window.limit_update_rate(Some(Duration::from_millis(16))); // ~60 FPS

        let buffer = vec![0; WIDTH * HEIGHT];
        let dns_resolver = genesis_dns::GenesisDnsResolver::new(genesis_node_url, true);

        Ok(Self {
            window,
            buffer,
            current_url: None,
            address_bar: String::new(),
            is_loading: false,
            dns_resolver,
            page_content: String::new(),
        })
    }

    /// Run the browser main loop
    pub async fn run(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        info!("🚀 Starting Genesis Browser UI main loop");

        // Show welcome screen
        self.show_welcome_screen();

        while self.window.is_open() && !self.window.is_key_down(Key::Escape) {
            // Handle input
            self.handle_input().await;

            // Update display
            self.update_display();

            // Update window
            self.window.update_with_buffer(&self.buffer, WIDTH, HEIGHT)?;
        }

        info!("🛑 Genesis Browser UI closed");
        Ok(())
    }

    /// Handle user input
    async fn handle_input(&mut self) {
        // Handle URL entry
        if self.window.is_key_pressed(Key::Enter, minifb::KeyRepeat::No) {
            if !self.address_bar.is_empty() {
                self.navigate(&self.address_bar.clone()).await;
            }
        }

        // Handle key input for address bar
        let keys = self.window.get_keys_pressed(minifb::KeyRepeat::No);
        for key in keys {
            match key {
                Key::Backspace => {
                    self.address_bar.pop();
                },
                Key::Space => {
                    self.address_bar.push(' ');
                },
                // Add more key handling as needed
                _ => {
                    if let Some(ch) = self.key_to_char(key) {
                        self.address_bar.push(ch);
                    }
                }
            }
        }
    }

    /// Convert key to character (simplified)
    fn key_to_char(&self, key: Key) -> Option<char> {
        match key {
            Key::A => Some('a'),
            Key::B => Some('b'),
            Key::C => Some('c'),
            Key::D => Some('d'),
            Key::E => Some('e'),
            Key::F => Some('f'),
            Key::G => Some('g'),
            Key::H => Some('h'),
            Key::I => Some('i'),
            Key::J => Some('j'),
            Key::K => Some('k'),
            Key::L => Some('l'),
            Key::M => Some('m'),
            Key::N => Some('n'),
            Key::O => Some('o'),
            Key::P => Some('p'),
            Key::Q => Some('q'),
            Key::R => Some('r'),
            Key::S => Some('s'),
            Key::T => Some('t'),
            Key::U => Some('u'),
            Key::V => Some('v'),
            Key::W => Some('w'),
            Key::X => Some('x'),
            Key::Y => Some('y'),
            Key::Z => Some('z'),
            Key::Key0 => Some('0'),
            Key::Key1 => Some('1'),
            Key::Key2 => Some('2'),
            Key::Key3 => Some('3'),
            Key::Key4 => Some('4'),
            Key::Key5 => Some('5'),
            Key::Key6 => Some('6'),
            Key::Key7 => Some('7'),
            Key::Key8 => Some('8'),
            Key::Key9 => Some('9'),
            Key::Period => Some('.'),
            Key::Slash => Some('/'),
            Key::Minus => Some('-'),
            _ => None,
        }
    }

    /// Navigate to URL
    async fn navigate(&mut self, url: &str) {
        info!("🔍 Navigating to: {}", url);
        
        self.is_loading = true;
        self.current_url = Some(url.to_string());

        // Parse URL
        let full_url = if !url.starts_with("http://") && !url.starts_with("https://") {
            format!("http://{}", url)
        } else {
            url.to_string()
        };

        match Url::parse(&full_url) {
            Ok(parsed_url) => {
                let domain = parsed_url.host_str().unwrap_or("unknown");
                
                // Check if it's a Genesis domain
                if self.is_genesis_domain(domain) {
                    debug!("🌐 Genesis domain detected: {}", domain);
                    
                    // Resolve using Genesis DNS
                    match self.dns_resolver.resolve(domain).await {
                        Ok(dns_result) => {
                            info!("✅ Genesis DNS resolved: {} -> {:?}", domain, dns_result.ip_address);
                            self.load_genesis_page(&dns_result, &parsed_url).await;
                        },
                        Err(e) => {
                            error!("❌ Genesis DNS resolution failed: {:?}", e);
                            self.show_error_page(&format!("Genesis DNS resolution failed: {:?}", e));
                        }
                    }
                } else {
                    debug!("🌍 Traditional domain: {}", domain);
                    self.load_traditional_page(&parsed_url).await;
                }
            },
            Err(e) => {
                error!("❌ Invalid URL: {:?}", e);
                self.show_error_page(&format!("Invalid URL: {}", e));
            }
        }

        self.is_loading = false;
    }

    /// Check if domain is Genesis blockchain domain
    fn is_genesis_domain(&self, domain: &str) -> bool {
        let genesis_tlds = [".genesis", ".free", ".web", ".defi", ".dao"];
        genesis_tlds.iter().any(|tld| domain.ends_with(tld))
    }

    /// Load Genesis blockchain page
    async fn load_genesis_page(&mut self, dns_result: &genesis_dns::DnsResult, url: &Url) {
        info!("📄 Loading Genesis page: {}", url);

        // Create content based on DNS result
        let content = if let Some(ip) = dns_result.ip_address {
            format!(
                r#"
<!DOCTYPE html>
<html>
<head>
    <title>Genesis Domain: {}</title>
    <style>
        body {{ font-family: Arial, sans-serif; margin: 40px; background: #f0f8ff; }}
        .header {{ background: #4169e1; color: white; padding: 20px; border-radius: 8px; }}
        .content {{ background: white; padding: 20px; margin: 20px 0; border-radius: 8px; }}
        .footer {{ text-align: center; color: #666; margin-top: 20px; }}
    </style>
</head>
<body>
    <div class="header">
        <h1>🌐 Genesis Blockchain Domain</h1>
        <p>Domain: {}</p>
        <p>Resolved IP: {}</p>
        <p>Resolver: Genesis Blockchain</p>
    </div>
    <div class="content">
        <h2>Welcome to the Decentralized Web!</h2>
        <p>This domain is resolved using Genesis blockchain technology.</p>
        <p><strong>Features:</strong></p>
        <ul>
            <li>✅ Decentralized DNS resolution</li>
            <li>✅ Censorship resistant</li>
            <li>✅ Community governed</li>
            <li>✅ No ICANN dependency</li>
        </ul>
        <p><strong>Domain Info:</strong></p>
        <ul>
            <li>TTL: {} seconds</li>
            <li>Timestamp: {}</li>
        </ul>
    </div>
    <div class="footer">
        <p>Powered by Genesis Blockchain | Decentralized Web Browser</p>
    </div>
</body>
</html>
                "#,
                url.host_str().unwrap_or("unknown"),
                url.host_str().unwrap_or("unknown"),
                ip,
                dns_result.ttl,
                dns_result.timestamp
            )
        } else if let Some(content_hash) = &dns_result.content_hash {
            format!(
                r#"
<!DOCTYPE html>
<html>
<head>
    <title>Genesis IPFS Domain: {}</title>
</head>
<body>
    <h1>🌐 IPFS Content</h1>
    <p>Domain: {}</p>
    <p>Content Hash: {}</p>
    <p>This content is stored on IPFS.</p>
</body>
</html>
                "#,
                url.host_str().unwrap_or("unknown"),
                url.host_str().unwrap_or("unknown"),
                content_hash
            )
        } else {
            format!(
                r#"
<!DOCTYPE html>
<html>
<head>
    <title>Genesis Domain Error</title>
</head>
<body>
    <h1>❌ Genesis Domain Resolution Error</h1>
    <p>Could not resolve domain: {}</p>
</body>
</html>
                "#,
                url.host_str().unwrap_or("unknown")
            )
        };

        self.page_content = content;
    }

    /// Load traditional web page
    async fn load_traditional_page(&mut self, url: &Url) {
        info!("📄 Loading traditional page: {}", url);

        // Mock traditional page loading
        let content = format!(
            r#"
<!DOCTYPE html>
<html>
<head>
    <title>Traditional Domain: {}</title>
    <style>
        body {{ font-family: Arial, sans-serif; margin: 40px; background: #fff8dc; }}
        .header {{ background: #daa520; color: white; padding: 20px; border-radius: 8px; }}
        .content {{ background: white; padding: 20px; margin: 20px 0; border-radius: 8px; }}
    </style>
</head>
<body>
    <div class="header">
        <h1>🌍 Traditional Web Domain</h1>
        <p>Domain: {}</p>
        <p>Resolver: Traditional DNS (Fallback)</p>
    </div>
    <div class="content">
        <h2>Traditional Web Page</h2>
        <p>This domain uses traditional DNS resolution as fallback.</p>
        <p><strong>Note:</strong> Genesis Browser supports both Genesis blockchain domains and traditional domains.</p>
    </div>
</body>
</html>
            "#,
            url.host_str().unwrap_or("unknown"),
            url.host_str().unwrap_or("unknown")
        );

        self.page_content = content;
    }

    /// Show error page
    fn show_error_page(&mut self, error: &str) {
        error!("❌ Showing error page: {}", error);

        let content = format!(
            r#"
<!DOCTYPE html>
<html>
<head>
    <title>Genesis Browser - Error</title>
    <style>
        body {{ font-family: Arial, sans-serif; margin: 40px; background: #ffe4e1; }}
        .error {{ background: #dc143c; color: white; padding: 20px; border-radius: 8px; }}
    </style>
</head>
<body>
    <div class="error">
        <h1>❌ Genesis Browser Error</h1>
        <p>{}</p>
    </div>
</body>
</html>
            "#,
            html_escape::encode_text(error)
        );

        self.page_content = content;
    }

    /// Show welcome screen
    fn show_welcome_screen(&mut self) {
        self.page_content = r#"
<!DOCTYPE html>
<html>
<head>
    <title>Genesis Browser - Welcome</title>
    <style>
        body { font-family: Arial, sans-serif; margin: 0; padding: 40px; background: linear-gradient(135deg, #667eea 0%, #764ba2 100%); color: white; }
        .container { max-width: 800px; margin: 0 auto; text-align: center; }
        .logo { font-size: 3em; margin-bottom: 20px; }
        .subtitle { font-size: 1.2em; margin-bottom: 40px; opacity: 0.9; }
        .features { text-align: left; margin: 40px 0; }
        .feature { margin: 10px 0; padding: 10px; background: rgba(255,255,255,0.1); border-radius: 5px; }
        .domains { margin: 40px 0; }
        .domain-list { list-style: none; padding: 0; }
        .domain-list li { display: inline-block; margin: 10px; padding: 10px 20px; background: rgba(255,255,255,0.2); border-radius: 20px; }
    </style>
</head>
<body>
    <div class="container">
        <div class="logo">🌐 GENESIS BROWSER</div>
        <div class="subtitle">The First Decentralized Web Browser<br>No ICANN • No Censorship • Pure Freedom</div>
        
        <div class="features">
            <div class="feature">✅ Decentralized DNS using Genesis Blockchain</div>
            <div class="feature">✅ Censorship-resistant domain resolution</div>
            <div class="feature">✅ Community-governed domains</div>
            <div class="feature">✅ Traditional DNS fallback support</div>
            <div class="feature">✅ Built on 2M+ TPS Genesis blockchain</div>
        </div>

        <div class="domains">
            <h3>Try Genesis Domains:</h3>
            <ul class="domain-list">
                <li>welcome.genesis</li>
                <li>freedom.free</li>
                <li>decentral.web</li>
                <li>exchange.defi</li>
                <li>community.dao</li>
            </ul>
        </div>

        <p>Type a URL in the address bar and press Enter to navigate!</p>
    </div>
</body>
</html>
        "#.to_string();
    }

    /// Update display buffer
    fn update_display(&mut self) {
        // Clear buffer with white background
        for pixel in &mut self.buffer {
            *pixel = 0xFFFFFF; // White
        }

        // Draw address bar
        self.draw_address_bar();

        // Draw page content (simplified HTML rendering)
        self.draw_page_content();

        // Draw loading indicator if loading
        if self.is_loading {
            self.draw_loading_indicator();
        }
    }

    /// Draw address bar
    fn draw_address_bar(&mut self) {
        // Draw toolbar background (top 80 pixels)
        for y in 0..80 {
            for x in 0..WIDTH {
                let color = if y < 10 {
                    0x4169E1 // Genesis blue header
                } else if y < 70 {
                    0xF8F8F8 // Light background for address bar
                } else {
                    0xDDDDDD // Separator line
                };
                self.buffer[y * WIDTH + x] = color;
            }
        }

        // Draw Genesis Browser title
        let title = "GENESIS BROWSER - Decentralized Web";
        for (i, _) in title.chars().enumerate().take(50) {
            let x = 20 + i * 12;
            if x < WIDTH - 20 {
                for dy in 0..8 {
                    for dx in 0..8 {
                        let px = x + dx;
                        let py = 2 + dy;
                        if px < WIDTH && py < 10 {
                            self.buffer[py * WIDTH + px] = 0xFFFFFF; // White text
                        }
                    }
                }
            }
        }

        // Draw address bar border (more prominent)
        let bar_start_x = 50;
        let bar_end_x = WIDTH - 50;
        let bar_start_y = 15;
        let bar_end_y = 55;

        // Address bar background
        for y in bar_start_y..bar_end_y {
            for x in bar_start_x..bar_end_x {
                if x < WIDTH && y < 80 {
                    let color = if y == bar_start_y || y == bar_end_y - 1 || 
                                  x == bar_start_x || x == bar_end_x - 1 {
                        0x666666 // Border
                    } else {
                        0xFFFFFF // White input field
                    };
                    self.buffer[y * WIDTH + x] = color;
                }
            }
        }

        // Draw URL text (much more visible)
        let url_text = if self.address_bar.is_empty() {
            "🌐 Type Genesis domain here (e.g., freedom.genesis) and press ENTER..."
        } else {
            &self.address_bar
        };

        // Draw text with better visibility
        for (i, ch) in url_text.chars().enumerate().take(120) {
            let x = bar_start_x + 10 + i * 8;
            if x < bar_end_x - 10 {
                // Draw character as block (simplified font)
                let color = if self.address_bar.is_empty() { 0x888888 } else { 0x000000 };
                
                for dy in 0..20 {
                    for dx in 0..6 {
                        let px = x + dx;
                        let py = bar_start_y + 8 + dy;
                        if px < WIDTH && py < bar_end_y - 5 {
                            // Simple character representation
                            let should_draw = match ch {
                                'a'..='z' | 'A'..='Z' | '0'..='9' => {
                                    (dy >= 5 && dy <= 15) && (dx >= 1 && dx <= 4)
                                },
                                '.' => {
                                    (dy >= 12 && dy <= 15) && (dx >= 2 && dx <= 3)
                                },
                                ':' => {
                                    ((dy >= 6 && dy <= 8) || (dy >= 12 && dy <= 14)) && (dx >= 2 && dx <= 3)
                                },
                                '/' => {
                                    (dy >= 5 && dy <= 15) && (dx == 2 || dx == 3)
                                },
                                '-' => {
                                    (dy >= 9 && dy <= 11) && (dx >= 1 && dx <= 4)
                                },
                                ' ' => false,
                                '🌐' => {
                                    (dy >= 6 && dy <= 14) && (dx >= 1 && dx <= 4)
                                },
                                _ => {
                                    (dy >= 7 && dy <= 13) && (dx >= 1 && dx <= 4)
                                }
                            };
                            
                            if should_draw {
                                self.buffer[py * WIDTH + px] = color;
                            }
                        }
                    }
                }
            }
        }

        // Draw cursor if typing
        if !self.address_bar.is_empty() {
            let cursor_x = bar_start_x + 10 + self.address_bar.len() * 8;
            if cursor_x < bar_end_x - 10 {
                for dy in 0..20 {
                    let py = bar_start_y + 8 + dy;
                    if cursor_x < WIDTH && py < bar_end_y - 5 {
                        self.buffer[py * WIDTH + cursor_x] = 0x000000; // Black cursor
                    }
                }
            }
        }

        // Draw navigation buttons (Back, Forward, Refresh)
        let button_y = bar_start_y + 5;
        let button_height = 25;
        
        // Back button
        for y in button_y..(button_y + button_height) {
            for x in 10..35 {
                if x < WIDTH && y < 80 {
                    let color = if y == button_y || y == button_y + button_height - 1 || 
                                  x == 10 || x == 34 {
                        0x666666 // Border
                    } else {
                        0xE0E0E0 // Button background
                    };
                    self.buffer[y * WIDTH + x] = color;
                }
            }
        }

        // Draw "<" in back button
        for dy in 8..17 {
            for dx in 15..20 {
                if dx < WIDTH && (button_y + dy) < 80 {
                    if (dy == 12 && dx >= 15 && dx <= 19) || 
                       (dy == 11 && dx >= 16 && dx <= 18) ||
                       (dy == 13 && dx >= 16 && dx <= 18) {
                        self.buffer[(button_y + dy) * WIDTH + dx] = 0x333333;
                    }
                }
            }
        }
    }

    /// Draw page content (simplified)
    fn draw_page_content(&mut self) {
        // Draw content area (below address bar)
        let content_start_y = 90;
        
        // Fill content area with light background
        for y in content_start_y..HEIGHT {
            for x in 0..WIDTH {
                let color = if self.page_content.contains("Genesis") {
                    0xF0F8FF // Alice blue for Genesis pages
                } else if self.page_content.contains("Traditional") {
                    0xFFF8DC // Cornsilk for traditional pages
                } else if self.page_content.contains("Error") {
                    0xFFE4E1 // Misty rose for errors
                } else {
                    0xFFFFFF // White for welcome
                };
                self.buffer[y * WIDTH + x] = color;
            }
        }

        // Draw content based on page type
        if self.page_content.contains("Genesis") {
            // Draw Genesis blockchain indicator
            self.draw_genesis_page_indicator(content_start_y);
        } else if self.page_content.contains("Traditional") {
            // Draw traditional web indicator
            self.draw_traditional_page_indicator(content_start_y);
        } else if self.page_content.contains("Error") {
            // Draw error indicator
            self.draw_error_page_indicator(content_start_y);
        } else {
            // Draw welcome page
            self.draw_welcome_page_indicator(content_start_y);
        }
    }

    /// Draw loading indicator
    fn draw_loading_indicator(&mut self) {
        // Draw spinning loading indicator (simplified)
        let center_x = WIDTH / 2;
        let center_y = HEIGHT / 2;
        let radius = 20;

        // Draw circle
        for angle in 0..360 {
            let rad = (angle as f32) * std::f32::consts::PI / 180.0;
            let x = center_x as f32 + radius as f32 * rad.cos();
            let y = center_y as f32 + radius as f32 * rad.sin();

            if x >= 0.0 && x < WIDTH as f32 && y >= 0.0 && y < HEIGHT as f32 {
                let px = x as usize;
                let py = y as usize;
                if px < WIDTH && py < HEIGHT {
                    self.buffer[py * WIDTH + px] = 0x4169E1; // Royal blue
                }
            }
        }
    }

    /// Draw Genesis page indicator
    fn draw_genesis_page_indicator(&mut self, start_y: usize) {
        // Draw Genesis blockchain logo (simplified)
        let logo_x = 100;
        let logo_y = start_y + 50;
        
        // Draw "🌐 GENESIS BLOCKCHAIN" text
        let text = "GENESIS BLOCKCHAIN DOMAIN";
        for (i, _) in text.chars().enumerate().take(25) {
            let x = logo_x + i * 16;
            if x < WIDTH - 100 {
                for dy in 0..20 {
                    for dx in 0..12 {
                        let px = x + dx;
                        let py = logo_y + dy;
                        if px < WIDTH && py < HEIGHT {
                            if dy >= 4 && dy <= 16 && dx >= 2 && dx <= 10 {
                                self.buffer[py * WIDTH + px] = 0x4169E1; // Genesis blue
                            }
                        }
                    }
                }
            }
        }

        // Draw checkmarks for features
        let features = ["Decentralized DNS", "Censorship Resistant", "Community Governed"];
        for (i, feature) in features.iter().enumerate() {
            let y = start_y + 120 + i * 40;
            
            // Draw checkmark
            for dy in 0..15 {
                for dx in 0..15 {
                    if 50 + dx < WIDTH && y + dy < HEIGHT {
                        if (dy >= 6 && dy <= 9 && dx >= 6 && dx <= 9) {
                            self.buffer[(y + dy) * WIDTH + (50 + dx)] = 0x00AA00; // Green checkmark
                        }
                    }
                }
            }

            // Draw feature text
            for (j, _) in feature.chars().enumerate().take(20) {
                let x = 80 + j * 12;
                if x < WIDTH - 100 {
                    for dy in 0..15 {
                        for dx in 0..8 {
                            let px = x + dx;
                            let py = y + dy;
                            if px < WIDTH && py < HEIGHT {
                                if dy >= 2 && dy <= 12 && dx >= 1 && dx <= 6 {
                                    self.buffer[py * WIDTH + px] = 0x333333; // Dark gray text
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    /// Draw traditional page indicator
    fn draw_traditional_page_indicator(&mut self, start_y: usize) {
        let logo_x = 100;
        let logo_y = start_y + 50;
        
        // Draw "TRADITIONAL WEB" text
        let text = "TRADITIONAL WEB DOMAIN";
        for (i, _) in text.chars().enumerate().take(22) {
            let x = logo_x + i * 16;
            if x < WIDTH - 100 {
                for dy in 0..20 {
                    for dx in 0..12 {
                        let px = x + dx;
                        let py = logo_y + dy;
                        if px < WIDTH && py < HEIGHT {
                            if dy >= 4 && dy <= 16 && dx >= 2 && dx <= 10 {
                                self.buffer[py * WIDTH + px] = 0xDAA520; // Golden rod
                            }
                        }
                    }
                }
            }
        }

        // Draw warning for centralized nature
        let warning_y = start_y + 120;
        let warning_text = "Using traditional DNS fallback";
        for (i, _) in warning_text.chars().enumerate().take(30) {
            let x = 100 + i * 12;
            if x < WIDTH - 100 {
                for dy in 0..15 {
                    for dx in 0..8 {
                        let px = x + dx;
                        let py = warning_y + dy;
                        if px < WIDTH && py < HEIGHT {
                            if dy >= 2 && dy <= 12 && dx >= 1 && dx <= 6 {
                                self.buffer[py * WIDTH + px] = 0x666666; // Gray text
                            }
                        }
                    }
                }
            }
        }
    }

    /// Draw error page indicator
    fn draw_error_page_indicator(&mut self, start_y: usize) {
        let center_x = WIDTH / 2;
        let center_y = start_y + 100;
        
        // Draw error symbol (large X)
        for i in 0..50 {
            let x1 = center_x - 25 + i;
            let y1 = center_y - 25 + i;
            let x2 = center_x + 25 - i;
            let y2 = center_y - 25 + i;
            
            if x1 < WIDTH && y1 < HEIGHT {
                self.buffer[y1 * WIDTH + x1] = 0xFF0000; // Red
            }
            if x2 < WIDTH && y2 < HEIGHT {
                self.buffer[y2 * WIDTH + x2] = 0xFF0000; // Red
            }
        }

        // Draw "ERROR" text
        let error_text = "ERROR - RESOLUTION FAILED";
        for (i, _) in error_text.chars().enumerate().take(25) {
            let x = center_x - 200 + i * 16;
            if x < WIDTH {
                for dy in 0..20 {
                    for dx in 0..12 {
                        let px = x + dx;
                        let py = center_y + 60 + dy;
                        if px < WIDTH && py < HEIGHT {
                            if dy >= 4 && dy <= 16 && dx >= 2 && dx <= 10 {
                                self.buffer[py * WIDTH + px] = 0xFF0000; // Red text
                            }
                        }
                    }
                }
            }
        }
    }

    /// Draw welcome page indicator
    fn draw_welcome_page_indicator(&mut self, start_y: usize) {
        let center_x = WIDTH / 2;
        let center_y = start_y + 100;
        
        // Draw large Genesis logo (circle with G)
        for angle in 0..360 {
            let rad = (angle as f32) * std::f32::consts::PI / 180.0;
            let radius = 60.0;
            let x = center_x as f32 + radius * rad.cos();
            let y = center_y as f32 + radius * rad.sin();
            
            if x >= 0.0 && x < WIDTH as f32 && y >= 0.0 && y < HEIGHT as f32 {
                let px = x as usize;
                let py = y as usize;
                if px < WIDTH && py < HEIGHT {
                    self.buffer[py * WIDTH + px] = 0x4169E1; // Genesis blue circle
                }
            }
        }

        // Draw "GENESIS" text below logo
        let welcome_text = "WELCOME TO GENESIS BROWSER";
        for (i, _) in welcome_text.chars().enumerate().take(26) {
            let x = center_x - 200 + i * 16;
            if x < WIDTH {
                for dy in 0..25 {
                    for dx in 0..12 {
                        let px = x + dx;
                        let py = center_y + 100 + dy;
                        if px < WIDTH && py < HEIGHT {
                            if dy >= 5 && dy <= 20 && dx >= 2 && dx <= 10 {
                                self.buffer[py * WIDTH + px] = 0x4169E1; // Genesis blue text
                            }
                        }
                    }
                }
            }
        }

        // Draw instruction text
        let instruction = "Type a domain in the address bar above and press ENTER";
        for (i, _) in instruction.chars().enumerate().take(55) {
            let x = center_x - 300 + i * 11;
            if x < WIDTH {
                for dy in 0..15 {
                    for dx in 0..8 {
                        let px = x + dx;
                        let py = center_y + 150 + dy;
                        if px < WIDTH && py < HEIGHT {
                            if dy >= 2 && dy <= 12 && dx >= 1 && dx <= 6 {
                                self.buffer[py * WIDTH + px] = 0x666666; // Gray instruction text
                            }
                        }
                    }
                }
            }
        }
    }
}