// Theme Manager - Single Responsibility: Görsel tema yönetimi
//
// Bu manager sadece tema ve görsellikle ilgili işlemleri yönetir:
// - Renk şemaları
// - Font yönetimi
// - Visual style states
// - Theme switching

use egui::{Color32, FontFamily, FontId, Stroke, Rounding};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug)]
pub struct ThemeManager {
    current_theme: BrowserTheme,
    available_themes: Vec<BrowserTheme>,
    custom_fonts_loaded: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum ThemeType {
    Light,
    Dark,
    Genesis,
    HighContrast,
    Custom,
}

#[derive(Clone, Debug)]
pub struct BrowserTheme {
    pub theme_type: ThemeType,
    pub name: String,
    pub colors: ThemeColors,
    pub fonts: ThemeFonts,
    pub spacing: ThemeSpacing,
    pub animations: ThemeAnimations,
}

#[derive(Clone, Debug)]
pub struct ThemeColors {
    // Window & Background
    pub window_fill: Color32,
    pub panel_fill: Color32,
    pub tab_area_fill: Color32,
    pub content_area_fill: Color32,
    
    // Tab Colors
    pub active_tab_fill: Color32,
    pub inactive_tab_fill: Color32,
    pub tab_hover_fill: Color32,
    pub tab_text_color: Color32,
    pub tab_close_button_color: Color32,
    
    // Navigation
    pub address_bar_fill: Color32,
    pub address_bar_text: Color32,
    pub address_bar_border: Color32,
    pub button_fill: Color32,
    pub button_hover_fill: Color32,
    pub button_active_fill: Color32,
    pub button_text: Color32,
    
    // Status & Progress
    pub progress_bar_bg: Color32,
    pub progress_bar_fill: Color32,
    pub loading_spinner_color: Color32,
    pub status_text_color: Color32,
    
    // Genesis Specific
    pub genesis_accent_color: Color32,
    pub genesis_domain_highlight: Color32,
    pub blockchain_indicator_color: Color32,
    
    // Borders & Strokes
    pub border_color: Color32,
    pub separator_color: Color32,
    pub focus_ring_color: Color32,
}

#[derive(Clone, Debug)]
pub struct ThemeFonts {
    pub ui_font_family: FontFamily,
    pub ui_font_size: f32,
    pub tab_font_size: f32,
    pub address_bar_font_size: f32,
    pub monospace_font_size: f32,
}

#[derive(Clone, Debug)]
pub struct ThemeSpacing {
    pub tab_height: f32,
    pub tab_width: f32,
    pub tab_padding: f32,
    pub button_padding: f32,
    pub panel_margin: f32,
    pub border_radius: f32,
}

#[derive(Clone, Debug)]
pub struct ThemeAnimations {
    pub tab_open_duration_ms: u64,
    pub tab_close_duration_ms: u64,
    pub hover_transition_ms: u64,
    pub page_load_animation_speed: f32,
}

impl Default for ThemeManager {
    fn default() -> Self {
        let mut manager = Self {
            current_theme: Self::create_genesis_theme(),
            available_themes: Vec::new(),
            custom_fonts_loaded: false,
        };
        
        // Predefined themes'i yükle
        manager.load_predefined_themes();
        manager
    }
}

impl ThemeManager {
    pub fn new() -> Self {
        Self::default()
    }
    
    // === Theme Creation ===
    
