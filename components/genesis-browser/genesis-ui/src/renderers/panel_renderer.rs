// Panel Renderer - Single Responsibility: Side panels rendering
//
// Bu renderer sadece yan panellerin görselleştirmesi ile ilgili işlemleri yönetir:
// - Bookmarks panel
// - Downloads panel  
// - History panel
// - Settings panel
// - Dev tools panel

use crate::enhanced_browser::{Bookmark, Download, HistoryEntry};
use crate::managers::ThemeManager;
use egui::{Ui, Rect, Color32, Stroke, Sense, RichText, ScrollArea, Align};
use std::collections::HashMap;

pub struct PanelRenderer {
    // Panel state
    panel_widths: HashMap<PanelType, f32>,
    panel_scroll_positions: HashMap<PanelType, f32>,
    
    // Animation state
    panel_animations: HashMap<PanelType, PanelAnimation>,
    
    // Configuration
    default_panel_width: f32,
    min_panel_width: f32,
    max_panel_width: f32,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum PanelType {
    Bookmarks,
    Downloads,
    History,
    Settings,
    DevTools,
}

#[derive(Clone, Debug)]
pub struct PanelAnimation {
    pub progress: f32,
    pub target_width: f32,
    pub is_opening: bool,
}

#[derive(Clone, Debug, Default)]
pub struct PanelRenderResponse {
    pub navigate_to: Option<String>,
    pub close_panel: bool,
    pub delete_bookmark: Option<String>,
    pub add_bookmark: Option<(String, String)>, // title, url
    pub cancel_download: Option<String>,
    pub open_download: Option<String>,
    pub clear_history: bool,
    pub setting_changed: Option<(String, String)>, // key, value
}

impl Default for PanelRenderer {
    fn default() -> Self {
        Self {
            panel_widths: HashMap::new(),
            panel_scroll_positions: HashMap::new(),
            panel_animations: HashMap::new(),
            default_panel_width: 300.0,
            min_panel_width: 200.0,
            max_panel_width: 500.0,
        }
    }
}

impl PanelRenderer {
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Panel render et
    pub fn render_panel(
        &mut self,
        ui: &mut Ui,
        panel_type: PanelType,
        panel_rect: Rect,
        theme_manager: &ThemeManager,
        panel_data: &PanelData,
    ) -> PanelRenderResponse {
        let mut response = PanelRenderResponse {
            navigate_to: None,
            close_panel: false,
            delete_bookmark: None,
            add_bookmark: None,
            cancel_download: None,
            open_download: None,
            clear_history: false,
            setting_changed: None,
        };
        
        let theme = theme_manager.get_current_theme();
        
        // Panel background
        ui.painter().rect_filled(
            panel_rect,
            theme_manager.get_rounding(),
            theme.colors.panel_fill,
        );
        
        // Panel border
        ui.painter().rect_stroke(
            panel_rect,
            theme_manager.get_rounding(),
            Stroke::new(1.0, theme.colors.border_color),
        );
        
        // Panel header
        let header_rect = Rect::from_min_size(
            panel_rect.min,
            [panel_rect.width(), 40.0].into(),
        );
        
        let header_response = self.render_panel_header(
            ui,
            header_rect,
            &panel_type,
            theme_manager,
        );
        
        if header_response.close_clicked {
            response.close_panel = true;
        }
        
        // Panel content
        let content_rect = Rect::from_min_size(
            [panel_rect.min.x, panel_rect.min.y + 40.0].into(),
            [panel_rect.width(), panel_rect.height() - 40.0].into(),
        );
        
        let content_response = self.render_panel_content(
            ui,
            content_rect,
            &panel_type,
            theme_manager,
            panel_data,
        );
        
        // Merge content response
        response.navigate_to = content_response.navigate_to;
        response.delete_bookmark = content_response.delete_bookmark;
        response.add_bookmark = content_response.add_bookmark;
        response.cancel_download = content_response.cancel_download;
        response.open_download = content_response.open_download;
        response.clear_history = content_response.clear_history;
        response.setting_changed = content_response.setting_changed;
        
        response
    }
    
