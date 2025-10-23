// Tab Manager - Single Responsibility: Tab operasyonları
//
// Bu manager sadece tab'larla ilgili işlemleri yönetir:
// - Tab oluşturma/kapatma
// - Tab geçişleri
// - Tab durumu yönetimi
// - Tab meta verileri

use crate::enhanced_browser::BrowserTab;
use std::collections::HashMap;

pub struct TabManager {
    tabs: Vec<BrowserTab>,
    active_tab_index: usize,
    tab_counter: u32,
    tab_animations: HashMap<String, TabAnimationState>,
}

#[derive(Clone)]
pub struct TabAnimationState {
    pub progress: f32,
    pub animation_type: TabAnimationType,
    pub target_width: f32,
}

#[derive(Clone, PartialEq)]
pub enum TabAnimationType {
    Opening,
    Closing,
    None,
}

impl Default for TabManager {
    fn default() -> Self {
        let mut manager = Self {
            tabs: Vec::new(),
            active_tab_index: 0,
            tab_counter: 0,
            tab_animations: HashMap::new(),
        };
        
        // Default tab oluştur
        manager.create_tab("genesis://welcome");
        manager
    }
}

impl TabManager {
    pub fn new() -> Self {
        Self::default()
    }
    
    // === Tab Oluşturma/Kapatma ===
    
    pub fn create_tab(&mut self, url: &str) -> String {
        self.tab_counter += 1;
        let tab = BrowserTab::new(url);
        let tab_id = tab.id.clone();
        
        self.tabs.push(tab);
        
        // Animasyon başlat
        self.tab_animations.insert(tab_id.clone(), TabAnimationState {
            progress: 0.0,
            animation_type: TabAnimationType::Opening,
            target_width: 200.0,
        });
        
        // Yeni tab'ı aktif yap
        self.active_tab_index = self.tabs.len() - 1;
        
        tab_id
    }
    
    pub fn close_tab(&mut self, tab_index: usize) -> bool {
        if tab_index >= self.tabs.len() || self.tabs.len() <= 1 {
            return false; // Son tab kapatılamaz
        }
        
        let tab_id = self.tabs[tab_index].id.clone();
        
        // Kapanma animasyonu başlat
        self.tab_animations.insert(tab_id.clone(), TabAnimationState {
            progress: 0.0,
            animation_type: TabAnimationType::Closing,
            target_width: 0.0,
        });
        
        self.tabs.remove(tab_index);
        
        // Aktif tab index'ini düzelt
        if self.active_tab_index >= tab_index && self.active_tab_index > 0 {
            self.active_tab_index -= 1;
        }
        if self.active_tab_index >= self.tabs.len() {
            self.active_tab_index = self.tabs.len() - 1;
        }
        
        true
    }
    
    pub fn close_tab_by_id(&mut self, tab_id: &str) -> bool {
        if let Some(index) = self.tabs.iter().position(|t| t.id == tab_id) {
            self.close_tab(index)
        } else {
            false
        }
    }
    
    // === Tab Geçişleri ===
    
    pub fn switch_to_tab(&mut self, tab_index: usize) -> bool {
        if tab_index < self.tabs.len() {
            self.active_tab_index = tab_index;
            true
        } else {
            false
        }
    }
    
    pub fn switch_to_tab_by_id(&mut self, tab_id: &str) -> bool {
        if let Some(index) = self.tabs.iter().position(|t| t.id == tab_id) {
            self.switch_to_tab(index)
        } else {
            false
        }
    }
    
    pub fn next_tab(&mut self) {
        if self.tabs.len() > 1 {
            self.active_tab_index = (self.active_tab_index + 1) % self.tabs.len();
        }
    }
    
    pub fn previous_tab(&mut self) {
        if self.tabs.len() > 1 {
            self.active_tab_index = if self.active_tab_index == 0 {
                self.tabs.len() - 1
            } else {
                self.active_tab_index - 1
            };
        }
    }
    
