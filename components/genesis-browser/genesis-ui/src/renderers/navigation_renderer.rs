// Navigation Renderer - Single Responsibility: Navigation bar rendering
//
// Bu renderer sadece navigasyon çubuğu görselleştirmesi ile ilgili işlemleri yönetir:
// - Address bar rendering
// - Navigation buttons (back/forward/reload)
// - URL suggestions
// - Security indicators

use crate::managers::{NavigationManager, ThemeManager};
use egui::{Ui, Rect, Color32, Stroke, Response, Sense, TextEdit};

pub struct NavigationRenderer {
    // UI state
    url_edit_focused: bool,
    show_suggestions: bool,
    suggestion_index: Option<usize>,
    
    // Animation state
    security_pulse: f32,
    
    // Configuration
    button_size: f32,
    address_bar_height: f32,
}

#[derive(Clone, Debug)]
pub struct NavigationRenderResponse {
    pub navigate_to: Option<String>,
    pub go_back: bool,
    pub go_forward: bool,
    pub reload: bool,
    pub stop_loading: bool,
    pub home: bool,
    pub show_suggestions: bool,
    pub url_changed: bool,
}

impl Default for NavigationRenderer {
    fn default() -> Self {
        Self {
            url_edit_focused: false,
            show_suggestions: false,
            suggestion_index: None,
            security_pulse: 0.0,
            button_size: 32.0,
            address_bar_height: 36.0,
        }
    }
}

impl NavigationRenderer {
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Navigation bar'ı render et
    pub fn render_navigation_bar(
        &mut self,
        ui: &mut Ui,
        navigation_manager: &NavigationManager,
        theme_manager: &ThemeManager,
        is_loading: bool,
    ) -> NavigationRenderResponse {
        let mut response = NavigationRenderResponse {
            navigate_to: None,
            go_back: false,
            go_forward: false,
            reload: false,
            stop_loading: false,
            home: false,
            show_suggestions: false,
            url_changed: false,
        };
        
        let available_rect = ui.available_rect_before_wrap();
        let theme = theme_manager.get_current_theme();
        
        // Navigation bar background
        ui.painter().rect_filled(
            available_rect,
            theme_manager.get_rounding(),
            theme.colors.panel_fill,
        );
        
        // Layout calculations
        let button_spacing = 4.0;
        let side_padding = 8.0;
        let button_area_width = (self.button_size + button_spacing) * 4.0; // 4 buttons
        
        let mut current_x = available_rect.left() + side_padding;
        
        // Back button
        if let Some(back_response) = self.render_navigation_button(
            ui,
            Rect::from_min_size(
                [current_x, available_rect.top() + 2.0].into(),
                [self.button_size, self.button_size].into(),
            ),
            NavigationButton::Back,
            navigation_manager.can_go_back(),
            theme_manager,
        ) {
            if back_response.clicked() && navigation_manager.can_go_back() {
                response.go_back = true;
            }
        }
        current_x += self.button_size + button_spacing;
        
        // Forward button
        if let Some(forward_response) = self.render_navigation_button(
            ui,
            Rect::from_min_size(
                [current_x, available_rect.top() + 2.0].into(),
                [self.button_size, self.button_size].into(),
            ),
            NavigationButton::Forward,
            false, // TODO: Add forward navigation support
            theme_manager,
        ) {
            if forward_response.clicked() {
                response.go_forward = true;
            }
        }
        current_x += self.button_size + button_spacing;
        
        // Reload/Stop button
        let reload_stop_button = if is_loading {
            NavigationButton::Stop
        } else {
            NavigationButton::Reload
        };
        
        if let Some(reload_response) = self.render_navigation_button(
            ui,
            Rect::from_min_size(
                [current_x, available_rect.top() + 2.0].into(),
                [self.button_size, self.button_size].into(),
            ),
            reload_stop_button,
            true,
            theme_manager,
        ) {
            if reload_response.clicked() {
                if is_loading {
                    response.stop_loading = true;
                } else {
                    response.reload = true;
                }
            }
        }
        current_x += self.button_size + button_spacing;
        
        // Home button
        if let Some(home_response) = self.render_navigation_button(
            ui,
            Rect::from_min_size(
                [current_x, available_rect.top() + 2.0].into(),
                [self.button_size, self.button_size].into(),
            ),
            NavigationButton::Home,
            true,
            theme_manager,
        ) {
            if home_response.clicked() {
                response.home = true;
            }
        }
        current_x += self.button_size + button_spacing;
        
        // Address bar
        let address_bar_width = available_rect.width() - button_area_width - side_padding * 3.0 - 100.0;
        let address_bar_rect = Rect::from_min_size(
            [current_x, available_rect.top() + 2.0].into(),
            [address_bar_width, self.address_bar_height].into(),
        );
        
        let (address_response, url_changed) = self.render_address_bar(
            ui,
            address_bar_rect,
            navigation_manager,
            theme_manager,
        );
        
        if url_changed {
            response.url_changed = true;
        }
        
        if let Some(url) = address_response {
            response.navigate_to = Some(url);
        }
        
        // URL suggestions (if needed)
        if self.show_suggestions {
            response.show_suggestions = true;
            self.render_url_suggestions(ui, address_bar_rect, navigation_manager, theme_manager);
        }
        
        // Security indicator
        current_x += address_bar_width + button_spacing;
        self.render_security_indicator(
            ui,
            Rect::from_min_size(
                [current_x, available_rect.top() + 2.0].into(),
                [80.0, self.address_bar_height].into(),
            ),
            navigation_manager,
            theme_manager,
        );
        
        response
    }
    
