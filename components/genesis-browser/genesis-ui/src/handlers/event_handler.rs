// Event Handler - Single Responsibility: UI event işleme
//
// Bu handler sadece UI event'lerinin işlenmesi ve yönlendirilmesi ile ilgilenir:
// - Tab eventi yönlendirme
// - Navigation eventi yönlendirme  
// - Panel eventi yönlendirme
// - Window eventi yönlendirme

use crate::managers::{TabManager, NavigationManager};
use crate::handlers::input_handler::{InputAction, InputEvents};

pub struct EventHandler {
    // Event queues
    pending_tab_events: Vec<TabEvent>,
    pending_navigation_events: Vec<NavigationEvent>,
    pending_panel_events: Vec<PanelEvent>,
    pending_window_events: Vec<WindowEvent>,
    
    // Event processing settings
    max_events_per_frame: usize,
    event_processing_enabled: bool,
}

#[derive(Clone, Debug)]
pub enum TabEvent {
    Create(String), // URL
    Close(usize),   // Tab index
    Switch(usize),  // Tab index
    Move(usize, usize), // From index, To index
    Duplicate(usize), // Tab index
    Pin(usize),     // Tab index
    Unpin(usize),   // Tab index
    Mute(usize),    // Tab index
    Unmute(usize),  // Tab index
}

#[derive(Clone, Debug)]
pub enum NavigationEvent {
    Navigate(String),  // URL
    Back,
    Forward,
    Reload,
    Stop,
    Home,
    Search(String),   // Query
    BookmarkPage,
    SharePage,
}

#[derive(Clone, Debug)]
pub enum PanelEvent {
    ShowBookmarks,
    HideBookmarks,
    ShowDownloads,
    HideDownloads,
    ShowHistory,
    HideHistory,
    ShowSettings,
    HideSettings,
    ShowDevTools,
    HideDevTools,
    TogglePanel(PanelType),
}

#[derive(Clone, Debug, PartialEq)]
pub enum PanelType {
    Bookmarks,
    Downloads,
    History,
    Settings,
    DevTools,
}

#[derive(Clone, Debug)]
pub enum WindowEvent {
    Minimize,
    Maximize,
    Restore,
    Close,
    Fullscreen,
    ExitFullscreen,
    Focus,
    Blur,
    Resize(egui::Vec2),
}

#[derive(Clone, Debug)]
pub struct EventProcessingResult {
    pub tab_actions_performed: usize,
    pub navigation_actions_performed: usize,
    pub panel_actions_performed: usize,
    pub window_actions_performed: usize,
    pub events_dropped: usize,
}

impl Default for EventHandler {
    fn default() -> Self {
        Self {
            pending_tab_events: Vec::new(),
            pending_navigation_events: Vec::new(),
            pending_panel_events: Vec::new(),
            pending_window_events: Vec::new(),
            max_events_per_frame: 50,
            event_processing_enabled: true,
        }
    }
}

impl EventHandler {
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Process all pending events and input actions
    pub fn process_events(
        &mut self,
        input_events: &InputEvents,
        tab_manager: &mut TabManager,
        navigation_manager: &mut NavigationManager,
    ) -> EventProcessingResult {
        let mut result = EventProcessingResult {
            tab_actions_performed: 0,
            navigation_actions_performed: 0,
            panel_actions_performed: 0,
            window_actions_performed: 0,
            events_dropped: 0,
        };
        
        if !self.event_processing_enabled {
            return result;
        }
        
        // Convert input actions to events
        self.convert_input_actions_to_events(&input_events.actions);
        
        // Process events with frame limits
        let mut total_processed = 0;
        
        // Process tab events
        let tab_events_processed = self.process_tab_events(tab_manager, &mut total_processed);
        result.tab_actions_performed = tab_events_processed;
        
        // Process navigation events
        if total_processed < self.max_events_per_frame {
            let nav_events_processed = self.process_navigation_events(navigation_manager, &mut total_processed);
            result.navigation_actions_performed = nav_events_processed;
        }
        
        // Process panel events
        if total_processed < self.max_events_per_frame {
            let panel_events_processed = self.process_panel_events(&mut total_processed);
            result.panel_actions_performed = panel_events_processed;
        }
        
        // Process window events
        if total_processed < self.max_events_per_frame {
            let window_events_processed = self.process_window_events(&mut total_processed);
            result.window_actions_performed = window_events_processed;
        }
        
        // Count dropped events
        result.events_dropped = self.pending_tab_events.len() +
                               self.pending_navigation_events.len() +
                               self.pending_panel_events.len() +
                               self.pending_window_events.len();
        
        result
    }
    
