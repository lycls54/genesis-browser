// UI Renderers - Single Responsibility Principle uyumlu render modülleri
//
// Her renderer tek bir UI bileşeninin render edilmesinden sorumludur:
// - TabRenderer: Tab bar rendering
// - NavigationRenderer: Navigation bar rendering  
// - PanelRenderer: Side panels rendering
// - StatusRenderer: Status bar rendering

pub mod tab_renderer;
pub mod navigation_renderer;
pub mod panel_renderer;
pub mod status_renderer;

// Re-exports
pub use tab_renderer::TabRenderer;
pub use navigation_renderer::NavigationRenderer;
pub use panel_renderer::PanelRenderer;
pub use status_renderer::StatusRenderer;