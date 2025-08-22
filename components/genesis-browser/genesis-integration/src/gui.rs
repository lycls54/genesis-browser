use std::cell::{Cell, RefCell};
use std::rc::Rc;
use std::sync::Arc;
use std::time::Instant;

use anyhow::Result;
use tracing::{info, warn, error};
use winit::event_loop::{EventLoop, ActiveEventLoop};
use winit::window::Window;
use winit::event::{Event, WindowEvent, ElementState, MouseButton};
use winit::dpi::PhysicalSize;

use egui::{
    Button, CentralPanel, Frame, Key, Label, Modifiers, TopBottomPanel, Vec2, pos2,
    Color32, FontId, RichText, Stroke
};
use egui_glow::CallbackFn;
use egui_winit::EventResponse;

// OpenGL and rendering
use glutin::prelude::*;
use glutin::{context::PossiblyCurrentContext, display::Display, surface::Surface};
use glutin::config::ConfigTemplateBuilder;
use glutin::context::ContextAttributesBuilder;
use glutin::display::GetGlDisplay;
use glutin::surface::{SurfaceAttributesBuilder, WindowSurface};
// Import both versions to handle the compatibility issue
use winit::raw_window_handle::{HasWindowHandle, HasDisplayHandle};
use glow::HasContext;

use crate::servo_integration::GenesisBrowserEngine;
use genesis_dns::GenesisDnsResolver;

/// Genesis Browser GUI - Custom Servo-based browser window
pub struct GenesisBrowserGUI {
    /// Browser engine instance
    browser_engine: Option<GenesisBrowserEngine>,
    /// Window instance
    window: Option<Window>,
    /// egui context for UI rendering
    egui_ctx: Option<egui::Context>,
    /// egui-winit integration
    egui_winit: Option<egui_winit::State>,
    /// OpenGL display
    gl_display: Option<Display>,
    /// OpenGL surface
    gl_surface: Option<Surface<WindowSurface>>,
    /// OpenGL context
    gl_context: Option<PossiblyCurrentContext>,
    /// egui_glow renderer
    egui_glow: Option<egui_glow::Painter>,
    /// Current URL
    current_url: RefCell<String>,
    /// URL input field content
    url_input: RefCell<String>,
    /// Genesis DNS status
    dns_status: RefCell<String>,
    /// Browser status
    is_loading: Cell<bool>,
    /// Genesis node connection status
    genesis_connected: Cell<bool>,
    /// Last update time
    last_update: Instant,
}

impl GenesisBrowserGUI {
    /// Create new Genesis Browser GUI
    pub fn new() -> Self {
        info!("🎨 Creating Genesis Browser GUI");
        
        Self {
            browser_engine: None,
            window: None,
            egui_ctx: None,
            egui_winit: None,
            gl_display: None,
            gl_surface: None,
            gl_context: None,
            egui_glow: None,
            current_url: RefCell::new("http://welcome.genesis".to_string()),
            url_input: RefCell::new("http://welcome.genesis".to_string()),
            dns_status: RefCell::new("Genesis DNS Ready".to_string()),
            is_loading: Cell::new(false),
            genesis_connected: Cell::new(false),
            last_update: Instant::now(),
        }
    }
    
    /// Initialize the GUI with window and browser engine
    pub async fn initialize(&mut self, browser_engine: GenesisBrowserEngine) -> Result<()> {
        info!("🚀 Initializing Genesis Browser GUI");
        
        self.browser_engine = Some(browser_engine);
        info!("✅ Genesis Browser GUI initialized");
        
        Ok(())
    }
    