    pub fn create_genesis_theme() -> BrowserTheme {
        BrowserTheme {
            theme_type: ThemeType::Genesis,
            name: "Genesis".to_string(),
            colors: ThemeColors {
                // Chrome-like base with Genesis accents
                window_fill: Color32::from_rgb(255, 255, 255),
                panel_fill: Color32::from_rgb(255, 255, 255),
                tab_area_fill: Color32::from_rgb(222, 225, 230),
                content_area_fill: Color32::from_rgb(255, 255, 255),
                
                // Tab styling
                active_tab_fill: Color32::from_rgb(255, 255, 255),
                inactive_tab_fill: Color32::from_rgb(241, 243, 244),
                tab_hover_fill: Color32::from_rgb(248, 249, 250),
                tab_text_color: Color32::from_rgb(60, 64, 67),
                tab_close_button_color: Color32::from_rgb(95, 99, 104),
                
                // Navigation
                address_bar_fill: Color32::from_rgb(255, 255, 255),
                address_bar_text: Color32::from_rgb(60, 64, 67),
                address_bar_border: Color32::from_rgb(218, 220, 224),
                button_fill: Color32::from_rgb(255, 255, 255),
                button_hover_fill: Color32::from_rgb(248, 249, 250),
                button_active_fill: Color32::from_rgb(241, 243, 244),
                button_text: Color32::from_rgb(60, 64, 67),
                
                // Progress & Status
                progress_bar_bg: Color32::from_rgb(232, 234, 237),
                progress_bar_fill: Color32::from_rgb(66, 133, 244),
                loading_spinner_color: Color32::from_rgb(66, 133, 244),
                status_text_color: Color32::from_rgb(95, 99, 104),
                
                // Genesis Specific (Purple-blue gradient theme)
                genesis_accent_color: Color32::from_rgb(138, 43, 226), // BlueViolet
                genesis_domain_highlight: Color32::from_rgb(147, 112, 219), // MediumSlateBlue
                blockchain_indicator_color: Color32::from_rgb(75, 0, 130), // Indigo
                
                // Borders
                border_color: Color32::from_rgb(218, 220, 224),
                separator_color: Color32::from_rgb(232, 234, 237),
                focus_ring_color: Color32::from_rgb(26, 115, 232),
            },
            fonts: ThemeFonts {
                ui_font_family: FontFamily::Proportional,
                ui_font_size: 13.0,
                tab_font_size: 12.0,
                address_bar_font_size: 14.0,
                monospace_font_size: 12.0,
            },
            spacing: ThemeSpacing {
                tab_height: 35.0,
                tab_width: 200.0,
                tab_padding: 12.0,
                button_padding: 8.0,
                panel_margin: 4.0,
                border_radius: 6.0,
            },
            animations: ThemeAnimations {
                tab_open_duration_ms: 200,
                tab_close_duration_ms: 150,
                hover_transition_ms: 100,
                page_load_animation_speed: 1.5,
            },
        }
    }
    
    pub fn create_dark_theme() -> BrowserTheme {
        BrowserTheme {
            theme_type: ThemeType::Dark,
            name: "Dark".to_string(),
            colors: ThemeColors {
                // Dark theme colors
                window_fill: Color32::from_rgb(32, 33, 36),
                panel_fill: Color32::from_rgb(32, 33, 36),
                tab_area_fill: Color32::from_rgb(53, 54, 58),
                content_area_fill: Color32::from_rgb(32, 33, 36),
                
                active_tab_fill: Color32::from_rgb(32, 33, 36),
                inactive_tab_fill: Color32::from_rgb(53, 54, 58),
                tab_hover_fill: Color32::from_rgb(60, 64, 67),
                tab_text_color: Color32::from_rgb(232, 234, 237),
                tab_close_button_color: Color32::from_rgb(154, 160, 166),
                
                address_bar_fill: Color32::from_rgb(32, 33, 36),
                address_bar_text: Color32::from_rgb(232, 234, 237),
                address_bar_border: Color32::from_rgb(95, 99, 104),
                button_fill: Color32::from_rgb(48, 49, 52),
                button_hover_fill: Color32::from_rgb(60, 64, 67),
                button_active_fill: Color32::from_rgb(95, 99, 104),
                button_text: Color32::from_rgb(232, 234, 237),
                
                progress_bar_bg: Color32::from_rgb(95, 99, 104),
                progress_bar_fill: Color32::from_rgb(138, 180, 248),
                loading_spinner_color: Color32::from_rgb(138, 180, 248),
                status_text_color: Color32::from_rgb(154, 160, 166),
                
                // Genesis accents in dark mode
                genesis_accent_color: Color32::from_rgb(186, 85, 211), // MediumOrchid
                genesis_domain_highlight: Color32::from_rgb(221, 160, 221), // Plum
                blockchain_indicator_color: Color32::from_rgb(147, 112, 219), // MediumSlateBlue
                
                border_color: Color32::from_rgb(95, 99, 104),
                separator_color: Color32::from_rgb(60, 64, 67),
                focus_ring_color: Color32::from_rgb(138, 180, 248),
            },
            fonts: ThemeFonts {
                ui_font_family: FontFamily::Proportional,
                ui_font_size: 13.0,
                tab_font_size: 12.0,
                address_bar_font_size: 14.0,
                monospace_font_size: 12.0,
            },
            spacing: ThemeSpacing {
                tab_height: 35.0,
                tab_width: 200.0,
                tab_padding: 12.0,
                button_padding: 8.0,
                panel_margin: 4.0,
                border_radius: 6.0,
            },
            animations: ThemeAnimations {
                tab_open_duration_ms: 200,
                tab_close_duration_ms: 150,
                hover_transition_ms: 100,
                page_load_animation_speed: 1.5,
            },
        }
    }
    
