// Modern Genesis Browser UI with egui + eframe
// Exact Chrome replica with pixel-perfect details

#![cfg(feature = "modern-ui")]

use eframe::egui;
use egui::{
    CentralPanel, TopBottomPanel, SidePanel, ScrollArea, TextEdit,
    RichText, Color32, Ui, Vec2, Rounding, FontFamily, FontId, Stroke
};
use tracing::info;

use crate::enhanced_browser::BrowserUIState;
use std::collections::HashMap;

/// Smooth easing function for animations
fn ease_out_cubic(t: f32) -> f32 {
    1.0 - (1.0 - t).powi(3)
}

/// Tab animation state
#[derive(Clone)]
struct TabAnimation {
    /// Animation progress (0.0 to 1.0)
    progress: f32,
    /// Animation type
    anim_type: TabAnimationType,
    /// Target width
    target_width: f32,
}

#[derive(Clone, PartialEq)]
enum TabAnimationType {
    Opening,
    Closing,
}

/// Modern Genesis Browser using egui
pub struct ModernGenesisBrowser {
    ui_state: BrowserUIState,
    genesis_node_url: String,
    
    // UI state
    url_input: String,
    search_query: String,
    
    // Panel visibility
    show_devtools: bool,
    show_downloads: bool,
    show_bookmarks: bool,
    show_history: bool,
    
    // Tab scrolling
    tab_scroll_offset: f32,
    ensure_last_tab_visible: bool,
    
    // Tab animations
    tab_animations: std::collections::HashMap<String, TabAnimation>,
    
    // Performance metrics
    frame_time: f32,
    fps: f32,
    last_frame: std::time::Instant,
}

impl Default for ModernGenesisBrowser {
    fn default() -> Self {
        Self::new("http://localhost:3000".to_string())
    }
}

impl ModernGenesisBrowser {
    pub fn new(genesis_node_url: String) -> Self {
        info!("🎨 Creating Modern Genesis Browser with egui");
        
        Self {
            ui_state: BrowserUIState::default(),
            genesis_node_url,
            url_input: "genesis://welcome".to_string(),
            search_query: String::new(),
            show_devtools: false,
            show_downloads: false,
            show_bookmarks: false,
            show_history: false,
            tab_scroll_offset: 0.0,
            ensure_last_tab_visible: false,
            tab_animations: HashMap::new(),
            frame_time: 0.0,
            fps: 144.0,
            last_frame: std::time::Instant::now(),
        }
    }
    
    /// Run the modern browser
    pub fn run() -> Result<(), Box<dyn std::error::Error>> {
        info!("🚀 Starting Modern Genesis Browser with egui");
        
        let options = eframe::NativeOptions {
            viewport: egui::ViewportBuilder::default()
                .with_inner_size([1400.0, 900.0])
                .with_min_inner_size([800.0, 600.0])
                .with_title("Genesis Browser")
                .with_decorations(false) // Remove system title bar
                .with_resizable(true)
                .with_transparent(false)
                .with_icon(eframe::icon_data::from_png_bytes(&[]).unwrap_or_default()),
            vsync: false, // For maximum performance
            multisampling: 4, // Anti-aliasing for smooth graphics
            depth_buffer: 24,
            ..Default::default()
        };
        
        eframe::run_native(
            "Genesis Browser",
            options,
            Box::new(|_cc| Box::new(ModernGenesisBrowser::default())),
        )?;
        
        Ok(())
    }
}

impl eframe::App for ModernGenesisBrowser {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Calculate FPS
        let now = std::time::Instant::now();
        self.frame_time = now.duration_since(self.last_frame).as_secs_f32();
        self.fps = 1.0 / self.frame_time.max(0.001);
        self.last_frame = now;
        
        // Set Chrome-like light theme with exact colors
        let mut visuals = egui::Visuals::light();
        
        // Exact Chrome colors from DevTools
        visuals.window_fill = Color32::from_rgb(255, 255, 255);
        visuals.panel_fill = Color32::from_rgb(255, 255, 255);
        visuals.faint_bg_color = Color32::from_rgb(241, 243, 244); // Chrome's exact tab bar
        visuals.extreme_bg_color = Color32::from_rgb(255, 255, 255);
        
        // Chrome button styling
        visuals.widgets.noninteractive.bg_fill = Color32::from_rgb(255, 255, 255);
        visuals.widgets.inactive.bg_fill = Color32::from_rgb(255, 255, 255);
        visuals.widgets.hovered.bg_fill = Color32::from_rgb(248, 249, 250);
        visuals.widgets.active.bg_fill = Color32::from_rgb(241, 243, 244);
        
        // Chrome borders and strokes
        visuals.widgets.noninteractive.bg_stroke = Stroke::new(1.0, Color32::from_rgb(218, 220, 224));
        visuals.widgets.inactive.bg_stroke = Stroke::new(1.0, Color32::from_rgb(218, 220, 224));
        visuals.widgets.hovered.bg_stroke = Stroke::new(1.0, Color32::from_rgb(189, 193, 198));
        
