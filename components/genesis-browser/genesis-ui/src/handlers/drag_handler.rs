// Drag Handler - Single Responsibility: Sürükle/bırak işlemleri
//
// Bu handler sadece drag & drop işlemlerini yönetir:
// - Tab reordering
// - File drag & drop
// - Window dragging
// - Content selection dragging

use egui::Pos2;

pub struct DragHandler {
    current_drag: Option<DragOperation>,
    drag_threshold: f32,
}

#[derive(Clone, Debug)]
pub struct DragOperation {
    pub drag_type: DragType,
    pub start_pos: Pos2,
    pub current_pos: Pos2,
    pub data: DragData,
}

#[derive(Clone, Debug)]
pub enum DragType {
    Tab,
    File,
    Window,
    Selection,
}

#[derive(Clone, Debug)]
pub enum DragData {
    Tab { index: usize, width: f32 },
    File { path: String, size: u64 },
    Window,
    Selection { text: String },
}

impl Default for DragHandler {
    fn default() -> Self {
        Self {
            current_drag: None,
            drag_threshold: 5.0,
        }
    }
}

impl DragHandler {
    pub fn new() -> Self {
        Self::default()
    }
    
    pub fn start_drag(&mut self, drag_type: DragType, start_pos: Pos2, data: DragData) {
        self.current_drag = Some(DragOperation {
            drag_type,
            start_pos,
            current_pos: start_pos,
            data,
        });
    }
    
    pub fn update_drag(&mut self, current_pos: Pos2) -> bool {
        if let Some(ref mut drag) = self.current_drag {
            drag.current_pos = current_pos;
            true
        } else {
            false
        }
    }
    
    pub fn end_drag(&mut self) -> Option<DragOperation> {
        self.current_drag.take()
    }
    
    pub fn is_dragging(&self) -> bool {
        self.current_drag.is_some()
    }
    
    pub fn get_current_drag(&self) -> Option<&DragOperation> {
        self.current_drag.as_ref()
    }
}