// Tab Renderer - Single Responsibility: Tab bar rendering
//
// Bu renderer sadece tab görselleştirmesi ile ilgili işlemleri yönetir:
// - Tab şekil ve görünümü
// - Tab animasyonları
// - Tab close button'ları
// - Tab scroll handling

use crate::managers::{TabManager, ThemeManager, PerformanceMonitor};
use crate::enhanced_browser::BrowserTab;
use egui::{Ui, Rect, Vec2, Color32, Stroke, Response, Sense};

pub struct TabRenderer {
    // Render state
    scroll_offset: f32,
    drag_tab_index: Option<usize>,
    drag_start_pos: Option<Vec2>,
    hover_tab_index: Option<usize>,
    
    // Animation state
    animation_time: f32,
    
    // Configuration
    tab_min_width: f32,
    tab_max_width: f32,
    tab_height: f32,
    close_button_size: f32,
}

#[derive(Clone, Debug)]
pub struct TabRenderResponse {
    pub switch_to_tab: Option<usize>,
    pub close_tab: Option<usize>,
    pub new_tab_clicked: bool,
    pub reorder_tabs: Option<(usize, usize)>, // from, to
    pub context_menu_tab: Option<usize>,
}

impl Default for TabRenderer {
    fn default() -> Self {
        Self {
            scroll_offset: 0.0,
            drag_tab_index: None,
            drag_start_pos: None,
            hover_tab_index: None,
            animation_time: 0.0,
            tab_min_width: 120.0,
            tab_max_width: 240.0,
            tab_height: 35.0,
            close_button_size: 16.0,
        }
    }
}

impl TabRenderer {
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Tab bar'ı render et
    pub fn render_tab_bar(
        &mut self, 
        ui: &mut Ui, 
        tab_manager: &TabManager,
        theme_manager: &ThemeManager,
        _performance_monitor: &PerformanceMonitor,
    ) -> TabRenderResponse {
        let mut response = TabRenderResponse {
            switch_to_tab: None,
            close_tab: None,
            new_tab_clicked: false,
            reorder_tabs: None,
            context_menu_tab: None,
        };
        
        // Update animation time
        self.animation_time += ui.ctx().input(|i| i.stable_dt);
        
        let available_rect = ui.available_rect_before_wrap();
        let theme = theme_manager.get_current_theme();
        
        // Tab area background
        ui.painter().rect_filled(
            available_rect,
            theme_manager.get_rounding(),
            theme.colors.tab_area_fill,
        );
        
        // Calculate tab dimensions
        let tabs = tab_manager.get_tabs();
        let tab_count = tabs.len();
        let new_tab_button_width = 36.0;
        let controls_width = 120.0; // Window controls
        
        let available_width = available_rect.width() - new_tab_button_width - controls_width;
        let tab_width = self.calculate_tab_width(tab_count, available_width);
        
        // Render tabs
        let mut current_x = available_rect.left() + self.scroll_offset;
        
        for (index, tab) in tabs.iter().enumerate() {
            let tab_rect = Rect::from_min_size(
                [current_x, available_rect.top()].into(),
                [tab_width, self.tab_height].into(),
            );
            
            if tab_rect.right() > available_rect.left() && tab_rect.left() < available_rect.right() - new_tab_button_width - controls_width {
                let tab_response = self.render_single_tab(
                    ui, 
                    tab, 
                    tab_rect, 
                    index, 
                    tab_manager.get_active_tab_index() == index,
                    theme_manager
                );
                
                // Handle tab interactions
                if tab_response.clicked() {
                    response.switch_to_tab = Some(index);
                } else if tab_response.clicked_by(egui::PointerButton::Middle) {
                    response.close_tab = Some(index);
                } else if tab_response.secondary_clicked() {
                    response.context_menu_tab = Some(index);
                }
                
                // Check close button click
                if let Some(close_response) = self.render_tab_close_button(ui, tab_rect, theme_manager) {
                    if close_response.clicked() {
                        response.close_tab = Some(index);
                    }
                }
            }
            
            current_x += tab_width;
        }
        
        // Render new tab button
        let new_tab_rect = Rect::from_min_size(
            [current_x.max(available_rect.left()), available_rect.top()].into(),
            [new_tab_button_width, self.tab_height].into(),
        );
        
        if let Some(new_tab_response) = self.render_new_tab_button(ui, new_tab_rect, theme_manager) {
            if new_tab_response.clicked() {
                response.new_tab_clicked = true;
            }
        }
        
        // Handle scrolling if needed
        self.handle_tab_scrolling(ui, available_width, tab_count, tab_width);
        
        response
    }
    