    // === Theme Management ===
    
    pub fn switch_theme(&mut self, theme_type: ThemeType) -> bool {
        if let Some(theme) = self.available_themes.iter()
            .find(|t| t.theme_type == theme_type) {
            self.current_theme = theme.clone();
            true
        } else {
            false
        }
    }
    
    pub fn switch_theme_by_name(&mut self, name: &str) -> bool {
        if let Some(theme) = self.available_themes.iter()
            .find(|t| t.name == name) {
            self.current_theme = theme.clone();
            true
        } else {
            false
        }
    }
    
    pub fn add_custom_theme(&mut self, theme: BrowserTheme) {
        // Aynı isimde tema varsa değiştir
        if let Some(index) = self.available_themes.iter()
            .position(|t| t.name == theme.name) {
            self.available_themes[index] = theme;
        } else {
            self.available_themes.push(theme);
        }
    }
    
    pub fn get_current_theme(&self) -> &BrowserTheme {
        &self.current_theme
    }
    
    pub fn get_available_themes(&self) -> &Vec<BrowserTheme> {
        &self.available_themes
    }
    
    // === Visual Helpers ===
    
    pub fn get_stroke_for_element(&self, element: UIElement, state: ElementState) -> Stroke {
        let colors = &self.current_theme.colors;
        
        let color = match (element, state) {
            (UIElement::Button, ElementState::Normal) => colors.border_color,
            (UIElement::Button, ElementState::Hover) => colors.focus_ring_color,
            (UIElement::Tab, ElementState::Active) => colors.genesis_accent_color,
            (UIElement::Tab, _) => colors.border_color,
            (UIElement::AddressBar, ElementState::Focus) => colors.focus_ring_color,
            _ => colors.border_color,
        };
        
        Stroke::new(1.0, color)
    }
    
    pub fn get_rounding(&self) -> Rounding {
        Rounding::same(self.current_theme.spacing.border_radius)
    }
    
    pub fn get_font_id(&self, element: FontElement) -> FontId {
        let fonts = &self.current_theme.fonts;
        
        match element {
            FontElement::UI => FontId::new(fonts.ui_font_size, fonts.ui_font_family.clone()),
            FontElement::Tab => FontId::new(fonts.tab_font_size, fonts.ui_font_family.clone()),
            FontElement::AddressBar => FontId::new(fonts.address_bar_font_size, fonts.ui_font_family.clone()),
            FontElement::Monospace => FontId::new(fonts.monospace_font_size, FontFamily::Monospace),
        }
    }
    
    // === Animation Helpers ===
    
    pub fn get_tab_animation_duration(&self, animation_type: TabAnimationType) -> u64 {
        match animation_type {
            TabAnimationType::Opening => self.current_theme.animations.tab_open_duration_ms,
            TabAnimationType::Closing => self.current_theme.animations.tab_close_duration_ms,
        }
    }
    
    pub fn get_hover_transition_duration(&self) -> u64 {
        self.current_theme.animations.hover_transition_ms
    }
    
    // === Genesis Theme Specific ===
    
    pub fn is_genesis_domain_active(&self, url: &str) -> bool {
        url.starts_with("genesis://") || 
        url.contains(".genesis") || 
        url.contains(".free") ||
        url.contains(".web") ||
        url.contains(".defi") ||
        url.contains(".dao")
    }
    
    pub fn get_genesis_accent_color(&self) -> Color32 {
        self.current_theme.colors.genesis_accent_color
    }
    
    // === Private Methods ===
    
    fn load_predefined_themes(&mut self) {
        self.available_themes.push(Self::create_genesis_theme());
        self.available_themes.push(Self::create_dark_theme());
        // Diğer temalar buraya eklenebilir
    }
}

#[derive(Clone, Debug)]
pub enum UIElement {
    Button,
    Tab,
    AddressBar,
    Panel,
}

#[derive(Clone, Debug)]
pub enum ElementState {
    Normal,
    Hover,
    Active,
    Focus,
    Disabled,
}

#[derive(Clone, Debug)]
pub enum FontElement {
    UI,
    Tab,
    AddressBar,
    Monospace,
}

#[derive(Clone, Debug, PartialEq)]
pub enum TabAnimationType {
    Opening,
    Closing,
}