        // Chrome text colors
        visuals.widgets.noninteractive.fg_stroke = Stroke::new(1.0, Color32::from_rgb(60, 64, 67));
        visuals.widgets.inactive.fg_stroke = Stroke::new(1.0, Color32::from_rgb(60, 64, 67));
        
        // Chrome selection colors
        visuals.selection.bg_fill = Color32::from_rgb(138, 180, 248);
        
        ctx.set_visuals(visuals);
        
        // Request repaint for smooth animation
        ctx.request_repaint();
        
        // Main browser UI
        self.render_top_panel(ctx);
        self.render_main_content(ctx);
        self.render_status_bar(ctx);
        self.render_side_panels(ctx);
    }
}

impl ModernGenesisBrowser {
    /// Render custom title bar with tabs (exact Chrome replica)
    fn render_top_panel(&mut self, ctx: &egui::Context) {
        // Chrome tab area with exact height and color
        TopBottomPanel::top("tab_area")
            .exact_height(35.0) // Chrome's exact tab height
            .show(ctx, |ui| {
                ui.style_mut().visuals.panel_fill = Color32::from_rgb(222, 225, 230); // Chrome's exact color
                
                // Get full panel rect
                let panel_rect = ui.available_rect_before_wrap();
                
                // Draggable area (invisible background)
                let drag_response = ui.allocate_rect(panel_rect, egui::Sense::click_and_drag());
                if drag_response.dragged() {
                    ctx.send_viewport_cmd(egui::ViewportCommand::StartDrag);
                }
                if drag_response.double_clicked() {
                    ctx.send_viewport_cmd(egui::ViewportCommand::Maximized(!ctx.input(|i| i.viewport().maximized.unwrap_or(false))));
                }
                
                // Fixed layout with absolute positioning
                let controls_width = 120.0;
                let new_tab_width = 36.0;
                let scroll_button_width = 32.0; // Width for scroll buttons
                let left_padding = 8.0;
                
                // Check if we need scroll (pre-calculate)
                let available_width = panel_rect.width() - controls_width - new_tab_width - left_padding;
                let tabs_borrow = self.ui_state.tabs.borrow();
                let tab_count = tabs_borrow.len();
                let min_tab_width = 120.0;
                let dynamic_tab_width = if tab_count == 0 {
                    240.0
                } else {
                    (available_width / tab_count as f32).clamp(min_tab_width, 240.0)
                };
                let needs_scroll_buttons = dynamic_tab_width <= min_tab_width && (tab_count as f32 * min_tab_width) > available_width;
                drop(tabs_borrow);
                
                // Adjust tab container width if scroll buttons are needed
                let tabs_width = if needs_scroll_buttons {
                    available_width - (scroll_button_width * 2.0) // Space for both scroll buttons
                } else {
                    available_width
                };
                
                // Render scroll buttons if needed
                if needs_scroll_buttons {
                    // Left scroll button
                    let left_scroll_rect = egui::Rect::from_min_size(
                        panel_rect.left_top() + Vec2::new(left_padding, 0.0),
                        Vec2::new(scroll_button_width, 35.0)
                    );
                    let left_response = ui.allocate_rect(left_scroll_rect, egui::Sense::click());
                    
                    // Draw left arrow
                    let left_center = left_scroll_rect.center();
                    if left_response.hovered() {
                        ui.painter().rect_filled(left_scroll_rect, 4.0, Color32::from_rgba_premultiplied(0, 0, 0, 20));
                    }
                    ui.painter().text(
                        left_center,
                        egui::Align2::CENTER_CENTER,
                        "◀",
                        egui::FontId::new(14.0, egui::FontFamily::Proportional),
                        if self.tab_scroll_offset > 0.0 { Color32::from_rgb(95, 99, 104) } else { Color32::from_rgb(180, 180, 180) }
                    );
                    
                    if left_response.clicked() && self.tab_scroll_offset > 0.0 {
                        self.tab_scroll_offset = (self.tab_scroll_offset - (min_tab_width * 1.5)).max(0.0);
                    }
                    
                    // Right scroll button
                    let right_scroll_rect = egui::Rect::from_min_size(
                        panel_rect.left_top() + Vec2::new(left_padding + scroll_button_width + tabs_width, 0.0),
                        Vec2::new(scroll_button_width, 35.0)
                    );
                    let right_response = ui.allocate_rect(right_scroll_rect, egui::Sense::click());
                    
                    // Draw right arrow
                    let right_center = right_scroll_rect.center();
                    if right_response.hovered() {
                        ui.painter().rect_filled(right_scroll_rect, 4.0, Color32::from_rgba_premultiplied(0, 0, 0, 20));
                    }
                    
                    let tabs = self.ui_state.tabs.borrow();
                    let max_scroll = (tabs.len() as f32 * min_tab_width - tabs_width).max(0.0);
                    drop(tabs);
                    
                    ui.painter().text(
                        right_center,
                        egui::Align2::CENTER_CENTER,
                        "▶",
                        egui::FontId::new(14.0, egui::FontFamily::Proportional),
                        if self.tab_scroll_offset < max_scroll { Color32::from_rgb(95, 99, 104) } else { Color32::from_rgb(180, 180, 180) }
                    );
                    
                    if right_response.clicked() && self.tab_scroll_offset < max_scroll {
                        self.tab_scroll_offset = (self.tab_scroll_offset + (min_tab_width * 1.5)).min(max_scroll);
                    }
                }
                
                // Tabs area (absolutely positioned, adjusted for scroll buttons)
                let tabs_x = if needs_scroll_buttons {
                    left_padding + scroll_button_width
                } else {
                    left_padding
                };
                
                let tabs_rect = egui::Rect::from_min_size(
                    panel_rect.left_top() + Vec2::new(tabs_x, 0.0),
                    Vec2::new(tabs_width, 35.0)
                );
                let visible_tabs_width = ui.allocate_ui_at_rect(tabs_rect, |ui| {
                    self.render_chrome_tabs(ui)
                }).inner;
                
                // New tab button (right after tabs or scroll buttons, no gap)
                let max_x = panel_rect.right() - controls_width - new_tab_width;
                let desired_x = if needs_scroll_buttons {
                    panel_rect.left() + left_padding + scroll_button_width * 2.0 + tabs_width + 4.0
                } else {
                    panel_rect.left() + left_padding + visible_tabs_width
                };
                let actual_x = desired_x.min(max_x);
                
                let new_tab_pos = egui::Pos2::new(actual_x, panel_rect.top() + 3.5);
                let new_tab_rect = egui::Rect::from_min_size(new_tab_pos, Vec2::new(28.0, 28.0));
                let new_tab_response = ui.allocate_rect(new_tab_rect, egui::Sense::click());
                
                // Draw new tab button
                if new_tab_response.hovered() {
                    ui.painter().rect_filled(new_tab_rect, 4.0, Color32::from_rgba_premultiplied(0, 0, 0, 20));
                }
                
                let center = new_tab_rect.center();
                let stroke = Stroke::new(1.5, Color32::from_rgb(95, 99, 104));
                ui.painter().line_segment([center - Vec2::new(6.0, 0.0), center + Vec2::new(6.0, 0.0)], stroke);
                ui.painter().line_segment([center - Vec2::new(0.0, 6.0), center + Vec2::new(0.0, 6.0)], stroke);
                
                if new_tab_response.clicked() {
                    self.handle_new_tab();
                }
                
                // Window controls (absolutely positioned, fixed)
                let controls_rect = egui::Rect::from_min_size(
                    egui::Pos2::new(panel_rect.right() - controls_width, panel_rect.top()),
                    Vec2::new(controls_width, 35.0)
                );
                ui.allocate_ui_at_rect(controls_rect, |ui| {
                    self.render_chrome_window_controls(ui, ctx);
                });
            });
        
        // Chrome toolbar area (address bar)
        TopBottomPanel::top("toolbar_area")
            .exact_height(40.0) // Chrome's exact toolbar height
            .show(ctx, |ui| {
                ui.style_mut().visuals.panel_fill = Color32::from_rgb(255, 255, 255);
                
                // Add subtle border at bottom
                let rect = ui.available_rect_before_wrap();
                ui.painter().line_segment(
                    [rect.left_bottom(), rect.right_bottom()],
                    Stroke::new(1.0, Color32::from_rgb(218, 220, 224))
                );
                
                self.render_chrome_navigation_bar(ui);
            });
    }
    
    
    /// Render Chrome-style tabs - REWRITTEN
    /// Returns where the new tab button should be positioned
    fn render_chrome_tabs(&mut self, ui: &mut Ui) -> f32 {
        let mut tab_actions = Vec::new();
        let container_width = ui.available_width();
        let max_tab_width = 240.0;
        let min_tab_width = 120.0; // Minimum tab width before scrolling
        let tab_height = 35.0;
        
        // Request continuous repaints for smooth animations
        if !self.tab_animations.is_empty() {
            ui.ctx().request_repaint();
        }
        
        // Update animations
        let delta_time = ui.input(|i| i.unstable_dt);
        let animation_speed = 5.0; // Higher = faster animation
        
        let mut completed_animations = Vec::new();
        let mut tabs_to_close = Vec::new();
        
        for (id, anim) in self.tab_animations.iter_mut() {
            anim.progress = (anim.progress + delta_time * animation_speed).min(1.0);
            if anim.progress >= 1.0 {
                if anim.anim_type == TabAnimationType::Closing {
                    completed_animations.push(id.clone());
                    tabs_to_close.push(id.clone());
                } else if anim.anim_type == TabAnimationType::Opening {
                    completed_animations.push(id.clone());
                }
            }
        }
        
        // Remove completed animations
        for id in completed_animations {
            self.tab_animations.remove(&id);
        }
        
        // Actually close tabs after animation completes
        for tab_id in tabs_to_close {
            let tabs = self.ui_state.tabs.borrow();
            let index = tabs.iter().position(|t| t.id == tab_id);
            drop(tabs);
            
            if let Some(idx) = index {
                self.ui_state.close_tab(idx);
            }
        }
        
        let tabs = self.ui_state.tabs.borrow();
        let tab_count = tabs.len();
        let active_index = self.ui_state.active_tab_index.get();
        
        // Calculate dynamic tab width - shrink tabs until minimum width
        let tab_width = if tab_count == 0 {
            max_tab_width
        } else {
            let width_per_tab = container_width / tab_count as f32;
            width_per_tab.clamp(min_tab_width, max_tab_width)
        };
        
        // Calculate if we need scrolling (only when tabs are at minimum width and still don't fit)
        let total_tabs_width = tab_count as f32 * tab_width;
        let needs_scroll = tab_width <= min_tab_width && total_tabs_width > container_width;
        
        // Handle mouse wheel scrolling
        if needs_scroll {
            let scroll_delta = ui.input(|i| i.scroll_delta);
            if scroll_delta.x != 0.0 || scroll_delta.y != 0.0 {
                // Scroll down (positive y) = go to right tabs (increase offset)
                // Scroll up (negative y) = go to left tabs (decrease offset)
                let delta = if scroll_delta.x != 0.0 { scroll_delta.x } else { -scroll_delta.y };
                self.tab_scroll_offset += delta * 2.0;
                let max_scroll = (total_tabs_width - container_width).max(0.0);
                self.tab_scroll_offset = self.tab_scroll_offset.clamp(0.0, max_scroll);
            }
        }
        
        // Auto-scroll to show last tab when requested
        if self.ensure_last_tab_visible && needs_scroll {
            let max_scroll = (total_tabs_width - container_width).max(0.0);
            self.tab_scroll_offset = max_scroll;
            self.ensure_last_tab_visible = false;
        }
        
        // Always ensure scroll doesn't leave empty space at the end
        if needs_scroll {
            let max_valid_scroll = (total_tabs_width - container_width).max(0.0);
            if self.tab_scroll_offset > max_valid_scroll {
                // Smoothly adjust scroll to prevent empty space
                self.tab_scroll_offset = max_valid_scroll;
            }
        } else {
            // If scrolling is not needed anymore, reset
            if self.tab_scroll_offset > 0.0 {
                self.tab_scroll_offset = 0.0;
            }
        }
        
        // Simple horizontal layout with manual scrolling
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing = Vec2::ZERO;
            
            // Set clip rect to hide overflowing tabs
            let clip_rect = egui::Rect::from_min_size(
                ui.cursor().min,
                Vec2::new(container_width, tab_height)
            );
            ui.set_clip_rect(clip_rect);
            
            // Apply scroll offset
            if needs_scroll && self.tab_scroll_offset > 0.0 {
                ui.add_space(-self.tab_scroll_offset);
            }
            
            // Render all tabs with dynamic width and animations
            for (index, tab) in tabs.iter().enumerate() {
                // Check if this tab has an animation
                let animated_width = if let Some(anim) = self.tab_animations.get(&tab.id) {
                    // Smooth easing function
                    let eased_progress = ease_out_cubic(anim.progress);
                    if anim.anim_type == TabAnimationType::Opening {
                        tab_width * eased_progress // Grow from 0 to full width
                    } else {
                        tab_width * (1.0 - eased_progress) // Shrink from full width to 0
                    }
                } else {
                    tab_width
                };
                
                if animated_width > 1.0 { // Only render if width > 1px
                    self.render_single_chrome_tab(ui, tab, index, active_index, animated_width, &mut tab_actions);
                }
            }
        });
        
        drop(tabs);
        
        // Process tab actions
        for (action, index) in tab_actions {
            match action {
                "switch" => {
                    self.ui_state.switch_to_tab(index);
                    if let Some(active_tab) = self.ui_state.get_active_tab() {
                        self.url_input = active_tab.url;
                    }
                }
                "close" => {
                    // Add closing animation, don't close immediately
                    let tabs = self.ui_state.tabs.borrow();
                    if index < tabs.len() {
                        let tab_id = tabs[index].id.clone();
                        // Only start animation if not already animating
                        if !self.tab_animations.contains_key(&tab_id) {
                            self.tab_animations.insert(
                                tab_id,
                                TabAnimation {
                                    progress: 0.0,
                                    anim_type: TabAnimationType::Closing,
                                    target_width: 0.0,
                                }
                            );
                        }
                    }
                    drop(tabs);
                    // Don't close immediately - wait for animation to complete
                }
                _ => {}
            }
        }
        
        // Return position for new tab button
        // If tabs don't fill container: right after last tab
        // If tabs overflow: at container edge
        if total_tabs_width < container_width {
            total_tabs_width
        } else {
            container_width
        }
    }
    
    /// Handle new tab creation
    fn handle_new_tab(&mut self) {
        self.ui_state.create_tab("genesis://newtab");
        
        // Add opening animation for the new tab
        let tabs = self.ui_state.tabs.borrow();
        if let Some(new_tab) = tabs.last() {
            self.tab_animations.insert(
                new_tab.id.clone(),
                TabAnimation {
                    progress: 0.0,
                    anim_type: TabAnimationType::Opening,
                    target_width: 240.0,
                }
            );
        }
        
        // Auto-scroll to show the new tab
        let tab_width = 240.0;
        let total_tabs_width = tabs.len() as f32 * tab_width;
        
        // Scroll to the end to show the new tab (if needed)
        // We'll use a flag to trigger scroll in the next frame
        self.ensure_last_tab_visible = true;
    }
    
    /// Render a single Chrome-style tab
    fn render_single_chrome_tab(&self, ui: &mut egui::Ui, tab: &crate::enhanced_browser::BrowserTab, index: usize, active_index: usize, tab_width: f32, tab_actions: &mut Vec<(&str, usize)>) {
        let is_active = index == active_index;
        let tab_height = 35.0;
        
        let tab_response = ui.allocate_response(Vec2::new(tab_width, tab_height), egui::Sense::click());
        let rect = tab_response.rect;
        
        // Chrome tab shape with slanted edges
        let mut points = Vec::new();
        let slant = 8.0; // Chrome's tab slant angle
        
        points.push(egui::Pos2::new(rect.left() - slant, rect.bottom()));
        points.push(egui::Pos2::new(rect.left() + slant, rect.top() + 4.0));
        points.push(egui::Pos2::new(rect.right() - slant, rect.top() + 4.0));
        points.push(egui::Pos2::new(rect.right() + slant, rect.bottom()));
        
        // Tab colors with transparency for inactive tabs
        let bg_color = if is_active {
            Color32::from_rgb(255, 255, 255) // Solid white for active
        } else if tab_response.hovered() {
            Color32::from_rgba_unmultiplied(210, 213, 218, 240) // Slightly darker on hover
        } else {
            Color32::from_rgba_unmultiplied(190, 194, 200, 220) // Slightly darker for inactive
        };
        
        let text_color = if is_active {
            Color32::from_rgb(32, 33, 36) // Full opacity for active
        } else {
            Color32::from_rgba_unmultiplied(95, 99, 104, 200) // Slightly transparent for inactive
        };
        
        // Draw tab background
        ui.painter().add(egui::Shape::convex_polygon(
            points.clone(),
            bg_color,
            Stroke::NONE
        ));
        
        // Draw subtle separator only for inactive tabs
        if !is_active {
            // Right separator line (very subtle)
            ui.painter().line_segment(
                [points[2], points[3]],
                Stroke::new(1.0, Color32::from_rgba_premultiplied(0, 0, 0, 20))
            );
        }
        
        // Favicon area (16x16 Chrome standard)
        let favicon_rect = egui::Rect::from_min_size(
            rect.left_top() + Vec2::new(16.0, 9.0),
            Vec2::new(16.0, 16.0)
        );
        
        // Draw favicon (placeholder)
        if tab.is_genesis_domain {
            ui.painter().circle_filled(
                favicon_rect.center(),
                8.0,
                Color32::from_rgb(34, 197, 94)
            );
            ui.painter().text(
                favicon_rect.center(),
                egui::Align2::CENTER_CENTER,
                "G",
                FontId::new(10.0, FontFamily::Proportional),
                Color32::WHITE
            );
        } else {
            ui.painter().circle_filled(
                favicon_rect.center(),
                8.0,
                Color32::from_rgb(95, 99, 104)
            );
        }
        
        // Tab title with Chrome's exact font size (dynamic truncation based on tab width)
        let max_title_width = tab_width - 70.0; // Leave space for icon and close button
        let max_chars = (max_title_width / 7.0) as usize; // Approximate char width
        let title_text = if tab.title.len() > max_chars && max_chars > 3 {
            format!("{}...", &tab.title[..max_chars.saturating_sub(3)])
        } else {
            tab.title.clone()
        };
        
        let title_pos = rect.left_top() + Vec2::new(40.0, tab_height / 2.0);
        ui.painter().text(
            title_pos,
            egui::Align2::LEFT_CENTER,
            title_text,
            FontId::new(13.0, FontFamily::Proportional), // Chrome's exact font size
            text_color
        );
        
        // Close button (overlay - doesn't affect layout)
        let close_rect = egui::Rect::from_min_size(
            rect.right_top() + Vec2::new(-28.0, 7.0),
            Vec2::new(20.0, 20.0)
        );
        
        // Use interact instead of allocate_rect to avoid affecting layout
        let close_id = ui.id().with(("tab_close", index));
        let close_response = ui.interact(close_rect, close_id, egui::Sense::click());
        
        // Draw close button hover background
        if close_response.hovered() {
            ui.painter().circle_filled(
                close_rect.center(),
                9.0,
                Color32::from_rgba_premultiplied(0, 0, 0, 40)
            );
        }
        
        // Draw X icon (always visible)
        let x_size = 6.0;
        let center = close_rect.center();
        let stroke = Stroke::new(1.5, Color32::from_rgb(95, 99, 104)); // Gray color when not hovered
        
        ui.painter().line_segment(
            [center - Vec2::new(x_size, x_size), center + Vec2::new(x_size, x_size)],
            stroke
        );
        ui.painter().line_segment(
            [center - Vec2::new(x_size, -x_size), center + Vec2::new(x_size, -x_size)],
            stroke
        );
        
        if close_response.clicked() {
            tab_actions.push(("close", index));
            return;
        }
        
        if tab_response.clicked() {
            tab_actions.push(("switch", index));
        }
    }
    
    /// Chrome-style window controls
    fn render_chrome_window_controls(&mut self, ui: &mut Ui, ctx: &egui::Context) {
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing = Vec2::new(0.0, 0.0);
            
            // Minimize button
            let min_response = ui.allocate_response(Vec2::new(40.0, 35.0), egui::Sense::click());
            if min_response.hovered() {
                ui.painter().rect_filled(
                    min_response.rect,
                    0.0,
                    Color32::from_rgba_premultiplied(0, 0, 0, 20)
                );
            }
            
            // Draw minimize line
            let min_center = min_response.rect.center();
            ui.painter().line_segment(
                [min_center - Vec2::new(5.0, 0.0), min_center + Vec2::new(5.0, 0.0)],
                Stroke::new(1.0, Color32::from_rgb(95, 99, 104))
            );
            
            if min_response.clicked() {
                ctx.send_viewport_cmd(egui::ViewportCommand::Minimized(true));
            }
            
            // Maximize button
            let max_response = ui.allocate_response(Vec2::new(40.0, 35.0), egui::Sense::click());
            if max_response.hovered() {
                ui.painter().rect_filled(
                    max_response.rect,
                    0.0,
                    Color32::from_rgba_premultiplied(0, 0, 0, 20)
                );
            }
            
            // Draw maximize square
            let max_center = max_response.rect.center();
            ui.painter().rect_stroke(
                egui::Rect::from_center_size(max_center, Vec2::new(10.0, 10.0)),
                0.0,
                Stroke::new(1.0, Color32::from_rgb(95, 99, 104))
            );
            
            if max_response.clicked() {
                ctx.send_viewport_cmd(egui::ViewportCommand::Maximized(!ctx.input(|i| i.viewport().maximized.unwrap_or(false))));
            }
            
            // Close button
            let close_response = ui.allocate_response(Vec2::new(40.0, 35.0), egui::Sense::click());
            if close_response.hovered() {
                ui.painter().rect_filled(
                    close_response.rect,
                    0.0,
                    Color32::from_rgb(232, 17, 35) // Chrome's red close button
                );
            }
            
            // Draw X
            let close_center = close_response.rect.center();
            let x_color = if close_response.hovered() {
                Color32::WHITE
            } else {
                Color32::from_rgb(95, 99, 104)
            };
            
            let x_size = 5.0;
            ui.painter().line_segment(
                [close_center - Vec2::new(x_size, x_size), close_center + Vec2::new(x_size, x_size)],
                Stroke::new(1.0, x_color)
            );
            ui.painter().line_segment(
                [close_center - Vec2::new(x_size, -x_size), close_center + Vec2::new(x_size, -x_size)],
                Stroke::new(1.0, x_color)
            );
            
            if close_response.clicked() {
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            }
        });
    }
    
    /// Chrome navigation bar with exact styling
    fn render_chrome_navigation_bar(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.add_space(8.0);
            
            // Navigation buttons with Chrome styling
            ui.scope(|ui| {
                ui.spacing_mut().item_spacing = Vec2::new(4.0, 0.0);
                
                // Back button
                let back_response = ui.allocate_response(Vec2::new(32.0, 32.0), egui::Sense::click());
                if back_response.hovered() {
                    ui.painter().circle_filled(
                        back_response.rect.center(),
                        16.0,
                        Color32::from_rgba_premultiplied(60, 64, 67, 20)
                    );
                }
                
                // Draw back arrow
                let center = back_response.rect.center();
                let arrow_color = Color32::from_rgb(95, 99, 104);
                ui.painter().line_segment(
                    [center - Vec2::new(4.0, 0.0), center + Vec2::new(4.0, 0.0)],
                    Stroke::new(2.0, arrow_color)
                );
                ui.painter().line_segment(
                    [center - Vec2::new(4.0, 0.0), center - Vec2::new(0.0, 4.0)],
                    Stroke::new(2.0, arrow_color)
                );
                ui.painter().line_segment(
                    [center - Vec2::new(4.0, 0.0), center - Vec2::new(0.0, -4.0)],
                    Stroke::new(2.0, arrow_color)
                );
                
                // Forward button
                let forward_response = ui.allocate_response(Vec2::new(32.0, 32.0), egui::Sense::click());
                if forward_response.hovered() {
                    ui.painter().circle_filled(
                        forward_response.rect.center(),
                        16.0,
                        Color32::from_rgba_premultiplied(60, 64, 67, 20)
                    );
                }
                
                // Draw forward arrow
                let center = forward_response.rect.center();
                ui.painter().line_segment(
                    [center - Vec2::new(4.0, 0.0), center + Vec2::new(4.0, 0.0)],
                    Stroke::new(2.0, arrow_color)
                );
                ui.painter().line_segment(
                    [center + Vec2::new(4.0, 0.0), center + Vec2::new(0.0, 4.0)],
                    Stroke::new(2.0, arrow_color)
                );
                ui.painter().line_segment(
                    [center + Vec2::new(4.0, 0.0), center + Vec2::new(0.0, -4.0)],
                    Stroke::new(2.0, arrow_color)
                );
                
                // Reload button
                let reload_response = ui.allocate_response(Vec2::new(32.0, 32.0), egui::Sense::click());
                if reload_response.hovered() {
                    ui.painter().circle_filled(
                        reload_response.rect.center(),
                        16.0,
                        Color32::from_rgba_premultiplied(60, 64, 67, 20)
                    );
                }
                
                // Draw reload icon (circular arrow)
                let center = reload_response.rect.center();
                ui.painter().circle_stroke(
                    center,
                    6.0,
                    Stroke::new(2.0, arrow_color)
                );
            });
            
            ui.add_space(8.0);
            
            // Chrome's omnibox with exact styling
            let url_rect = egui::Rect::from_min_size(
                ui.cursor().min,
                Vec2::new(ui.available_width() - 100.0, 32.0)
            );
            
            // Draw omnibox background
            ui.painter().rect(
                url_rect,
                16.0, // Chrome's border radius
                Color32::from_rgb(241, 243, 244),
                Stroke::NONE
            );
            
            // URL input
            ui.allocate_ui_at_rect(url_rect, |ui| {
                ui.add_space(12.0);
                ui.horizontal_centered(|ui| {
                    let url_response = ui.add_sized(
                        Vec2::new(url_rect.width() - 24.0, 28.0),
                        TextEdit::singleline(&mut self.url_input)
                            .font(FontId::new(14.0, FontFamily::Proportional))
                            .hint_text("Search Google or type a URL")
                    );
                    
                    if url_response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                        let url = self.url_input.clone();
                        self.navigate_to(&url);
                    }
                });
            });
            
            ui.add_space(8.0);
            
            // Chrome menu button (three dots)
            let menu_response = ui.allocate_response(Vec2::new(32.0, 32.0), egui::Sense::click());
            if menu_response.hovered() {
                ui.painter().circle_filled(
                    menu_response.rect.center(),
                    16.0,
                    Color32::from_rgba_premultiplied(60, 64, 67, 20)
                );
            }
            
            // Draw three dots
            let center = menu_response.rect.center();
            for i in -1..=1 {
                ui.painter().circle_filled(
                    center + Vec2::new(0.0, i as f32 * 6.0),
                    2.0,
                    Color32::from_rgb(95, 99, 104)
                );
            }
            
            if menu_response.clicked() {
                self.show_devtools = !self.show_devtools;
            }
            
            ui.add_space(8.0);
        });
    }
    
    /// Render main content area
    fn render_main_content(&mut self, ctx: &egui::Context) {
        CentralPanel::default().show(ctx, |ui| {
            if let Some(tab) = self.ui_state.get_active_tab() {
                let _content_rect = ui.available_rect_before_wrap();
                
                ui.vertical_centered(|ui| {
                    ui.add_space(50.0);
                    
                    ui.heading(
                        RichText::new(&tab.title)
                            .size(24.0)
                            .color(if tab.is_genesis_domain { 
                                Color32::from_rgb(34, 197, 94) 
                            } else { 
                                Color32::GRAY 
                            })
                    );
                    
                    ui.add_space(20.0);
                    
                    ui.label(
                        RichText::new(&tab.url)
                            .size(16.0)
                            .color(Color32::from_rgb(156, 163, 175))
                    );
                    
                    ui.add_space(40.0);
                    
                    if tab.is_genesis_domain {
                        ui.group(|ui| {
                            ui.vertical(|ui| {
                                ui.label(
                                    RichText::new("🌐 Genesis Blockchain Domain")
                                        .size(18.0)
                                        .color(Color32::from_rgb(34, 197, 94))
                                        .strong()
                                );
                                
                                ui.separator();
                                
                                ui.label("✅ Decentralized DNS resolution");
                                ui.label("✅ Censorship resistant");
                                ui.label("✅ Community governed");
                                ui.label("✅ No ICANN dependency");
                                ui.label("✅ Built on 2M+ TPS Genesis blockchain");
                                
                                ui.separator();
                                
                                ui.horizontal(|ui| {
                                    if ui.button("📖 Learn More").clicked() {
                                        self.navigate_to("genesis://docs");
                                    }
                                    if ui.button("🌐 Explore Genesis").clicked() {
                                        self.navigate_to("genesis://explorer");
                                    }
                                });
                            });
                        });
                    } else {
                        ui.group(|ui| {
                            ui.vertical(|ui| {
                                ui.label(
                                    RichText::new("🌍 Traditional Web Domain")
                                        .size(18.0)
                                        .color(Color32::GRAY)
                                );
                                
                                ui.separator();
                                
                                ui.label("🔄 Using traditional DNS fallback");
                                ui.label("⚠️ Centralized infrastructure");
                                ui.label("🔒 Subject to censorship");
                                
                                ui.separator();
                                
                                ui.horizontal(|ui| {
                                    if ui.button("🌐 Try Genesis Domains").clicked() {
                                        self.navigate_to("genesis://directory");
                                    }
                                });
                            });
                        });
                    }
                    
                    ui.add_space(40.0);
                    
                    ui.horizontal(|ui| {
                        if ui.button("🏠 Genesis Home").clicked() {
                            self.navigate_to("genesis://home");
                        }
                        if ui.button("🔍 Domain Directory").clicked() {
                            self.navigate_to("genesis://directory");
                        }
                        if ui.button("💡 Get Started").clicked() {
                            self.navigate_to("genesis://getting-started");
                        }
                    });
                });
            } else {
                ui.centered_and_justified(|ui| {
                    ui.label("No active tab");
                });
            }
        });
    }
    
    /// Render status bar
    fn render_status_bar(&mut self, ctx: &egui::Context) {
        TopBottomPanel::bottom("status_bar")
            .exact_height(22.0) // Chrome's exact status bar height
            .show(ctx, |ui| {
                ui.style_mut().visuals.panel_fill = Color32::from_rgb(241, 243, 244);
                
                ui.horizontal(|ui| {
                    ui.add_space(8.0);
                    
                    if let Some(tab) = self.ui_state.get_active_tab() {
                        if tab.is_loading {
                            ui.spinner();
                            ui.label(
                                RichText::new("Loading...")
                                    .size(11.0)
                                    .color(Color32::from_rgb(95, 99, 104))
                            );
                        } else {
                            ui.label(
                                RichText::new(&tab.url)
                                    .size(11.0)
                                    .color(Color32::from_rgb(95, 99, 104))
                            );
                        }
                        
                        ui.separator();
                        
                        if tab.is_genesis_domain {
                            ui.colored_label(Color32::from_rgb(34, 197, 94), "Genesis DNS");
                        } else {
                            ui.label("Traditional DNS");
                        }
                    }
                    
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.add_space(8.0);
                        
                        ui.label(
                            RichText::new(format!("{}%", 100))
                                .size(11.0)
                                .color(Color32::from_rgb(95, 99, 104))
                        );
                        
                        ui.separator();
                        
                        ui.label(
                            RichText::new(&*self.ui_state.genesis_node_status.borrow())
                                .size(11.0)
                                .color(Color32::from_rgb(95, 99, 104))
                        );
                    });
                });
            });
    }
    
    /// Render side panels
    fn render_side_panels(&mut self, ctx: &egui::Context) {
        if self.show_bookmarks {
            SidePanel::right("bookmarks_panel").show(ctx, |ui| {
                ui.heading("⭐ Bookmarks");
                ui.separator();
                
                let mut bookmark_actions = Vec::new();
                let mut add_bookmark_clicked = false;
                
                ScrollArea::vertical().show(ui, |ui| {
                    {
                        let bookmarks = self.ui_state.bookmarks.borrow();
                        for bookmark in bookmarks.iter() {
                            ui.horizontal(|ui| {
                                if ui.link(&bookmark.title).clicked() {
                                    bookmark_actions.push(("navigate", bookmark.url.clone()));
                                }
                                
                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                    if ui.small_button("🗑").clicked() {
                                        bookmark_actions.push(("remove", bookmark.id.clone()));
                                    }
                                });
                            });
                            
                            ui.label(
                                RichText::new(&bookmark.url)
                                    .small()
                                    .color(Color32::GRAY)
                            );
                            ui.separator();
                        }
                    }
                });
                
                ui.separator();
                if ui.button("➕ Bookmark This Page").clicked() {
                    add_bookmark_clicked = true;
                }
                
                for (action, data) in bookmark_actions {
                    match action {
                        "navigate" => {
                            self.navigate_to(&data);
                            self.show_bookmarks = false;
                        }
                        "remove" => {
                            self.ui_state.remove_bookmark(&data);
                        }
                        _ => {}
                    }
                }
                
                if add_bookmark_clicked {
                    if let Some(tab) = self.ui_state.get_active_tab() {
                        self.ui_state.add_bookmark(tab.title, tab.url, None);
                    }
                }
            });
        }
        
        if self.show_downloads {
            SidePanel::right("downloads_panel").show(ctx, |ui| {
                ui.heading("⬇ Downloads");
                ui.separator();
                
                ui.centered_and_justified(|ui| {
                    ui.label("No downloads yet");
                });
            });
        }
        
        if self.show_devtools {
            SidePanel::right("devtools_panel").show(ctx, |ui| {
                ui.heading("🔧 Developer Tools");
                ui.separator();
                
                ui.horizontal(|ui| {
                    ui.selectable_label(true, "Console");
                    ui.selectable_label(false, "Network");
                    ui.selectable_label(false, "Elements");
                });
                
                ui.separator();
                
                ScrollArea::vertical().show(ui, |ui| {
                    ui.monospace("genesis-browser: console initialized");
                    ui.monospace("dns-resolver: genesis domains ready");
                    ui.monospace("servo-engine: webrender active");
                });
            });
        }
    }
    
    /// Navigate to a URL
    fn navigate_to(&mut self, url: &str) {
        info!("🔍 Modern UI navigating to: {}", url);
        
        let active_index = self.ui_state.active_tab_index.get();
        self.ui_state.update_tab(active_index, Some(url.to_string()), Some(url.to_string()), Some(true));
        self.url_input = url.to_string();
        
        self.ui_state.add_to_history(url.to_string(), url.to_string());
        
        self.ui_state.update_tab(active_index, None, None, Some(false));
    }
}