    fn render_panel_header(
        &self,
        ui: &mut Ui,
        header_rect: Rect,
        panel_type: &PanelType,
        theme_manager: &ThemeManager,
    ) -> PanelHeaderResponse {
        let theme = theme_manager.get_current_theme();
        
        // Header background
        ui.painter().rect_filled(
            header_rect,
            egui::Rounding::ZERO,
            theme.colors.tab_area_fill,
        );
        
        // Panel title
        let title = match panel_type {
            PanelType::Bookmarks => "📑 Bookmarks",
            PanelType::Downloads => "⬇ Downloads", 
            PanelType::History => "🕒 History",
            PanelType::Settings => "⚙ Settings",
            PanelType::DevTools => "🔧 Developer Tools",
        };
        
        ui.painter().text(
            egui::Pos2::new(header_rect.left() + 12.0, header_rect.center().y),
            egui::Align2::LEFT_CENTER,
            title,
            theme_manager.get_font_id(crate::managers::theme_manager::FontElement::UI),
            theme.colors.tab_text_color,
        );
        
        // Close button
        let close_button_rect = Rect::from_center_size(
            [header_rect.right() - 20.0, header_rect.center().y].into(),
            [16.0, 16.0].into(),
        );
        
        let close_response = ui.allocate_rect(close_button_rect, Sense::click());
        
        // Close button background on hover
        if close_response.hovered() {
            ui.painter().circle_filled(
                close_button_rect.center(),
                8.0,
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
            Stroke::new(1.5, color),
        );
        
        ui.painter().line_segment(
            [
                [center.x + x_size / 2.0, center.y - x_size / 2.0].into(),
                [center.x - x_size / 2.0, center.y + x_size / 2.0].into(),
            ],
            Stroke::new(1.5, color),
        );
        
        PanelHeaderResponse {
            close_clicked: close_response.clicked(),
        }
    }
    
    fn render_panel_content(
        &mut self,
        ui: &mut Ui,
        content_rect: Rect,
        panel_type: &PanelType,
        theme_manager: &ThemeManager,
        panel_data: &PanelData,
    ) -> PanelRenderResponse {
        ui.allocate_ui_at_rect(content_rect, |ui| {
            match panel_type {
                PanelType::Bookmarks => self.render_bookmarks_content(ui, theme_manager, &panel_data.bookmarks),
                PanelType::Downloads => self.render_downloads_content(ui, theme_manager, &panel_data.downloads),
                PanelType::History => self.render_history_content(ui, theme_manager, &panel_data.history),
                PanelType::Settings => self.render_settings_content(ui, theme_manager),
                PanelType::DevTools => self.render_devtools_content(ui, theme_manager),
            }
        }).inner
    }
    
    fn render_bookmarks_content(
        &self,
        ui: &mut Ui,
        theme_manager: &ThemeManager,
        bookmarks: &[Bookmark],
    ) -> PanelRenderResponse {
        let mut response = PanelRenderResponse::default();
        let _theme = theme_manager.get_current_theme();
        
        ScrollArea::vertical()
            .id_source("bookmarks_scroll")
            .show(ui, |ui| {
                // Add bookmark button
                if ui.button("➕ Add Current Page").clicked() {
                    response.add_bookmark = Some(("Current Page".to_string(), "current://url".to_string()));
                }
                
                ui.separator();
                
                // Bookmarks list
                for bookmark in bookmarks {
                    ui.horizontal(|ui| {
                        // Favicon placeholder
                        ui.label("🔖");
                        
                        // Bookmark title and URL
                        ui.vertical(|ui| {
                            if ui.link(&bookmark.title).clicked() {
                                response.navigate_to = Some(bookmark.url.clone());
                            }
                            
                            ui.label(RichText::new(&bookmark.url)
                                .size(10.0)
                                .color(Color32::from_gray(120)));
                        });
                        
                        ui.with_layout(egui::Layout::right_to_left(Align::Center), |ui| {
                            if ui.small_button("🗑").clicked() {
                                response.delete_bookmark = Some(bookmark.id.clone());
                            }
                        });
                    });
                    
                    ui.separator();
                }
                
                if bookmarks.is_empty() {
                    ui.centered_and_justified(|ui| {
                        ui.label(RichText::new("No bookmarks yet")
                            .color(Color32::from_gray(120)));
                    });
                }
            });
            
        response
    }
    
    fn render_downloads_content(
        &self,
        ui: &mut Ui,
        theme_manager: &ThemeManager,
        downloads: &[Download],
    ) -> PanelRenderResponse {
        let mut response = PanelRenderResponse::default();
        let _theme = theme_manager.get_current_theme();
        
        ScrollArea::vertical()
            .id_source("downloads_scroll")
            .show(ui, |ui| {
                // Clear all button
                if !downloads.is_empty() {
                    ui.horizontal(|ui| {
                        if ui.button("🗑 Clear All").clicked() {
                            // TODO: Clear all downloads
                        }
                        ui.with_layout(egui::Layout::right_to_left(Align::Center), |ui| {
                            ui.label(format!("{} downloads", downloads.len()));
                        });
                    });
                    
                    ui.separator();
                }
                
                // Downloads list
                for download in downloads {
                    ui.group(|ui| {
                        ui.horizontal(|ui| {
                            // File icon
                            let file_icon = match download.file_type.as_str() {
                                "image" => "🖼",
                                "video" => "🎥",
                                "audio" => "🎵",
                                "document" => "📄",
                                "archive" => "📦",
                                _ => "📁",
                            };
                            ui.label(file_icon);
                            
                            ui.vertical(|ui| {
                                // File name
                                if ui.link(&download.filename).clicked() {
                                    response.open_download = Some(download.id.clone());
                                }
                                
                                // Progress bar
                                if download.progress < 1.0 {
                                    let progress_bar = egui::ProgressBar::new(download.progress)
                                        .text(format!("{:.1}%", download.progress * 100.0));
                                    ui.add(progress_bar);
                                } else {
                                    ui.label(RichText::new("Complete ✅")
                                        .size(10.0)
                                        .color(Color32::from_rgb(34, 139, 34)));
                                }
                                
                                // File size and speed
                                ui.horizontal(|ui| {
                                    ui.label(RichText::new(format!("{} bytes", download.size))
                                        .size(10.0)
                                        .color(Color32::from_gray(120)));
                                    
                                    if download.progress < 1.0 {
                                        ui.label(RichText::new("Calculating...")
                                            .size(10.0)
                                            .color(Color32::from_gray(120)));
                                    }
                                });
                            });
                            
                            ui.with_layout(egui::Layout::right_to_left(Align::Center), |ui| {
                                if download.progress < 1.0 {
                                    if ui.small_button("⏸").clicked() {
                                        // TODO: Pause download
                                    }
                                    if ui.small_button("❌").clicked() {
                                        response.cancel_download = Some(download.id.clone());
                                    }
                                } else {
                                    if ui.small_button("📁").clicked() {
                                        // TODO: Open folder
                                    }
                                }
                            });
                        });
                    });
                }
                
                if downloads.is_empty() {
                    ui.centered_and_justified(|ui| {
                        ui.label(RichText::new("No downloads")
                            .color(Color32::from_gray(120)));
                    });
                }
            });
            
        response
    }
    
    fn render_history_content(
        &self,
        ui: &mut Ui,
        theme_manager: &ThemeManager,
        history: &[HistoryEntry],
    ) -> PanelRenderResponse {
        let mut response = PanelRenderResponse::default();
        
        ScrollArea::vertical()
            .id_source("history_scroll")
            .show(ui, |ui| {
                // Clear history button
                if !history.is_empty() {
                    ui.horizontal(|ui| {
                        if ui.button("🗑 Clear History").clicked() {
                            response.clear_history = true;
                        }
                        ui.with_layout(egui::Layout::right_to_left(Align::Center), |ui| {
                            ui.label(format!("{} entries", history.len()));
                        });
                    });
                    
                    ui.separator();
                }
                
                // History list grouped by date
                let mut current_date = String::new();
                
                for entry in history {
                    let entry_date = entry.last_visit.format("%Y-%m-%d").to_string();
                    
                    if entry_date != current_date {
                        if !current_date.is_empty() {
                            ui.separator();
                        }
                        
                        ui.label(RichText::new(&entry_date)
                            .strong()
                            .color(theme_manager.get_current_theme().colors.genesis_accent_color));
                        current_date = entry_date;
                    }
                    
                    ui.horizontal(|ui| {
                        // Site icon/favicon placeholder
                        ui.label("🌐");
                        
                        ui.vertical(|ui| {
                            // Page title
                            if ui.link(&entry.title).clicked() {
                                response.navigate_to = Some(entry.url.clone());
                            }
                            
                            // URL and timestamp
                            ui.horizontal(|ui| {
                                ui.label(RichText::new(&entry.url)
                                    .size(10.0)
                                    .color(Color32::from_gray(120)));
                                
                                ui.with_layout(egui::Layout::right_to_left(Align::Center), |ui| {
                                    ui.label(RichText::new(entry.last_visit.format("%H:%M").to_string())
                                        .size(10.0)
                                        .color(Color32::from_gray(100)));
                                });
                            });
                        });
                    });
                }
                
                if history.is_empty() {
                    ui.centered_and_justified(|ui| {
                        ui.label(RichText::new("No browsing history")
                            .color(Color32::from_gray(120)));
                    });
                }
            });
            
        response
    }
    
    fn render_settings_content(
        &self,
        ui: &mut Ui,
        theme_manager: &ThemeManager,
    ) -> PanelRenderResponse {
        let mut response = PanelRenderResponse::default();
        
        ScrollArea::vertical()
            .id_source("settings_scroll")
            .show(ui, |ui| {
                ui.heading("Genesis Browser Settings");
                ui.separator();
                
                // Genesis settings
                ui.group(|ui| {
                    ui.label(RichText::new("🌐 Genesis Network").strong());
                    
                    ui.horizontal(|ui| {
                        ui.label("Genesis Node:");
                        // TODO: Add node selection
                        ui.label("localhost:3000");
                    });
                    
                    ui.checkbox(&mut true, "Enable Genesis DNS");
                    ui.checkbox(&mut true, "Traditional DNS fallback");
                    ui.checkbox(&mut false, "Blockchain verification");
                });
                
                ui.separator();
                
                // Privacy settings
                ui.group(|ui| {
                    ui.label(RichText::new("🔒 Privacy & Security").strong());
                    
                    ui.checkbox(&mut true, "Block tracking scripts");
                    ui.checkbox(&mut false, "Private browsing mode");
                    ui.checkbox(&mut true, "HTTPS everywhere");
                    ui.checkbox(&mut false, "Clear data on exit");
                });
                
                ui.separator();
                
                // Appearance settings
                ui.group(|ui| {
                    ui.label(RichText::new("🎨 Appearance").strong());
                    
                    ui.horizontal(|ui| {
                        ui.label("Theme:");
                        egui::ComboBox::from_id_source("theme_combo")
                            .selected_text("Genesis")
                            .show_ui(ui, |ui| {
                                ui.selectable_value(&mut "Genesis", "Genesis", "Genesis");
                                ui.selectable_value(&mut "Dark", "Dark", "Dark");
                                ui.selectable_value(&mut "Light", "Light", "Light");
                            });
                    });
                    
                    ui.checkbox(&mut true, "Show bookmarks bar");
                    ui.checkbox(&mut true, "Show download notifications");
                });
                
                ui.separator();
                
                // Advanced settings
                ui.group(|ui| {
                    ui.label(RichText::new("⚙ Advanced").strong());
                    
                    ui.checkbox(&mut true, "Enable JavaScript");
                    ui.checkbox(&mut true, "Enable WebGL");
                    ui.checkbox(&mut false, "Enable experimental features");
                    ui.checkbox(&mut true, "Hardware acceleration");
                });
            });
            
        response
    }
    
    fn render_devtools_content(
        &self,
        ui: &mut Ui,
        theme_manager: &ThemeManager,
    ) -> PanelRenderResponse {
        let mut response = PanelRenderResponse::default();
        
        ScrollArea::vertical()
            .id_source("devtools_scroll")
            .show(ui, |ui| {
                ui.heading("Developer Tools");
                ui.separator();
                
                // Console
                ui.group(|ui| {
                    ui.label(RichText::new("📟 Console").strong());
                    
                    // Mock console output
                    ui.add(
                        egui::TextEdit::multiline(&mut "Genesis Browser Developer Console\n> Ready for debugging\n".to_string())
                            .font(egui::TextStyle::Monospace)
                            .desired_rows(10),
                    );
                });
                
                ui.separator();
                
                // Network
                ui.group(|ui| {
                    ui.label(RichText::new("🌐 Network").strong());
                    ui.label("No network requests yet");
                });
                
                ui.separator();
                
                // Performance
                ui.group(|ui| {
                    ui.label(RichText::new("⚡ Performance").strong());
                    ui.label("FPS: 144");
                    ui.label("Memory: 45.2 MB");
                    ui.label("Load time: 1.2s");
                });
            });
            
        response
    }
    
    // === Getters/Setters ===
    
    pub fn set_panel_width(&mut self, panel_type: PanelType, width: f32) {
        self.panel_widths.insert(panel_type, width.clamp(self.min_panel_width, self.max_panel_width));
    }
    
    pub fn get_panel_width(&self, panel_type: &PanelType) -> f32 {
        self.panel_widths.get(panel_type).cloned().unwrap_or(self.default_panel_width)
    }
}

#[derive(Clone, Debug)]
struct PanelHeaderResponse {
    close_clicked: bool,
}

#[derive(Clone, Debug, Default)]
pub struct PanelData {
    pub bookmarks: Vec<Bookmark>,
    pub downloads: Vec<Download>, 
    pub history: Vec<HistoryEntry>,
}