    fn render_navigation_button(
        &self,
        ui: &mut Ui,
        button_rect: Rect,
        button_type: NavigationButton,
        enabled: bool,
        theme_manager: &ThemeManager,
    ) -> Option<Response> {
        let theme = theme_manager.get_current_theme();
        
        let response = ui.allocate_rect(button_rect, Sense::click());
        
        // Button background
        let bg_color = if !enabled {
            Color32::from_gray(240)
        } else if response.clicked() {
            theme.colors.button_active_fill
        } else if response.hovered() {
            theme.colors.button_hover_fill
        } else {
            theme.colors.button_fill
        };
        
        ui.painter().rect_filled(button_rect, theme_manager.get_rounding(), bg_color);
        
        // Button border
        if response.hovered() && enabled {
            ui.painter().rect_stroke(
                button_rect,
                theme_manager.get_rounding(),
                Stroke::new(1.0, theme.colors.border_color),
            );
        }
        
        // Button icon
        let icon_color = if enabled {
            theme.colors.button_text
        } else {
            Color32::from_gray(160)
        };
        
        self.render_button_icon(ui, button_rect.center(), button_type, icon_color);
        
        if enabled {
            Some(response)
        } else {
            None
        }
    }
    
    fn render_button_icon(&self, ui: &mut Ui, center: egui::Pos2, button_type: NavigationButton, color: Color32) {
        let size = 12.0;
        let stroke = Stroke::new(2.0, color);
        
        match button_type {
            NavigationButton::Back => {
                // Left arrow
                let points = vec![
                    egui::Pos2::new(center.x + size / 3.0, center.y - size / 2.0),
                    egui::Pos2::new(center.x - size / 3.0, center.y),
                    egui::Pos2::new(center.x + size / 3.0, center.y + size / 2.0),
                ];
                
                for i in 0..points.len() - 1 {
                    ui.painter().line_segment([points[i], points[i + 1]], stroke);
                }
            },
            
            NavigationButton::Forward => {
                // Right arrow
                let points = vec![
                    egui::Pos2::new(center.x - size / 3.0, center.y - size / 2.0),
                    egui::Pos2::new(center.x + size / 3.0, center.y),
                    egui::Pos2::new(center.x - size / 3.0, center.y + size / 2.0),
                ];
                
                for i in 0..points.len() - 1 {
                    ui.painter().line_segment([points[i], points[i + 1]], stroke);
                }
            },
            
            NavigationButton::Reload => {
                // Circular arrow
                let radius = size / 2.0;
                let n_points = 12;
                
                for i in 0..n_points {
                    let angle1 = (i as f32 / n_points as f32) * std::f32::consts::PI * 1.5;
                    let angle2 = ((i + 1) as f32 / n_points as f32) * std::f32::consts::PI * 1.5;
                    
                    let p1 = center + egui::Vec2::new(radius * angle1.cos(), radius * angle1.sin());
                    let p2 = center + egui::Vec2::new(radius * angle2.cos(), radius * angle2.sin());
                    
                    ui.painter().line_segment([p1, p2], stroke);
                }
                
                // Arrow tip
                let arrow_pos = center + egui::Vec2::new(radius, 0.0);
                ui.painter().line_segment(
                    [arrow_pos, arrow_pos + egui::Vec2::new(-4.0, -3.0)],
                    stroke,
                );
                ui.painter().line_segment(
                    [arrow_pos, arrow_pos + egui::Vec2::new(-4.0, 3.0)],
                    stroke,
                );
            },
            
            NavigationButton::Stop => {
                // X mark
                ui.painter().line_segment(
                    [
                        egui::Pos2::new(center.x - size / 2.0, center.y - size / 2.0),
                        egui::Pos2::new(center.x + size / 2.0, center.y + size / 2.0),
                    ],
                    stroke,
                );
                ui.painter().line_segment(
                    [
                        egui::Pos2::new(center.x + size / 2.0, center.y - size / 2.0),
                        egui::Pos2::new(center.x - size / 2.0, center.y + size / 2.0),
                    ],
                    stroke,
                );
            },
            
            NavigationButton::Home => {
                // House icon
                let roof_points = vec![
                    egui::Pos2::new(center.x - size / 2.0, center.y),
                    egui::Pos2::new(center.x, center.y - size / 2.0),
                    egui::Pos2::new(center.x + size / 2.0, center.y),
                ];
                
                for i in 0..roof_points.len() - 1 {
                    ui.painter().line_segment([roof_points[i], roof_points[i + 1]], stroke);
                }
                
                // House base
                ui.painter().rect_stroke(
                    Rect::from_center_size(
                        egui::Pos2::new(center.x, center.y + size / 4.0),
                        egui::Vec2::new(size * 0.6, size / 2.0),
                    ),
                    egui::Rounding::ZERO,
                    stroke,
                );
            },
        }
    }
    