    fn render_single_tab(
        &mut self,
        ui: &mut Ui,
        tab: &BrowserTab,
        tab_rect: Rect,
        _tab_index: usize,
        is_active: bool,
        theme_manager: &ThemeManager,
    ) -> Response {
        let theme = theme_manager.get_current_theme();
        
        // Tab background color
        let bg_color = if is_active {
            theme.colors.active_tab_fill
        } else {
            theme.colors.inactive_tab_fill
        };
        
        // Tab shape with rounded corners (Chrome-style)
        let rounding = egui::Rounding {
            nw: theme.spacing.border_radius,
            ne: theme.spacing.border_radius,
            sw: 0.0,
            se: 0.0,
        };
        
        ui.painter().rect_filled(tab_rect, rounding, bg_color);
        
        // Tab border (only for active tab)
        if is_active {
            let border_stroke = Stroke::new(1.0, theme.colors.genesis_accent_color);
            ui.painter().rect_stroke(tab_rect, rounding, border_stroke);
        }
        
        // Favicon (if available)
        let mut text_area = tab_rect;
        text_area.min.x += 12.0; // Left padding
        
        if let Some(_favicon_url) = &tab.favicon {
            // TODO: Render favicon
            text_area.min.x += 20.0; // Space for favicon
        }
        
        // Loading indicator
        if tab.is_loading {
            let spinner_center = Vec2::new(text_area.min.x - 10.0, tab_rect.center().y);
            self.render_loading_spinner(ui, spinner_center, theme_manager);
        }
        
        // Tab title
        text_area.max.x -= self.close_button_size + 8.0; // Space for close button
        
        let title_text = if tab.title.len() > 20 {
            format!("{}...", &tab.title[..17])
        } else {
            tab.title.clone()
        };
        
        let text_color = if is_active {
            theme.colors.tab_text_color
        } else {
            Color32::from_gray(140)
        };
        
        ui.painter().text(
            text_area.center(),
            egui::Align2::CENTER_CENTER,
            title_text,
            theme_manager.get_font_id(crate::managers::theme_manager::FontElement::Tab),
            text_color,
        );
        
        // Genesis domain indicator
        if tab.is_genesis_domain {
            let indicator_rect = Rect::from_center_size(
                [text_area.max.x - 4.0, text_area.min.y + 4.0].into(),
                [6.0, 6.0].into(),
            );
            ui.painter().circle_filled(
                indicator_rect.center(),
                3.0,
                theme.colors.genesis_accent_color,
            );
        }
        
        // Progress bar (if loading)
        if tab.is_loading && tab.load_progress > 0.0 {
            let progress_rect = Rect::from_min_size(
                [tab_rect.min.x, tab_rect.max.y - 2.0].into(),
                [tab_rect.width() * tab.load_progress, 2.0].into(),
            );
            ui.painter().rect_filled(
                progress_rect,
                egui::Rounding::ZERO,
                theme.colors.progress_bar_fill,
            );
        }
        
        // Create interactive area
        ui.allocate_rect(tab_rect, Sense::click())
    }
    
    fn render_tab_close_button(
        &self,
        ui: &mut Ui,
        tab_rect: Rect,
        theme_manager: &ThemeManager,
    ) -> Option<Response> {
        let theme = theme_manager.get_current_theme();
        
        let close_button_rect = Rect::from_center_size(
            [tab_rect.max.x - self.close_button_size, tab_rect.center().y].into(),
            [self.close_button_size, self.close_button_size].into(),
        );
        
        let response = ui.allocate_rect(close_button_rect, Sense::click());
        
        // Close button background on hover
        if response.hovered() {
            ui.painter().circle_filled(
                close_button_rect.center(),
                self.close_button_size / 2.0,
                Color32::from_gray(200),
            );
        }
        
        // Close button X
        let x_size = 8.0;
        let center = close_button_rect.center();
        let color = theme.colors.tab_close_button_color;
        
        ui.painter().line_segment(
            [
                [center.x - x_size / 2.0, center.y - x_size / 2.0].into(),
                [center.x + x_size / 2.0, center.y + x_size / 2.0].into(),
            ],
            Stroke::new(2.0, color),
        );
        
        ui.painter().line_segment(
            [
                [center.x + x_size / 2.0, center.y - x_size / 2.0].into(),
                [center.x - x_size / 2.0, center.y + x_size / 2.0].into(),
            ],
            Stroke::new(2.0, color),
        );
        
        Some(response)
    }
    