    // === Tab Durumu Yönetimi ===
    
    pub fn update_tab_title(&mut self, tab_index: usize, title: String) {
        if let Some(tab) = self.tabs.get_mut(tab_index) {
            tab.title = title;
        }
    }
    
    pub fn update_tab_url(&mut self, tab_index: usize, url: String) {
        if let Some(tab) = self.tabs.get_mut(tab_index) {
            tab.url = url;
            tab.is_genesis_domain = BrowserTab::check_genesis_domain(&tab.url);
        }
    }
    
    pub fn update_tab_loading(&mut self, tab_index: usize, is_loading: bool) {
        if let Some(tab) = self.tabs.get_mut(tab_index) {
            tab.is_loading = is_loading;
        }
    }
    
    pub fn update_tab_progress(&mut self, tab_index: usize, progress: f32) {
        if let Some(tab) = self.tabs.get_mut(tab_index) {
            tab.load_progress = progress.clamp(0.0, 1.0);
        }
    }
    
    pub fn update_tab_navigation(&mut self, tab_index: usize, can_go_back: bool, can_go_forward: bool) {
        if let Some(tab) = self.tabs.get_mut(tab_index) {
            tab.can_go_back = can_go_back;
            tab.can_go_forward = can_go_forward;
        }
    }
    
    pub fn set_tab_favicon(&mut self, tab_index: usize, favicon_url: Option<String>) {
        if let Some(tab) = self.tabs.get_mut(tab_index) {
            tab.favicon = favicon_url;
        }
    }
    
    // === Getters ===
    
    pub fn get_tabs(&self) -> &Vec<BrowserTab> {
        &self.tabs
    }
    
    pub fn get_active_tab_index(&self) -> usize {
        self.active_tab_index
    }
    
    pub fn get_active_tab(&self) -> Option<&BrowserTab> {
        self.tabs.get(self.active_tab_index)
    }
    
    pub fn get_active_tab_mut(&mut self) -> Option<&mut BrowserTab> {
        self.tabs.get_mut(self.active_tab_index)
    }
    
    pub fn get_tab(&self, index: usize) -> Option<&BrowserTab> {
        self.tabs.get(index)
    }
    
    pub fn get_tab_by_id(&self, tab_id: &str) -> Option<&BrowserTab> {
        self.tabs.iter().find(|t| t.id == tab_id)
    }
    
    pub fn get_tab_count(&self) -> usize {
        self.tabs.len()
    }
    
    // === Animasyon Yönetimi ===
    
    pub fn get_tab_animation(&self, tab_id: &str) -> Option<&TabAnimationState> {
        self.tab_animations.get(tab_id)
    }
    
    pub fn update_tab_animation(&mut self, tab_id: &str, progress: f32) {
        if let Some(animation) = self.tab_animations.get_mut(tab_id) {
            animation.progress = progress.clamp(0.0, 1.0);
            
            // Animasyon tamamlandıysa temizle
            if animation.progress >= 1.0 {
                if animation.animation_type == TabAnimationType::Closing {
                    self.tab_animations.remove(tab_id);
                } else {
                    animation.animation_type = TabAnimationType::None;
                }
            }
        }
    }
    
    pub fn clear_finished_animations(&mut self) {
        self.tab_animations.retain(|_, anim| {
            anim.progress < 1.0 || anim.animation_type != TabAnimationType::Closing
        });
    }
    
    // === Utility Methods ===
    
    pub fn is_genesis_domain(&self, tab_index: usize) -> bool {
        self.get_tab(tab_index)
            .map(|tab| tab.is_genesis_domain)
            .unwrap_or(false)
    }
    
    pub fn get_total_tabs_width(&self) -> f32 {
        self.tabs.len() as f32 * 200.0 // Default tab width
    }
    
    pub fn find_tab_index_by_id(&self, tab_id: &str) -> Option<usize> {
        self.tabs.iter().position(|t| t.id == tab_id)
    }
}