    fn render_address_bar(
        &mut self,
        ui: &mut Ui,
        address_rect: Rect,
        navigation_manager: &NavigationManager,
        theme_manager: &ThemeManager,
    ) -> (Option<String>, bool) {
        let theme = theme_manager.get_current_theme();
        
        // Address bar background
        ui.painter().rect_filled(
            address_rect,
            theme_manager.get_rounding(),
            theme.colors.address_bar_fill,
        );
        
        // Address bar border
        let border_color = if self.url_edit_focused {
            theme.colors.focus_ring_color
        } else {
            theme.colors.address_bar_border
        };
        
        ui.painter().rect_stroke(
            address_rect,
            theme_manager.get_rounding(),
            Stroke::new(1.0, border_color),
        );
        
        // URL text edit
        let mut current_url = navigation_manager.get_current_url().clone();
        let original_url = current_url.clone();
        let mut should_navigate = false;
        
        let text_rect = address_rect.shrink(8.0);
        ui.allocate_ui_at_rect(text_rect, |ui| {
            ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
                let text_edit = TextEdit::singleline(&mut current_url)
                    .font(theme_manager.get_font_id(crate::managers::theme_manager::FontElement::AddressBar))
                    .text_color(theme.colors.address_bar_text)
                    .desired_width(text_rect.width() - 40.0);
                
                let response = ui.add(text_edit);
                self.url_edit_focused = response.has_focus();
                
                // Handle URL submission
                if response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                    should_navigate = true;
                }
            });
        });
        
        let url_changed = current_url != original_url;
        
        if should_navigate {
            (Some(current_url), true)
        } else if url_changed {
            (None, true)
        } else {
            (None, false)
        }
    }
    
    fn render_url_suggestions(
        &self,
        ui: &mut Ui,
        address_rect: Rect,
        navigation_manager: &NavigationManager,
        theme_manager: &ThemeManager,
    ) {
        let theme = theme_manager.get_current_theme();
        
        // Get suggestions
        let current_url = navigation_manager.get_current_url();
        let suggestions = navigation_manager.get_url_suggestions(current_url);
        
        if suggestions.is_empty() {
            return;
        }
        
        // Suggestions dropdown
        let dropdown_rect = Rect::from_min_size(
            [address_rect.left(), address_rect.bottom()].into(),
            [address_rect.width(), (suggestions.len() as f32 * 30.0).min(200.0)].into(),
        );
        
        ui.painter().rect_filled(
            dropdown_rect,
            theme_manager.get_rounding(),
            theme.colors.panel_fill,
        );
        
        ui.painter().rect_stroke(
            dropdown_rect,
            theme_manager.get_rounding(),
            Stroke::new(1.0, theme.colors.border_color),
        );
        
        // Render suggestions
        let mut suggestion_y = dropdown_rect.top();
        for (index, suggestion) in suggestions.iter().enumerate() {
            let suggestion_rect = Rect::from_min_size(
                [dropdown_rect.left(), suggestion_y].into(),
                [dropdown_rect.width(), 30.0].into(),
            );
            
            // Highlight selected suggestion
            if Some(index) == self.suggestion_index {
                ui.painter().rect_filled(
                    suggestion_rect,
                    egui::Rounding::ZERO,
                    theme.colors.button_hover_fill,
                );
            }
            
            // Suggestion text
            ui.painter().text(
                egui::Pos2::new(suggestion_rect.left() + 12.0, suggestion_rect.center().y),
                egui::Align2::LEFT_CENTER,
                suggestion,
                theme_manager.get_font_id(crate::managers::theme_manager::FontElement::UI),
                theme.colors.address_bar_text,
            );
            
            suggestion_y += 30.0;
        }
    }
    
    fn render_security_indicator(
        &mut self,
        ui: &mut Ui,
        indicator_rect: Rect,
        navigation_manager: &NavigationManager,
        theme_manager: &ThemeManager,
    ) {
        let theme = theme_manager.get_current_theme();
        let is_secure = navigation_manager.is_secure_context();
        let is_genesis = navigation_manager.is_genesis_domain(navigation_manager.get_current_url());
        
        // Update pulse animation
        self.security_pulse += ui.ctx().input(|i| i.stable_dt) * 3.0;
        
        let (icon_text, icon_color) = if is_genesis {
            ("🌐", theme.colors.genesis_accent_color)
        } else if is_secure {
            ("🔒", Color32::from_rgb(34, 139, 34))
        } else {
            ("⚠", Color32::from_rgb(255, 140, 0))
        };
        
        // Render security icon
        ui.painter().text(
            indicator_rect.center(),
            egui::Align2::CENTER_CENTER,
            icon_text,
            theme_manager.get_font_id(crate::managers::theme_manager::FontElement::UI),
            icon_color,
        );
        
        // Genesis domains get a special pulse effect
        if is_genesis {
            let pulse_strength = (self.security_pulse.sin() * 0.3 + 0.7).max(0.4);
            let pulse_color = Color32::from_rgba_unmultiplied(
                theme.colors.genesis_accent_color.r(),
                theme.colors.genesis_accent_color.g(),
                theme.colors.genesis_accent_color.b(),
                (50.0 * pulse_strength) as u8,
            );
            
            ui.painter().circle_filled(
                indicator_rect.center(),
                12.0,
                pulse_color,
            );
        }
    }
    
    // === Getters/Setters ===
    
    pub fn set_button_size(&mut self, size: f32) {
        self.button_size = size;
    }
    
    pub fn set_address_bar_height(&mut self, height: f32) {
        self.address_bar_height = height;
    }
    
    pub fn set_show_suggestions(&mut self, show: bool) {
        self.show_suggestions = show;
    }
    
    pub fn set_suggestion_index(&mut self, index: Option<usize>) {
        self.suggestion_index = index;
    }
}

#[derive(Clone, Debug, PartialEq)]
enum NavigationButton {
    Back,
    Forward,
    Reload,
    Stop,
    Home,
}