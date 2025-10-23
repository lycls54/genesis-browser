// Navigation Manager - Single Responsibility: Navigasyon ve URL yönetimi
//
// Bu manager sadece navigasyonla ilgili işlemleri yönetir:
// - URL validation ve parsing
// - Navigation history
// - URL suggestions
// - Search queries

use url::Url;
use std::collections::VecDeque;

pub struct NavigationManager {
    current_url: String,
    url_history: VecDeque<String>,
    search_suggestions: Vec<String>,
    genesis_domains: Vec<String>,
    max_history_size: usize,
}

#[derive(Clone, Debug)]
pub struct NavigationEvent {
    pub event_type: NavigationType,
    pub url: String,
    pub timestamp: u64,
}

#[derive(Clone, Debug, PartialEq)]
pub enum NavigationType {
    Navigate,
    Reload,
    Back,
    Forward,
    Home,
    Search,
}

#[derive(Clone, Debug)]
pub struct UrlValidationResult {
    pub is_valid: bool,
    pub is_genesis_domain: bool,
    pub is_search_query: bool,
    pub parsed_url: Option<String>,
    pub suggestions: Vec<String>,
}

impl Default for NavigationManager {
    fn default() -> Self {
        Self {
            current_url: "genesis://welcome".to_string(),
            url_history: VecDeque::new(),
            search_suggestions: Vec::new(),
            genesis_domains: vec![
                ".genesis".to_string(),
                ".free".to_string(),
                ".web".to_string(),
                ".defi".to_string(),
                ".dao".to_string(),
            ],
            max_history_size: 100,
        }
    }
}

impl NavigationManager {
    pub fn new() -> Self {
        Self::default()
    }
    
    // === URL Validation ve Parsing ===
    
    pub fn validate_url(&self, input: &str) -> UrlValidationResult {
        let trimmed = input.trim();
        
        // Boş input kontrolü
        if trimmed.is_empty() {
            return UrlValidationResult {
                is_valid: false,
                is_genesis_domain: false,
                is_search_query: false,
                parsed_url: None,
                suggestions: vec!["genesis://welcome".to_string()],
            };
        }
        
        // Genesis protocol kontrolü
        if trimmed.starts_with("genesis://") {
            return UrlValidationResult {
                is_valid: true,
                is_genesis_domain: true,
                is_search_query: false,
                parsed_url: Some(trimmed.to_string()),
                suggestions: Vec::new(),
            };
        }
        
        // Genesis domain kontrolü
        let is_genesis = self.genesis_domains.iter()
            .any(|domain| trimmed.contains(domain));
        
        // URL parsing denemesi
        match Url::parse(trimmed) {
            Ok(url) => UrlValidationResult {
                is_valid: true,
                is_genesis_domain: is_genesis,
                is_search_query: false,
                parsed_url: Some(url.to_string()),
                suggestions: Vec::new(),
            },
            Err(_) => {
                // Scheme eklemeyi dene
                let with_https = format!("https://{}", trimmed);
                match Url::parse(&with_https) {
                    Ok(url) => UrlValidationResult {
                        is_valid: true,
                        is_genesis_domain: is_genesis,
                        is_search_query: false,
                        parsed_url: Some(url.to_string()),
                        suggestions: Vec::new(),
                    },
                    Err(_) => {
                        // Genesis domain önerisi
                        if !is_genesis && !trimmed.contains(".") {
                            let genesis_suggestions = self.genesis_domains.iter()
                                .map(|domain| format!("{}{}", trimmed, domain))
                                .collect();
                                
                            UrlValidationResult {
                                is_valid: false,
                                is_genesis_domain: false,
                                is_search_query: true,
                                parsed_url: None,
                                suggestions: genesis_suggestions,
                            }
                        } else {
                            UrlValidationResult {
                                is_valid: false,
                                is_genesis_domain: false,
                                is_search_query: true,
                                parsed_url: None,
                                suggestions: Vec::new(),
                            }
                        }
                    }
                }
            }
        }
    }
    
    pub fn build_search_url(&self, query: &str) -> String {
        // Genesis search için özel URL
        if self.is_genesis_search_context() {
            format!("genesis://search?q={}", urlencoding::encode(query))
        } else {
            format!("https://duckduckgo.com/?q={}", urlencoding::encode(query))
        }
    }
    
    pub fn normalize_url(&self, input: &str) -> String {
        let validation = self.validate_url(input);
        
        if let Some(parsed_url) = validation.parsed_url {
            parsed_url
        } else if validation.is_search_query {
            self.build_search_url(input)
        } else {
            // Fallback to current URL
            self.current_url.clone()
        }
    }
    
