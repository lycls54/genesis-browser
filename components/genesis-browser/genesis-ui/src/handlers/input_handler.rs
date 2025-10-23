// Input Handler - Single Responsibility: Klavye ve fare girdi işleme
//
// Bu handler sadece kullanıcı girdilerini işler:
// - Klavye kısayolları
// - Fare etkileşimleri
// - Touch input (gelecekte)
// - Input validation

use egui::{Key, Modifiers, PointerButton, Pos2, Vec2};
use std::collections::HashMap;

pub struct InputHandler {
    // Keyboard state
    pressed_keys: std::collections::HashSet<Key>,
    key_combinations: HashMap<KeyCombination, InputAction>,
    
    // Mouse state
    last_click_time: f64,
    last_click_pos: Option<Pos2>,
    double_click_threshold: f64,
    drag_threshold: f32,
    
    // Input settings
    repeat_delay: f64,
    repeat_rate: f64,
    
    // Input history for gesture recognition
    mouse_history: Vec<MouseEvent>,
    max_history_size: usize,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct KeyCombination {
    pub key: Key,
    pub ctrl: bool,
    pub alt: bool,
    pub shift: bool,
    pub cmd: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub enum InputAction {
    // Tab actions
    NewTab,
    CloseTab,
    NextTab,
    PrevTab,
    ReopenClosedTab,
    
    // Navigation actions
    Back,
    Forward,
    Reload,
    Home,
    Stop,
    
    // Browser actions
    Find,
    FindNext,
    FindPrev,
    ZoomIn,
    ZoomOut,
    ZoomReset,
    ToggleFullscreen,
    
    // Developer tools
    ToggleDevTools,
    ToggleInspector,
    ShowConsole,
    
    // Genesis specific
    ToggleGenesisMode,
    ShowGenesisExplorer,
    
    // General
    Quit,
    Copy,
    Paste,
    Cut,
    SelectAll,
}

#[derive(Clone, Debug)]
pub struct MouseEvent {
    pub position: Pos2,
    pub button: Option<PointerButton>,
    pub timestamp: f64,
    pub event_type: MouseEventType,
}

#[derive(Clone, Debug, PartialEq)]
pub enum MouseEventType {
    Click,
    DoubleClick,
    DragStart,
    Drag,
    DragEnd,
    Scroll,
    Hover,
}

#[derive(Clone, Debug)]
pub struct InputEvents {
    pub actions: Vec<InputAction>,
    pub mouse_events: Vec<MouseEvent>,
    pub text_input: String,
    pub scroll_delta: Vec2,
    pub window_events: Vec<WindowEvent>,
}

#[derive(Clone, Debug)]
pub enum WindowEvent {
    Resize(Vec2),
    Focus,
    Unfocus,
    Minimize,
    Maximize,
    Close,
}

impl Default for InputHandler {
    fn default() -> Self {
        let mut handler = Self {
            pressed_keys: std::collections::HashSet::new(),
            key_combinations: HashMap::new(),
            last_click_time: 0.0,
            last_click_pos: None,
            double_click_threshold: 0.5, // 500ms
            drag_threshold: 5.0,
            repeat_delay: 0.5,
            repeat_rate: 0.05,
            mouse_history: Vec::new(),
            max_history_size: 100,
        };
        
        handler.setup_default_key_bindings();
        handler
    }
}

impl InputHandler {
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Process input from egui context
    pub fn process_input(&mut self, ctx: &egui::Context) -> InputEvents {
        let mut events = InputEvents {
            actions: Vec::new(),
            mouse_events: Vec::new(),
            text_input: String::new(),
            scroll_delta: Vec2::ZERO,
            window_events: Vec::new(),
        };
        
        ctx.input(|i| {
            // Process keyboard input
            self.process_keyboard_input(i, &mut events);
            
            // Process mouse input
            self.process_mouse_input(i, &mut events);
            
            // Process text input
            for event in &i.events {
                if let egui::Event::Text(text) = event {
                    events.text_input.push_str(text);
                }
            }
            
            // Process scroll
            if i.scroll_delta != Vec2::ZERO {
                events.scroll_delta = i.scroll_delta;
            }
            
            // Process window events
            self.process_window_events(i, &mut events);
        });
        
        events
    }
    
    fn process_keyboard_input(&mut self, input: &egui::InputState, events: &mut InputEvents) {
        // Update pressed keys
        for event in &input.events {
            match event {
                egui::Event::Key { key, pressed, .. } => {
                    if *pressed {
                        self.pressed_keys.insert(*key);
                    } else {
                        self.pressed_keys.remove(key);
                    }
                }
                _ => {}
            }
        }
        
        // Check for key combinations
        let modifiers = input.modifiers;
        
        for (combination, action) in &self.key_combinations {
            if self.is_key_combination_pressed(combination, &modifiers, input) {
                events.actions.push(action.clone());
            }
        }
    }
    
    fn process_mouse_input(&mut self, input: &egui::InputState, events: &mut InputEvents) {
        let current_time = input.time;
        
        // Process mouse clicks
        if input.pointer.any_click() {
            let click_pos = input.pointer.interact_pos().unwrap_or_default();
            
            // Check for double click
            let is_double_click = if let Some(last_pos) = self.last_click_pos {
                current_time - self.last_click_time < self.double_click_threshold &&
                (click_pos - last_pos).length() < self.drag_threshold
            } else {
                false
            };
            
            let mouse_event = MouseEvent {
                position: click_pos,
                button: if input.pointer.primary_clicked() {
                    Some(PointerButton::Primary)
                } else if input.pointer.secondary_clicked() {
                    Some(PointerButton::Secondary)
                } else {
                    // egui doesn't have middle_clicked() in this version
                    None
                },
                timestamp: current_time,
                event_type: if is_double_click {
                    MouseEventType::DoubleClick
                } else {
                    MouseEventType::Click
                },
            };
            
            events.mouse_events.push(mouse_event.clone());
            self.add_to_mouse_history(mouse_event);
            
            self.last_click_time = current_time;
            self.last_click_pos = Some(click_pos);
        }
        
        // Process mouse drag
        if let Some(pointer_pos) = input.pointer.interact_pos() {
            if input.pointer.is_decidedly_dragging() {
                let mouse_event = MouseEvent {
                    position: pointer_pos,
                    button: if input.pointer.primary_down() {
                        Some(PointerButton::Primary)
                    } else {
                        None
                    },
                    timestamp: current_time,
                    event_type: MouseEventType::Drag,
                };
                
                events.mouse_events.push(mouse_event.clone());
                self.add_to_mouse_history(mouse_event);
            }
        }
        
        // Process hover
        if let Some(hover_pos) = input.pointer.hover_pos() {
            let mouse_event = MouseEvent {
                position: hover_pos,
                button: None,
                timestamp: current_time,
                event_type: MouseEventType::Hover,
            };
            
            // Don't add all hover events to history, only when position changes significantly
            if let Some(last_event) = self.mouse_history.last() {
                if (hover_pos - last_event.position).length() > 10.0 {
                    self.add_to_mouse_history(mouse_event);
                }
            } else {
                self.add_to_mouse_history(mouse_event);
            }
        }
    }
    
    fn process_window_events(&self, input: &egui::InputState, events: &mut InputEvents) {
        for event in &input.events {
            match event {
                egui::Event::WindowFocused(focused) => {
                    events.window_events.push(if *focused {
                        WindowEvent::Focus
                    } else {
                        WindowEvent::Unfocus
                    });
                }
                _ => {}
            }
        }
        
        // Check viewport events  
        let _viewport = input.viewport();
        // Note: egui viewport API changed, we'll skip resize detection for now
        // TODO: Update to use new egui viewport API when needed
    }
    
    fn is_key_combination_pressed(
        &self,
        combination: &KeyCombination,
        modifiers: &Modifiers,
        input: &egui::InputState,
    ) -> bool {
        // Check if key was just pressed (not held)
        let key_pressed = input.key_pressed(combination.key);
        
        if !key_pressed {
            return false;
        }
        
        // Check modifiers
        modifiers.ctrl == combination.ctrl &&
        modifiers.alt == combination.alt &&
        modifiers.shift == combination.shift &&
        modifiers.command == combination.cmd
    }
    
    fn add_to_mouse_history(&mut self, event: MouseEvent) {
        self.mouse_history.push(event);
        
        // Limit history size
        if self.mouse_history.len() > self.max_history_size {
            self.mouse_history.remove(0);
        }
    }
    
    fn setup_default_key_bindings(&mut self) {
        // Tab shortcuts
        self.add_key_binding(Key::T, true, false, false, false, InputAction::NewTab);
        self.add_key_binding(Key::W, true, false, false, false, InputAction::CloseTab);
        self.add_key_binding(Key::Tab, true, false, false, false, InputAction::NextTab);
        self.add_key_binding(Key::Tab, true, false, true, false, InputAction::PrevTab);
        self.add_key_binding(Key::T, true, false, true, false, InputAction::ReopenClosedTab);
        
        // Navigation shortcuts
        self.add_key_binding(Key::ArrowLeft, false, true, false, false, InputAction::Back);
        self.add_key_binding(Key::ArrowRight, false, true, false, false, InputAction::Forward);
        self.add_key_binding(Key::R, true, false, false, false, InputAction::Reload);
        self.add_key_binding(Key::F5, false, false, false, false, InputAction::Reload);
        self.add_key_binding(Key::Home, false, true, false, false, InputAction::Home);
        self.add_key_binding(Key::Escape, false, false, false, false, InputAction::Stop);
        
        // Browser shortcuts
        self.add_key_binding(Key::F, true, false, false, false, InputAction::Find);
        self.add_key_binding(Key::G, true, false, false, false, InputAction::FindNext);
        self.add_key_binding(Key::G, true, false, true, false, InputAction::FindPrev);
        // Plus key doesn't exist in this egui version, use PlusEquals instead
        self.add_key_binding(Key::PlusEquals, true, false, false, false, InputAction::ZoomIn);
        self.add_key_binding(Key::Minus, true, false, false, false, InputAction::ZoomOut);
        self.add_key_binding(Key::Num0, true, false, false, false, InputAction::ZoomReset);
        self.add_key_binding(Key::F11, false, false, false, false, InputAction::ToggleFullscreen);
        
        // Developer tools
        self.add_key_binding(Key::F12, false, false, false, false, InputAction::ToggleDevTools);
        self.add_key_binding(Key::I, true, false, true, false, InputAction::ToggleInspector);
        self.add_key_binding(Key::J, true, false, true, false, InputAction::ShowConsole);
        
        // Genesis specific
        self.add_key_binding(Key::G, true, false, true, false, InputAction::ToggleGenesisMode);
        self.add_key_binding(Key::E, true, false, true, false, InputAction::ShowGenesisExplorer);
        
        // General shortcuts
        self.add_key_binding(Key::Q, true, false, false, false, InputAction::Quit);
        self.add_key_binding(Key::C, true, false, false, false, InputAction::Copy);
        self.add_key_binding(Key::V, true, false, false, false, InputAction::Paste);
        self.add_key_binding(Key::X, true, false, false, false, InputAction::Cut);
        self.add_key_binding(Key::A, true, false, false, false, InputAction::SelectAll);
        
        // Number keys for tab switching (Ctrl+1, Ctrl+2, etc.)
        for i in 1..=9 {
            if let Some(key) = num_key_from_digit(i) {
                self.key_combinations.insert(
                    KeyCombination { key, ctrl: true, alt: false, shift: false, cmd: false },
                    InputAction::NextTab, // We'll handle the specific tab switching in the UI
                );
            }
        }
    }
    
    fn add_key_binding(&mut self, key: Key, ctrl: bool, alt: bool, shift: bool, cmd: bool, action: InputAction) {
        let combination = KeyCombination { key, ctrl, alt, shift, cmd };
        self.key_combinations.insert(combination, action);
    }
    
    // === Public API ===
    
    pub fn add_custom_key_binding(&mut self, key: Key, ctrl: bool, alt: bool, shift: bool, cmd: bool, action: InputAction) {
        self.add_key_binding(key, ctrl, alt, shift, cmd, action);
    }
    
    pub fn remove_key_binding(&mut self, key: Key, ctrl: bool, alt: bool, shift: bool, cmd: bool) {
        let combination = KeyCombination { key, ctrl, alt, shift, cmd };
        self.key_combinations.remove(&combination);
    }
    
    pub fn get_mouse_history(&self) -> &Vec<MouseEvent> {
        &self.mouse_history
    }
    
    pub fn clear_mouse_history(&mut self) {
        self.mouse_history.clear();
    }
    
    pub fn is_key_pressed(&self, key: &Key) -> bool {
        self.pressed_keys.contains(key)
    }
    
    pub fn get_pressed_keys(&self) -> &std::collections::HashSet<Key> {
        &self.pressed_keys
    }
    
    // === Settings ===
    
    pub fn set_double_click_threshold(&mut self, threshold: f64) {
        self.double_click_threshold = threshold;
    }
    
    pub fn set_drag_threshold(&mut self, threshold: f32) {
        self.drag_threshold = threshold;
    }
    
    pub fn set_repeat_settings(&mut self, delay: f64, rate: f64) {
        self.repeat_delay = delay;
        self.repeat_rate = rate;
    }
}

// Helper function to convert digit to Key
fn num_key_from_digit(digit: u8) -> Option<Key> {
    match digit {
        1 => Some(Key::Num1),
        2 => Some(Key::Num2),
        3 => Some(Key::Num3),
        4 => Some(Key::Num4),
        5 => Some(Key::Num5),
        6 => Some(Key::Num6),
        7 => Some(Key::Num7),
        8 => Some(Key::Num8),
        9 => Some(Key::Num9),
        _ => None,
    }
}