    fn convert_input_actions_to_events(&mut self, actions: &[InputAction]) {
        for action in actions {
            match action {
                // Tab actions
                InputAction::NewTab => {
                    self.queue_tab_event(TabEvent::Create("genesis://welcome".to_string()));
                },
                InputAction::CloseTab => {
                    // Close current tab - we'll get the current index from tab manager
                    self.queue_tab_event(TabEvent::Close(usize::MAX)); // Special value for current tab
                },
                InputAction::NextTab => {
                    self.queue_tab_event(TabEvent::Switch(usize::MAX - 1)); // Special value for next
                },
                InputAction::PrevTab => {
                    self.queue_tab_event(TabEvent::Switch(usize::MAX - 2)); // Special value for prev
                },
                InputAction::ReopenClosedTab => {
                    // TODO: Implement recently closed tabs tracking
                    self.queue_tab_event(TabEvent::Create("genesis://welcome".to_string()));
                },
                
                // Navigation actions
                InputAction::Back => {
                    self.queue_navigation_event(NavigationEvent::Back);
                },
                InputAction::Forward => {
                    self.queue_navigation_event(NavigationEvent::Forward);
                },
                InputAction::Reload => {
                    self.queue_navigation_event(NavigationEvent::Reload);
                },
                InputAction::Home => {
                    self.queue_navigation_event(NavigationEvent::Home);
                },
                InputAction::Stop => {
                    self.queue_navigation_event(NavigationEvent::Stop);
                },
                
                // Panel actions
                InputAction::ToggleDevTools => {
                    self.queue_panel_event(PanelEvent::TogglePanel(PanelType::DevTools));
                },
                
                // Genesis specific actions
                InputAction::ToggleGenesisMode => {
                    // TODO: Implement Genesis mode toggle
                },
                InputAction::ShowGenesisExplorer => {
                    self.queue_navigation_event(NavigationEvent::Navigate("genesis://explorer".to_string()));
                },
                
                // Window actions
                InputAction::ToggleFullscreen => {
                    self.queue_window_event(WindowEvent::Fullscreen);
                },
                InputAction::Quit => {
                    self.queue_window_event(WindowEvent::Close);
                },
                
                _ => {
                    // Handle other actions as needed
                }
            }
        }
    }
    
    fn process_tab_events(&mut self, tab_manager: &mut TabManager, total_processed: &mut usize) -> usize {
        let mut processed = 0;
        let max_to_process = (self.max_events_per_frame - *total_processed).min(self.pending_tab_events.len());
        
        for _ in 0..max_to_process {
            if let Some(event) = self.pending_tab_events.pop() {
                match event {
                    TabEvent::Create(url) => {
                        tab_manager.create_tab(&url);
                    },
                    TabEvent::Close(index) => {
                        let actual_index = if index == usize::MAX {
                            tab_manager.get_active_tab_index()
                        } else {
                            index
                        };
                        tab_manager.close_tab(actual_index);
                    },
                    TabEvent::Switch(index) => {
                        const NEXT_TAB_MARKER: usize = usize::MAX - 1;
                        const PREV_TAB_MARKER: usize = usize::MAX - 2;
                        
                        let actual_index = if index == NEXT_TAB_MARKER {
                            // Next tab
                            let current = tab_manager.get_active_tab_index();
                            (current + 1) % tab_manager.get_tab_count()
                        } else if index == PREV_TAB_MARKER {
                            // Previous tab
                            let current = tab_manager.get_active_tab_index();
                            if current == 0 {
                                tab_manager.get_tab_count() - 1
                            } else {
                                current - 1
                            }
                        } else {
                            index
                        };
                        tab_manager.switch_to_tab(actual_index);
                    },
                    TabEvent::Move(_from, _to) => {
                        // TODO: Implement tab reordering in TabManager
                    },
                    TabEvent::Duplicate(index) => {
                        if let Some(tab) = tab_manager.get_tab(index) {
                            let url = tab.url.clone();
                            tab_manager.create_tab(&url);
                        }
                    },
                    _ => {
                        // Handle other tab events
                    }
                }
                processed += 1;
                *total_processed += 1;
            }
        }
        
        processed
    }
    