    // === Navigation History ===
    
    pub fn navigate_to(&mut self, url: String) -> NavigationEvent {
        // History'e ekle
        if !url.is_empty() && url != self.current_url {
            self.url_history.push_back(self.current_url.clone());
            
            // History boyutunu sınırla
            if self.url_history.len() > self.max_history_size {
                self.url_history.pop_front();
            }
        }
        
        self.current_url = url.clone();
        
        NavigationEvent {
            event_type: NavigationType::Navigate,
            url,
            timestamp: self.get_timestamp(),
        }
    }
    
    pub fn go_back(&mut self) -> Option<NavigationEvent> {
        if let Some(previous_url) = self.url_history.pop_back() {
            let _old_url = self.current_url.clone();
            self.current_url = previous_url.clone();
            
            Some(NavigationEvent {
                event_type: NavigationType::Back,
                url: previous_url,
                timestamp: self.get_timestamp(),
            })
        } else {
            None
        }
    }
    
    pub fn reload(&self) -> NavigationEvent {
        NavigationEvent {
            event_type: NavigationType::Reload,
            url: self.current_url.clone(),
            timestamp: self.get_timestamp(),
        }
    }
    
    pub fn go_home(&mut self) -> NavigationEvent {
        let home_url = "genesis://welcome".to_string();
        self.navigate_to(home_url)
    }
    
    // === URL Suggestions ===
    
    pub fn get_url_suggestions(&self, input: &str) -> Vec<String> {
        let mut suggestions = Vec::new();
        let input_lower = input.to_lowercase();
        
        // History'den öneriler
        for url in &self.url_history {
            if url.to_lowercase().contains(&input_lower) {
                suggestions.push(url.clone());
            }
        }
        
        // Genesis domain önerileri
        if !input_lower.contains(".") && !input_lower.starts_with("http") {
            for domain in &self.genesis_domains {
                suggestions.push(format!("{}{}", input, domain));
            }
        }
        
        // Popüler Genesis siteleri
        if input_lower.len() >= 2 {
            let popular_sites = vec![
                "welcome.genesis",
                "explorer.genesis", 
                "wallet.defi",
                "dex.defi",
                "governance.dao",
                "forum.free",
            ];
            
            for site in popular_sites {
                if site.contains(&input_lower) {
                    suggestions.push(format!("genesis://{}", site));
                }
            }
        }
        
        // Duplicates'i temizle ve sınırla
        suggestions.sort();
        suggestions.dedup();
        suggestions.truncate(10);
        
        suggestions
    }
    
    // === Getters ===
    
    pub fn get_current_url(&self) -> &String {
        &self.current_url
    }
    
    pub fn get_history(&self) -> &VecDeque<String> {
        &self.url_history
    }
    
    pub fn can_go_back(&self) -> bool {
        !self.url_history.is_empty()
    }
    
    pub fn is_genesis_domain(&self, url: &str) -> bool {
        url.starts_with("genesis://") || 
        self.genesis_domains.iter().any(|domain| url.contains(domain))
    }
    
    pub fn is_secure_context(&self) -> bool {
        self.current_url.starts_with("https://") || 
        self.current_url.starts_with("genesis://")
    }
    
    // === Utility Methods ===
    
    pub fn clear_history(&mut self) {
        self.url_history.clear();
    }
    
    pub fn set_max_history_size(&mut self, size: usize) {
        self.max_history_size = size;
        
        // Mevcut history'yi sınırla
        while self.url_history.len() > size {
            self.url_history.pop_front();
        }
    }
    
    pub fn extract_domain(&self, url: &str) -> Option<String> {
        if let Ok(parsed) = Url::parse(url) {
            parsed.host_str().map(|s| s.to_string())
        } else {
            None
        }
    }
    
    pub fn is_same_origin(&self, url1: &str, url2: &str) -> bool {
        match (Url::parse(url1), Url::parse(url2)) {
            (Ok(u1), Ok(u2)) => {
                u1.scheme() == u2.scheme() && 
                u1.host() == u2.host() && 
                u1.port() == u2.port()
            },
            _ => false,
        }
    }
    
    fn is_genesis_search_context(&self) -> bool {
        self.is_genesis_domain(&self.current_url)
    }
    
    fn get_timestamp(&self) -> u64 {
        use std::time::{SystemTime, UNIX_EPOCH};
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs()
    }
}