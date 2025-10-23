// Status Renderer - Single Responsibility: Status bar rendering
//
// Bu renderer sadece status bar görselleştirmesi ile ilgili işlemleri yönetir:
// - Load progress indication
// - Page loading status
// - Genesis network status
// - Connection indicators
// - Performance metrics display

use crate::managers::{ThemeManager, PerformanceMonitor};
use egui::{Ui, Rect, Color32, Stroke, RichText};

pub struct StatusRenderer {
    // Status state
    last_status_message: String,
    status_message_time: f32,
    
    // Animation state
    loading_animation: f32,
    network_pulse: f32,
    
    // Configuration
    status_bar_height: f32,
    show_performance_metrics: bool,
    show_detailed_status: bool,
}

#[derive(Clone, Debug)]
pub struct StatusInfo {
    pub is_loading: bool,
    pub load_progress: f32,
    pub page_title: String,
    pub current_url: String,
    pub is_genesis_domain: bool,
    pub genesis_connected: bool,
    pub security_state: SecurityState,
    pub error_message: Option<String>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum SecurityState {
    Secure,
    Insecure,
    Genesis,
    Mixed,
    Unknown,
}

impl Default for StatusRenderer {
    fn default() -> Self {
        Self {
            last_status_message: String::new(),
            status_message_time: 0.0,
            loading_animation: 0.0,
            network_pulse: 0.0,
            status_bar_height: 24.0,
            show_performance_metrics: false,
            show_detailed_status: true,
        }
    }
}

impl StatusRenderer {
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Status bar'ı render et
    pub fn render_status_bar(
        &mut self,
        ui: &mut Ui,
        status_info: &StatusInfo,
        theme_manager: &ThemeManager,
        performance_monitor: Option<&PerformanceMonitor>,
    ) {
        let available_rect = ui.available_rect_before_wrap();
        let theme = theme_manager.get_current_theme();
        
        // Update animations
        let dt = ui.ctx().input(|i| i.stable_dt);
        self.loading_animation += dt * 2.0;
        self.network_pulse += dt * 3.0;
        self.status_message_time += dt;
        
        // Status bar background
        ui.painter().rect_filled(
            available_rect,
            egui::Rounding::ZERO,
            theme.colors.panel_fill,
        );
        
        // Top border
        ui.painter().line_segment(
            [available_rect.left_top(), available_rect.right_top()],
            Stroke::new(1.0, theme.colors.border_color),
        );
        
        // Layout areas
        let left_area_width = 300.0;
        let right_area_width = if self.show_performance_metrics { 200.0 } else { 100.0 };
        let center_area_width = available_rect.width() - left_area_width - right_area_width;
        
        // Left area: Page status
        let left_rect = Rect::from_min_size(
            available_rect.min,
            [left_area_width, self.status_bar_height].into(),
        );
        self.render_page_status(ui, left_rect, status_info, theme_manager);
        
        // Center area: Loading progress
        let center_rect = Rect::from_min_size(
            [available_rect.min.x + left_area_width, available_rect.min.y].into(),
            [center_area_width, self.status_bar_height].into(),
        );
        self.render_loading_progress(ui, center_rect, status_info, theme_manager);
        
        // Right area: Network status and performance
        let right_rect = Rect::from_min_size(
            [available_rect.min.x + left_area_width + center_area_width, available_rect.min.y].into(),
            [right_area_width, self.status_bar_height].into(),
        );
        self.render_network_and_performance(ui, right_rect, status_info, theme_manager, performance_monitor);
    }
    
