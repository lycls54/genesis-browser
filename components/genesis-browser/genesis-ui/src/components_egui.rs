// UI Components for Genesis Browser with egui
// This module requires the "egui" or "enhanced-ui" feature

#![cfg(any(feature = "egui", feature = "enhanced-ui"))]

use egui::{
    Button, Color32, FontId, Label, RichText, ScrollArea, TextEdit, 
    Ui, vec2, ProgressBar, Layout, Align, Key
};

/// Navigation bar component with back, forward, reload, and URL input
pub struct NavigationBar<'a> {
    pub can_go_back: bool,
    pub can_go_forward: bool,
    pub url: &'a mut String,
    pub is_loading: bool,
    pub is_genesis_domain: bool,
}

impl<'a> NavigationBar<'a> {
    pub fn show(self, ui: &mut Ui) -> NavigationBarResponse {
        let mut response = NavigationBarResponse::default();
        
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing = vec2(4.0, 0.0);
            
            // Back button
            if ui.add_enabled(self.can_go_back, Button::new("⬅"))
                .on_hover_text("Back (Alt+Left)")
                .clicked() 
            {
                response.go_back = true;
            }
            
            // Forward button
            if ui.add_enabled(self.can_go_forward, Button::new("➡"))
                .on_hover_text("Forward (Alt+Right)")
                .clicked() 
            {
                response.go_forward = true;
            }
            
            // Reload/Stop button
            if self.is_loading {
                if ui.button("⏹")
                    .on_hover_text("Stop loading")
                    .clicked() 
                {
                    response.stop = true;
                }
            } else {
                if ui.button("🔄")
                    .on_hover_text("Reload (F5)")
                    .clicked() 
                {
                    response.reload = true;
                }
            }
            
            // Home button
            if ui.button("🏠")
                .on_hover_text("Genesis Home")
                .clicked() 
            {
                response.go_home = true;
            }
            
            // URL bar with Genesis indicator
            ui.horizontal(|ui| {
                // Genesis domain indicator
                if self.is_genesis_domain {
                    ui.add(Label::new(
                        RichText::new("🌐")
                            .color(Color32::from_rgb(34, 197, 94))
                            .size(16.0)
                    ));
                    ui.label(
                        RichText::new("Genesis")
                            .color(Color32::from_rgb(34, 197, 94))
                            .small()
                    );
                    ui.separator();
                } else {
                    ui.add(Label::new(
                        RichText::new("🌍")
                            .color(Color32::GRAY)
                            .size(16.0)
                    ));
                }
                
                // URL input field
                let url_response = ui.add(
                    TextEdit::singleline(self.url)
                        .desired_width(ui.available_width() - 100.0)
                        .hint_text("Enter Genesis domain or URL...")
                        .font(FontId::proportional(14.0))
                );
                
                if url_response.lost_focus() && ui.input(|i| i.key_pressed(Key::Enter)) {
                    response.navigate = true;
                    response.new_url = Some(self.url.clone());
                }
            });
        });
        
        response
    }
}

#[derive(Default)]
pub struct NavigationBarResponse {
    pub go_back: bool,
    pub go_forward: bool,
    pub reload: bool,
    pub stop: bool,
    pub go_home: bool,
    pub navigate: bool,
    pub new_url: Option<String>,
}