    fn process_navigation_events(&mut self, navigation_manager: &mut NavigationManager, total_processed: &mut usize) -> usize {
        let mut processed = 0;
        let max_to_process = (self.max_events_per_frame - *total_processed).min(self.pending_navigation_events.len());
        
        for _ in 0..max_to_process {
            if let Some(event) = self.pending_navigation_events.pop() {
                match event {
                    NavigationEvent::Navigate(url) => {
                        navigation_manager.navigate_to(url);
                    },
                    NavigationEvent::Back => {
                        navigation_manager.go_back();
                    },
                    NavigationEvent::Forward => {
                        // TODO: Implement forward navigation
                    },
                    NavigationEvent::Reload => {
                        navigation_manager.reload();
                    },
                    NavigationEvent::Home => {
                        navigation_manager.go_home();
                    },
                    NavigationEvent::Stop => {
                        // TODO: Implement stop loading
                    },
                    NavigationEvent::Search(query) => {
                        let search_url = navigation_manager.build_search_url(&query);
                        navigation_manager.navigate_to(search_url);
                    },
                    _ => {
                        // Handle other navigation events
                    }
                }
                processed += 1;
                *total_processed += 1;
            }
        }
        
        processed
    }
    
    fn process_panel_events(&mut self, total_processed: &mut usize) -> usize {
        let mut processed = 0;
        let max_to_process = (self.max_events_per_frame - *total_processed).min(self.pending_panel_events.len());
        
        for _ in 0..max_to_process {
            if let Some(event) = self.pending_panel_events.pop() {
                match event {
                    PanelEvent::TogglePanel(_panel_type) => {
                        // Panel visibility will be handled by the main browser app
                        // This is just event processing
                    },
                    _ => {
                        // Handle other panel events
                    }
                }
                processed += 1;
                *total_processed += 1;
            }
        }
        
        processed
    }
    
    fn process_window_events(&mut self, total_processed: &mut usize) -> usize {
        let mut processed = 0;
        let max_to_process = (self.max_events_per_frame - *total_processed).min(self.pending_window_events.len());
        
        for _ in 0..max_to_process {
            if let Some(event) = self.pending_window_events.pop() {
                match event {
                    WindowEvent::Close => {
                        // Window close will be handled by the main application
                    },
                    WindowEvent::Minimize => {
                        // TODO: Implement window minimize
                    },
                    WindowEvent::Maximize => {
                        // TODO: Implement window maximize
                    },
                    WindowEvent::Fullscreen => {
                        // TODO: Implement fullscreen toggle
                    },
                    _ => {
                        // Handle other window events
                    }
                }
                processed += 1;
                *total_processed += 1;
            }
        }
        
        processed
    }
    
    // === Public Event Queue Methods ===
    
    pub fn queue_tab_event(&mut self, event: TabEvent) {
        self.pending_tab_events.push(event);
    }
    
    pub fn queue_navigation_event(&mut self, event: NavigationEvent) {
        self.pending_navigation_events.push(event);
    }
    
    pub fn queue_panel_event(&mut self, event: PanelEvent) {
        self.pending_panel_events.push(event);
    }
    
    pub fn queue_window_event(&mut self, event: WindowEvent) {
        self.pending_window_events.push(event);
    }
    
    // === Event Queue Management ===
    
    pub fn clear_all_events(&mut self) {
        self.pending_tab_events.clear();
        self.pending_navigation_events.clear();
        self.pending_panel_events.clear();
        self.pending_window_events.clear();
    }
    
    pub fn get_pending_event_count(&self) -> usize {
        self.pending_tab_events.len() +
        self.pending_navigation_events.len() +
        self.pending_panel_events.len() +
        self.pending_window_events.len()
    }
    
    pub fn has_pending_events(&self) -> bool {
        self.get_pending_event_count() > 0
    }
    
    // === Settings ===
    
    pub fn set_max_events_per_frame(&mut self, max_events: usize) {
        self.max_events_per_frame = max_events;
    }
    
    pub fn enable_event_processing(&mut self, enabled: bool) {
        self.event_processing_enabled = enabled;
    }
    
    pub fn is_event_processing_enabled(&self) -> bool {
        self.event_processing_enabled
    }
    
    // === Event Introspection ===
    
    pub fn get_tab_event_count(&self) -> usize {
        self.pending_tab_events.len()
    }
    
    pub fn get_navigation_event_count(&self) -> usize {
        self.pending_navigation_events.len()
    }
    
    pub fn get_panel_event_count(&self) -> usize {
        self.pending_panel_events.len()
    }
    
    pub fn get_window_event_count(&self) -> usize {
        self.pending_window_events.len()
    }
}