    fn render_page_status(
        &mut self,
        ui: &mut Ui,
        status_rect: Rect,
        status_info: &StatusInfo,
        theme_manager: &ThemeManager,
    ) {
        let theme = theme_manager.get_current_theme();
        
        ui.allocate_ui_at_rect(status_rect, |ui| {
            ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
                ui.add_space(8.0);
                
                // Security indicator
                let (security_icon, security_color) = match status_info.security_state {
                    SecurityState::Secure => ("🔒", Color32::from_rgb(34, 139, 34)),
                    SecurityState::Genesis => ("🌐", theme.colors.genesis_accent_color),
                    SecurityState::Insecure => ("⚠", Color32::from_rgb(255, 140, 0)),
                    SecurityState::Mixed => ("⚠", Color32::from_rgb(255, 69, 0)),
                    SecurityState::Unknown => ("❓", Color32::from_gray(120)),
                };
                
                ui.label(RichText::new(security_icon).color(security_color));
                
                // Status message
                let status_message = if let Some(error) = &status_info.error_message {
                    error.clone()
                } else if status_info.is_loading {
                    if status_info.is_genesis_domain {
                        "Loading Genesis page...".to_string()
                    } else {
                        "Loading...".to_string()
                    }
                } else {
                    "Ready".to_string()
                };
                
                // Update last status message
                if status_message != self.last_status_message {
                    self.last_status_message = status_message.clone();
                    self.status_message_time = 0.0;
                }
                
                ui.label(RichText::new(status_message)
                    .size(11.0)
                    .color(theme.colors.status_text_color));
                
                // Genesis domain indicator
                if status_info.is_genesis_domain {
                    ui.add_space(8.0);
                    ui.label(RichText::new("GENESIS")
                        .size(9.0)
                        .strong()
                        .color(theme.colors.genesis_accent_color));
                }
                
                // URL preview (truncated)
                if self.show_detailed_status && !status_info.current_url.is_empty() {
                    ui.add_space(8.0);
                    ui.separator();
                    ui.add_space(8.0);
                    
                    let display_url = if status_info.current_url.len() > 40 {
                        format!("{}...", &status_info.current_url[..37])
                    } else {
                        status_info.current_url.clone()
                    };
                    
                    ui.label(RichText::new(display_url)
                        .size(10.0)
                        .color(Color32::from_gray(120)));
                }
            });
        });
    }
    
    fn render_loading_progress(
        &mut self,
        ui: &mut Ui,
        progress_rect: Rect,
        status_info: &StatusInfo,
        theme_manager: &ThemeManager,
    ) {
        let theme = theme_manager.get_current_theme();
        
        if status_info.is_loading && status_info.load_progress > 0.0 {
            // Progress bar background
            ui.painter().rect_filled(
                progress_rect.shrink(2.0),
                egui::Rounding::same(2.0),
                theme.colors.progress_bar_bg,
            );
            
            // Progress bar fill
            let progress_width = progress_rect.width() * status_info.load_progress;
            let progress_fill_rect = Rect::from_min_size(
                progress_rect.min,
                [progress_width, progress_rect.height()].into(),
            ).shrink(2.0);
            
            // Animated progress color for Genesis domains
            let progress_color = if status_info.is_genesis_domain {
                let pulse = (self.network_pulse.sin() * 0.3 + 0.7).clamp(0.4, 1.0);
                Color32::from_rgba_unmultiplied(
                    theme.colors.genesis_accent_color.r(),
                    theme.colors.genesis_accent_color.g(),
                    theme.colors.genesis_accent_color.b(),
                    (255.0 * pulse) as u8,
                )
            } else {
                theme.colors.progress_bar_fill
            };
            
            ui.painter().rect_filled(
                progress_fill_rect,
                egui::Rounding::same(2.0),
                progress_color,
            );
            
            // Progress text
            ui.allocate_ui_at_rect(progress_rect, |ui| {
                ui.centered_and_justified(|ui| {
                    ui.label(RichText::new(format!("{:.0}%", status_info.load_progress * 100.0))
                        .size(10.0)
                        .color(theme.colors.status_text_color));
                });
            });
        } else if status_info.is_loading {
            // Indeterminate progress (spinning dots)
            self.render_loading_spinner(ui, progress_rect.center(), theme_manager);
        }
    }
    
    fn render_loading_spinner(&self, ui: &mut Ui, center: egui::Pos2, theme_manager: &ThemeManager) {
        let theme = theme_manager.get_current_theme();
        let radius = 8.0;
        let dot_radius = 1.5;
        let n_dots = 8;
        
        for i in 0..n_dots {
            let angle = (i as f32 / n_dots as f32) * std::f32::consts::TAU + self.loading_animation;
            let dot_pos = center + egui::Vec2::new(radius * angle.cos(), radius * angle.sin());
            
            // Fade dots based on position
            let alpha = ((i as f32 / n_dots as f32) + (self.loading_animation / std::f32::consts::TAU)) % 1.0;
            let color = Color32::from_rgba_unmultiplied(
                theme.colors.loading_spinner_color.r(),
                theme.colors.loading_spinner_color.g(),
                theme.colors.loading_spinner_color.b(),
                (255.0 * alpha.powf(2.0)) as u8,
            );
            
            ui.painter().circle_filled(dot_pos, dot_radius, color);
        }
    }
    
    fn render_network_and_performance(
        &mut self,
        ui: &mut Ui,
        info_rect: Rect,
        status_info: &StatusInfo,
        theme_manager: &ThemeManager,
        performance_monitor: Option<&PerformanceMonitor>,
    ) {
        let theme = theme_manager.get_current_theme();
        
        ui.allocate_ui_at_rect(info_rect, |ui| {
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.add_space(8.0);
                
                // Performance metrics (if enabled)
                if let Some(perf_monitor) = performance_monitor {
                    if self.show_performance_metrics {
                        let stats = perf_monitor.get_performance_stats();
                        
                        ui.label(RichText::new(format!("{:.0} FPS", stats.current_fps))
                            .size(10.0)
                            .color(if stats.current_fps >= 50.0 {
                                Color32::from_rgb(34, 139, 34)
                            } else {
                                Color32::from_rgb(255, 140, 0)
                            }));
                        
                        ui.separator();
                        
                        ui.label(RichText::new(format!("{:.1}MB", stats.memory_usage_mb))
                            .size(10.0)
                            .color(theme.colors.status_text_color));
                        
                        ui.separator();
                    }
                }
                
                // Genesis network status
                let (network_icon, network_color, network_text) = if status_info.genesis_connected {
                    let pulse = (self.network_pulse.sin() * 0.5 + 0.5).clamp(0.3, 1.0);
                    let pulsed_color = Color32::from_rgba_unmultiplied(
                        theme.colors.genesis_accent_color.r(),
                        theme.colors.genesis_accent_color.g(),
                        theme.colors.genesis_accent_color.b(),
                        (255.0 * pulse) as u8,
                    );
                    ("🟢", pulsed_color, "Genesis Online")
                } else {
                    ("🔴", Color32::from_rgb(220, 53, 69), "Genesis Offline")
                };
                
                ui.label(RichText::new(network_text)
                    .size(10.0)
                    .color(theme.colors.status_text_color));
                
                ui.label(RichText::new(network_icon).color(network_color));
                
                // Connection indicator (animated)
                if status_info.genesis_connected {
                    let pulse_radius = 3.0 + (self.network_pulse.sin() * 1.0).abs();
                    let indicator_pos = [
                        info_rect.right() - 15.0,
                        info_rect.center().y,
                    ].into();
                    
                    ui.painter().circle_filled(
                        indicator_pos,
                        pulse_radius,
                        Color32::from_rgba_unmultiplied(
                            theme.colors.genesis_accent_color.r(),
                            theme.colors.genesis_accent_color.g(),
                            theme.colors.genesis_accent_color.b(),
                            30,
                        ),
                    );
                    
                    ui.painter().circle_filled(
                        indicator_pos,
                        2.0,
                        theme.colors.genesis_accent_color,
                    );
                }
            });
        });
    }
    
    // === Public Methods ===
    
    pub fn set_status_message(&mut self, message: String) {
        self.last_status_message = message;
        self.status_message_time = 0.0;
    }
    
    pub fn show_performance_metrics(&mut self, show: bool) {
        self.show_performance_metrics = show;
    }
    
    pub fn show_detailed_status(&mut self, show: bool) {
        self.show_detailed_status = show;
    }
    
    pub fn set_status_bar_height(&mut self, height: f32) {
        self.status_bar_height = height;
    }
    
    // === Getters ===
    
    pub fn get_status_bar_height(&self) -> f32 {
        self.status_bar_height
    }
    
    pub fn is_showing_performance_metrics(&self) -> bool {
        self.show_performance_metrics
    }
}