    /// Create and setup the main window
    pub fn create_window(&mut self, event_loop: &ActiveEventLoop) -> Result<()> {
        info!("🪟 Creating Genesis Browser window");
        
        let window_attributes = Window::default_attributes()
            .with_title("Genesis Browser - Decentralized Web Freedom")
            .with_inner_size(PhysicalSize::new(1200, 800))
            .with_min_inner_size(PhysicalSize::new(800, 600));
            
        let window = event_loop.create_window(window_attributes)?;
        
        info!("🔧 Setting up OpenGL context");
        
        // Setup OpenGL context with glutin
        let window_handle = window.window_handle().map_err(|e| anyhow::anyhow!("Failed to get window handle: {}", e))?;
        let display_handle = window.display_handle().map_err(|e| anyhow::anyhow!("Failed to get display handle: {}", e))?;
        
        // Convert to glutin's expected raw handle format
        let raw_display_handle = display_handle.as_raw();
        let raw_window_handle = window_handle.as_raw();
        
        // Ensure the window and display handles are from the same underlying platform
        info!("🔍 Display handle type: {:?}", raw_display_handle);
        info!("🔍 Window handle type: {:?}", raw_window_handle);
        
        // Create OpenGL display - try EGL first
        let gl_display = unsafe {
            glutin::display::Display::new(
                raw_display_handle,
                glutin::display::DisplayApiPreference::Egl,
            )
        }.map_err(|e| anyhow::anyhow!("Failed to create GL display: {}", e))?;
        
        // Create OpenGL config - match the window's visual
        let config_template = ConfigTemplateBuilder::new()
            .with_alpha_size(8)
            .with_transparency(false)  // Disable transparency to avoid visual conflicts
            .build();
            
        // Find a config that matches our window's visual
        let configs = unsafe { gl_display.find_configs(config_template) }?;
        let config = configs
            .reduce(|acc, config| {
                // Prefer the first config that works
                acc
            })
            .ok_or_else(|| anyhow::anyhow!("No suitable GL config found"))?;
        
        // Create OpenGL context - use the same raw_window_handle
        let context_attributes = ContextAttributesBuilder::new()
            .build(Some(raw_window_handle));
            
        let not_current_context = unsafe {
            gl_display.create_context(&config, &context_attributes)?
        };
        
        // Create window surface
        let size = window.inner_size();
        let surface_attributes = SurfaceAttributesBuilder::<WindowSurface>::new()
            .build(raw_window_handle, size.width.try_into()?, size.height.try_into()?);
            
        let surface = unsafe {
            gl_display.create_window_surface(&config, &surface_attributes)?
        };
        
        // Make context current
        let gl_context = not_current_context.make_current(&surface)?;
        
        info!("🎨 Initializing egui_glow renderer");
        
        // Initialize egui_glow
        let gl = unsafe { 
            glow::Context::from_loader_function(|s| {
                let c_str = std::ffi::CString::new(s).unwrap();
                gl_display.get_proc_address(&c_str) as *const _
            })
        };
        
        let egui_glow = egui_glow::Painter::new(Arc::new(gl), "", None, false)
            .map_err(|e| anyhow::anyhow!("Failed to create egui_glow painter: {}", e))?;
            
        // Initialize egui
        let egui_ctx = egui::Context::default();
        let egui_winit = egui_winit::State::new(
            egui_ctx.clone(),
            egui::ViewportId::ROOT,
            &window,
            None,
            None,
            None
        );
        
        // Store everything
        self.window = Some(window);
        self.egui_ctx = Some(egui_ctx);
        self.egui_winit = Some(egui_winit);
        self.gl_display = Some(gl_display);
        self.gl_surface = Some(surface);
        self.gl_context = Some(gl_context);
        self.egui_glow = Some(egui_glow);
        
        info!("✅ Genesis Browser window created with OpenGL context");
        Ok(())
    }
    
    /// Handle window events
    pub fn handle_event(&mut self, event: &WindowEvent) -> EventResponse {
        if let (Some(egui_winit), Some(window)) = (self.egui_winit.as_mut(), self.window.as_ref()) {
            egui_winit.on_window_event(window, event)
        } else {
            EventResponse::default()
        }
    }
    