    fn render_new_tab_button(
        &self,
        ui: &mut Ui,
        button_rect: Rect,
        theme_manager: &ThemeManager,
    ) -> Option<Response> {
        let theme = theme_manager.get_current_theme();
        let response = ui.allocate_rect(button_rect, Sense::click());
        
        // Button background
        let bg_color = if response.hovered() {
            theme.colors.button_hover_fill
        } else {
            theme.colors.button_fill
        };
        
        ui.painter().rect_filled(button_rect, theme_manager.get_rounding(), bg_color);
        
        // Plus icon
        let center = button_rect.center();
        let size = 12.0;
        let color = theme.colors.button_text;
        
        // Horizontal line
        ui.painter().line_segment(
            [
                [center.x - size / 2.0, center.y].into(),
                [center.x + size / 2.0, center.y].into(),
            ],
            Stroke::new(2.0, color),
        );
        
        // Vertical line
        ui.painter().line_segment(
            [
                [center.x, center.y - size / 2.0].into(),
                [center.x, center.y + size / 2.0].into(),
            ],
            Stroke::new(2.0, color),
        );
        
        Some(response)
    }
    
    fn render_loading_spinner(&self, ui: &mut Ui, center: Vec2, theme_manager: &ThemeManager) {
        let theme = theme_manager.get_current_theme();
        let radius = 6.0;
        let thickness = 2.0;
        
        // Spinning animation
        let angle = self.animation_time * 6.0; // 6 rad/s
        
        // Draw partial circle
        let n_points = 20;
        let arc_length = std::f32::consts::PI * 1.5; // 3/4 circle
        
        for i in 0..n_points {
            let t1 = (i as f32 / n_points as f32) * arc_length + angle;
            let t2 = ((i + 1) as f32 / n_points as f32) * arc_length + angle;
            
            let p1 = egui::Pos2::new(center.x + radius * t1.cos(), center.y + radius * t1.sin());
            let p2 = egui::Pos2::new(center.x + radius * t2.cos(), center.y + radius * t2.sin());
            
            let alpha = (i as f32 / n_points as f32).powf(2.0);
            let mut color = theme.colors.loading_spinner_color;
            color = Color32::from_rgba_unmultiplied(color.r(), color.g(), color.b(), (255.0 * alpha) as u8);
            
            ui.painter().line_segment([p1, p2], Stroke::new(thickness, color));
        }
    }
    
    fn calculate_tab_width(&self, tab_count: usize, available_width: f32) -> f32 {
        if tab_count == 0 {
            return self.tab_max_width;
        }
        
        let total_width = available_width;
        let width_per_tab = total_width / tab_count as f32;
        
        width_per_tab.clamp(self.tab_min_width, self.tab_max_width)
    }
    
    fn handle_tab_scrolling(&mut self, ui: &mut Ui, available_width: f32, tab_count: usize, tab_width: f32) {
        let total_tabs_width = tab_count as f32 * tab_width;
        
        if total_tabs_width > available_width {
            // Enable scrolling
            let max_scroll = total_tabs_width - available_width;
            
            // Mouse wheel scrolling
            ui.input(|i| {
                if i.scroll_delta.x != 0.0 {
                    self.scroll_offset = (self.scroll_offset - i.scroll_delta.x * 20.0)
                        .clamp(-max_scroll, 0.0);
                }
            });
        } else {
            self.scroll_offset = 0.0;
        }
    }
    
    // === Getters/Setters ===
    
    pub fn set_tab_dimensions(&mut self, min_width: f32, max_width: f32, height: f32) {
        self.tab_min_width = min_width;
        self.tab_max_width = max_width;
        self.tab_height = height;
    }
    
    pub fn get_scroll_offset(&self) -> f32 {
        self.scroll_offset
    }
    
    pub fn set_scroll_offset(&mut self, offset: f32) {
        self.scroll_offset = offset;
    }
}