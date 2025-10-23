// UI Managers - Single Responsibility Principle uyumlu yöneticiler
//
// Her manager tek bir sorumluluğa odaklanır:
// - TabManager: Tab operasyonları
// - NavigationManager: URL ve navigasyon yönetimi  
// - ThemeManager: Görsel tema yönetimi
// - PerformanceMonitor: Performans metrikleri

pub mod tab_manager;
pub mod navigation_manager;
pub mod theme_manager;
pub mod performance_monitor;

// Re-exports
pub use tab_manager::TabManager;
pub use navigation_manager::NavigationManager;
pub use theme_manager::ThemeManager;
pub use performance_monitor::PerformanceMonitor;