    /// Update and render the UI
    pub fn update_and_render(&mut self) -> Result<()> {
        // Check if we have all components needed for rendering
        if let (Some(egui_ctx), Some(egui_winit), Some(window), Some(egui_glow), Some(gl_context), Some(gl_surface)) = (
            self.egui_ctx.as_ref(),
            self.egui_winit.as_mut(),
            self.window.as_ref(),
            self.egui_glow.as_mut(),
            self.gl_context.as_ref(),
            self.gl_surface.as_ref()
        ) {
            let raw_input = egui_winit.take_egui_input(window);
            
            // Clone necessary data before the closure
            let current_url = self.current_url.borrow().clone();
            let url_input = self.url_input.borrow().clone();
            let dns_status = self.dns_status.borrow().clone();
            let is_loading = self.is_loading.get();
            let genesis_connected = self.genesis_connected.get();
            
            let full_output = egui_ctx.run(raw_input, |ctx| {
                // Simple Genesis Browser UI
                egui::CentralPanel::default().show(ctx, |ui| {
                    ui.vertical_centered(|ui| {
                        ui.add_space(50.0);
                        
                        ui.label(
                            egui::RichText::new("🌐 GENESIS BROWSER")
                                .size(32.0)
                                .color(egui::Color32::from_rgb(34, 197, 94))
                                .strong()
                        );
                        
                        ui.add_space(20.0);
                        ui.label("The First Decentralized Web Browser");
                        ui.label("No ICANN • No Censorship • Pure Freedom");
                        
                        ui.add_space(30.0);
                        ui.label(format!("Current URL: {}", current_url));
                        ui.label(format!("DNS Status: {}", dns_status));
                        
                        ui.add_space(30.0);
                        
                        ui.horizontal(|ui| {
                            if ui.button("🏠 Genesis Home").clicked() {
                                info!("🏠 Genesis Home button clicked");
                            }
                            if ui.button("🔍 Test DNS").clicked() {
                                info!("🔍 Test DNS button clicked");
                            }
                        });
                        
                        ui.add_space(50.0);
                        ui.label("🚀 Servo Web Engine Ready");
                        ui.label("Genesis DNS Integration Active");
                        
                        ui.add_space(20.0);
                        ui.separator();
                        ui.add_space(20.0);
                        
                        // Add status indicators
                        ui.horizontal(|ui| {
                            ui.label("Status:");
                            if genesis_connected {
                                ui.colored_label(egui::Color32::GREEN, "🟢 Connected");
                            } else {
                                ui.colored_label(egui::Color32::RED, "🔴 Disconnected");
                            }
                        });
                        
                        if is_loading {
                            ui.add_space(10.0);
                            ui.spinner();
                            ui.label("Loading...");
                        }
                    });
                });
            });
            
            // Handle platform output (clipboard, etc.)
            egui_winit.handle_platform_output(window, full_output.platform_output);
            
            // Actually render to the screen with OpenGL
            let clipped_primitives = egui_ctx.tessellate(full_output.shapes, full_output.pixels_per_point);
            let size = window.inner_size();
            
            // Set viewport
            unsafe {
                egui_glow.gl().viewport(0, 0, size.width as i32, size.height as i32);
                egui_glow.gl().clear_color(0.1, 0.1, 0.1, 1.0);
                egui_glow.gl().clear(glow::COLOR_BUFFER_BIT);
            }
            
            // Paint egui primitives
            egui_glow.paint_primitives(
                [size.width, size.height],
                full_output.pixels_per_point,
                &clipped_primitives,
            );
            
            // Swap buffers to display
            gl_surface.swap_buffers(gl_context)?;
            
            // Update and free textures  
            for (texture_id, image_delta) in &full_output.textures_delta.set {
                egui_glow.set_texture(*texture_id, image_delta);
            }
            
            for texture_id in &full_output.textures_delta.free {
                egui_glow.free_texture(*texture_id);
            }
        }
        
        Ok(())
    }
    
    /// Navigate to a URL
    fn navigate_to_url(&mut self, url: &str) {
        info!("🔍 Genesis Browser navigating to: {}", url);
        
        *self.current_url.borrow_mut() = url.to_string();
        *self.url_input.borrow_mut() = url.to_string();
        self.is_loading.set(true);
        
        // Update DNS status based on domain type
        if self.is_genesis_domain(url) {
            *self.dns_status.borrow_mut() = "Resolving Genesis Domain...".to_string();
            self.genesis_connected.set(true);
        } else {
            *self.dns_status.borrow_mut() = "Traditional DNS".to_string();
            self.genesis_connected.set(false);
        }
        
        // TODO: Integrate with Servo WebView navigation
        if let Some(ref mut engine) = self.browser_engine {
            // This would integrate with our Servo engine
            // engine.navigate(url).await;
        }
        
        // Simulate loading completion
        self.is_loading.set(false);
        
        if self.is_genesis_domain(url) {
            *self.dns_status.borrow_mut() = "Genesis DNS Active".to_string();
        } else {
            *self.dns_status.borrow_mut() = "Traditional DNS".to_string();
        }
    }
    
    /// Go back in history
    fn go_back(&mut self) {
        info!("⬅ Genesis Browser going back");
        // TODO: Implement back navigation
    }
    
    /// Go forward in history
    fn go_forward(&mut self) {
        info!("➡ Genesis Browser going forward");
        // TODO: Implement forward navigation
    }
    
    /// Reload current page
    fn reload(&mut self) {
        info!("🔄 Genesis Browser reloading");
        let url = self.current_url.borrow().clone();
        self.navigate_to_url(&url);
    }
    
    /// Check if URL is a Genesis domain
    fn is_genesis_domain(&self, url: &str) -> bool {
        let genesis_tlds = [".genesis", ".free", ".web", ".defi", ".dao"];
        genesis_tlds.iter().any(|tld| url.contains(tld))
    }
    
    /// Get window reference
    pub fn window(&self) -> Option<&Window> {
        self.window.as_ref()
    }
}