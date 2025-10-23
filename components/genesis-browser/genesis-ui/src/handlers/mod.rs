// Event Handlers - Single Responsibility Principle uyumlu event işleyiciler
//
// Her handler tek bir event tipinin işlenmesinden sorumludur:
// - InputHandler: Klavye ve fare girdi işleme
// - EventHandler: UI event'lerini işleme
// - DragHandler: Sürükle/bırak işlemleri
// - AnimationController: Animasyon kontrolü

pub mod input_handler;
pub mod event_handler;
pub mod drag_handler;
pub mod animation_controller;

// Re-exports
pub use input_handler::InputHandler;
pub use event_handler::EventHandler;
pub use drag_handler::DragHandler;
pub use animation_controller::AnimationController;