// Performance Monitor - Single Responsibility: Performans metrikleri
//
// Bu manager sadece performans ölçümü ve izleme ile ilgili işlemleri yönetir:
// - FPS tracking
// - Memory usage monitoring
// - Render time measurement
// - Performance statistics

use std::time::{Duration, Instant};
use std::collections::VecDeque;

pub struct PerformanceMonitor {
    // FPS tracking
    frame_times: VecDeque<Duration>,
    last_frame_time: Instant,
    current_fps: f32,
    target_fps: f32,
    
    // Render metrics
    render_start_time: Option<Instant>,
    last_render_duration: Duration,
    average_render_time: Duration,
    peak_render_time: Duration,
    
    // Memory tracking
    memory_samples: VecDeque<MemorySample>,
    
    // Performance settings
    max_samples: usize,
    update_interval: Duration,
    last_update: Instant,
    
    // Statistics
    total_frames: u64,
    dropped_frames: u64,
    performance_warnings: Vec<PerformanceWarning>,
}

#[derive(Clone, Debug)]
pub struct MemorySample {
    pub timestamp: Instant,
    pub heap_size: usize,
    pub used_memory: usize,
    pub allocated_objects: usize,
}

#[derive(Clone, Debug)]
pub struct PerformanceStats {
    pub current_fps: f32,
    pub average_fps: f32,
    pub min_fps: f32,
    pub max_fps: f32,
    pub frame_time_ms: f32,
    pub render_time_ms: f32,
    pub memory_usage_mb: f32,
    pub total_frames: u64,
    pub dropped_frames: u64,
    pub cpu_usage: f32,
}

#[derive(Clone, Debug)]
pub struct PerformanceWarning {
    pub warning_type: WarningType,
    pub message: String,
    pub timestamp: Instant,
    pub severity: WarningSeverity,
}

#[derive(Clone, Debug, PartialEq)]
pub enum WarningType {
    LowFPS,
    HighMemoryUsage,
    SlowRenderTime,
    DroppedFrames,
    CPUThrottle,
}

#[derive(Clone, Debug, PartialEq)]
pub enum WarningSeverity {
    Info,
    Warning,
    Critical,
}

impl Default for PerformanceMonitor {
    fn default() -> Self {
        Self {
            frame_times: VecDeque::new(),
            last_frame_time: Instant::now(),
            current_fps: 60.0,
            target_fps: 144.0,
            
            render_start_time: None,
            last_render_duration: Duration::from_millis(0),
            average_render_time: Duration::from_millis(0),
            peak_render_time: Duration::from_millis(0),
            
            memory_samples: VecDeque::new(),
            
            max_samples: 120, // 2 saniye @ 60fps
            update_interval: Duration::from_millis(16), // ~60Hz update
            last_update: Instant::now(),
            
            total_frames: 0,
            dropped_frames: 0,
            performance_warnings: Vec::new(),
        }
    }
}

impl PerformanceMonitor {
    pub fn new() -> Self {
        Self::default()
    }
    
    pub fn new_with_target_fps(target_fps: f32) -> Self {
        let mut monitor = Self::default();
        monitor.target_fps = target_fps;
        monitor
    }
    
    // === Frame Tracking ===
    
    pub fn start_frame(&mut self) {
        let now = Instant::now();
        let frame_time = now.duration_since(self.last_frame_time);
        
        // Frame time'ı kaydet
        self.frame_times.push_back(frame_time);
        if self.frame_times.len() > self.max_samples {
            self.frame_times.pop_front();
        }
        
        self.last_frame_time = now;
        self.total_frames += 1;
        
        // FPS hesapla
        self.update_fps();
        
        // Render başlangıcını kaydet
        self.render_start_time = Some(now);
    }
    
    pub fn end_frame(&mut self) {
        if let Some(start_time) = self.render_start_time.take() {
            let render_duration = Instant::now().duration_since(start_time);
            
            self.last_render_duration = render_duration;
            self.update_render_stats(render_duration);
            
            // Performance uyarıları kontrol et
            self.check_performance_warnings();
        }
    }
    
    fn update_fps(&mut self) {
        if self.frame_times.is_empty() {
            return;
        }
        
        // Son 60 frame'in ortalamasını al
        let sample_count = self.frame_times.len().min(60);
        let total_time: Duration = self.frame_times.iter()
            .rev()
            .take(sample_count)
            .sum();
            
        if !total_time.is_zero() {
            let average_frame_time = total_time / sample_count as u32;
            self.current_fps = 1.0 / average_frame_time.as_secs_f32();
            
            // FPS limitlerini kontrol et
            if self.current_fps > self.target_fps * 1.1 {
                self.current_fps = self.target_fps; // Limit FPS to target
            }
        }
    }
    
    // === Render Time Tracking ===
    
    fn update_render_stats(&mut self, render_duration: Duration) {
        // Average render time güncelle (exponential moving average)
        let alpha = 0.1; // Smoothing factor
        let new_render_time = render_duration.as_secs_f32();
        let current_avg = self.average_render_time.as_secs_f32();
        
        let updated_avg = current_avg * (1.0 - alpha) + new_render_time * alpha;
        self.average_render_time = Duration::from_secs_f32(updated_avg);
        
        // Peak render time güncelle
        if render_duration > self.peak_render_time {
            self.peak_render_time = render_duration;
        }
    }
    
    // === Memory Tracking ===
    
    pub fn record_memory_sample(&mut self, heap_size: usize, used_memory: usize, allocated_objects: usize) {
        let sample = MemorySample {
            timestamp: Instant::now(),
            heap_size,
            used_memory,
            allocated_objects,
        };
        
        self.memory_samples.push_back(sample);
        
        // Eski örnekleri temizle (son 5 dakika)
        let cutoff_time = Instant::now() - Duration::from_secs(300);
        while let Some(front) = self.memory_samples.front() {
            if front.timestamp < cutoff_time {
                self.memory_samples.pop_front();
            } else {
                break;
            }
        }
    }
    
    // === Performance Statistics ===
    
    pub fn get_performance_stats(&self) -> PerformanceStats {
        let (min_fps, max_fps, avg_fps) = self.calculate_fps_stats();
        let memory_usage_mb = self.get_current_memory_usage_mb();
        
        PerformanceStats {
            current_fps: self.current_fps,
            average_fps: avg_fps,
            min_fps,
            max_fps,
            frame_time_ms: if self.current_fps > 0.0 { 
                1000.0 / self.current_fps 
            } else { 
                0.0 
            },
            render_time_ms: self.last_render_duration.as_secs_f32() * 1000.0,
            memory_usage_mb,
            total_frames: self.total_frames,
            dropped_frames: self.dropped_frames,
            cpu_usage: self.estimate_cpu_usage(),
        }
    }
    
    fn calculate_fps_stats(&self) -> (f32, f32, f32) {
        if self.frame_times.is_empty() {
            return (0.0, 0.0, 0.0);
        }
        
        let fps_values: Vec<f32> = self.frame_times.iter()
            .map(|duration| 1.0 / duration.as_secs_f32())
            .collect();
            
        let min_fps = fps_values.iter().fold(f32::INFINITY, |a, &b| a.min(b));
        let max_fps = fps_values.iter().fold(0.0f32, |a, &b| a.max(b));
        let avg_fps = fps_values.iter().sum::<f32>() / fps_values.len() as f32;
        
        (min_fps, max_fps, avg_fps)
    }
    
    fn get_current_memory_usage_mb(&self) -> f32 {
        if let Some(latest_sample) = self.memory_samples.back() {
            latest_sample.used_memory as f32 / (1024.0 * 1024.0)
        } else {
            0.0
        }
    }
    
    fn estimate_cpu_usage(&self) -> f32 {
        // Basit CPU usage tahmini (render time based)
        let target_frame_time = 1.0 / self.target_fps;
        let actual_frame_time = self.average_render_time.as_secs_f32();
        
        (actual_frame_time / target_frame_time * 100.0).min(100.0)
    }
    
    // === Performance Warnings ===
    
    fn check_performance_warnings(&mut self) {
        let stats = self.get_performance_stats();
        
        // FPS uyarıları
        if stats.current_fps < self.target_fps * 0.5 {
            self.add_warning(WarningType::LowFPS, 
                format!("FPS dropped to {:.1} (target: {:.1})", stats.current_fps, self.target_fps),
                WarningSeverity::Critical);
        } else if stats.current_fps < self.target_fps * 0.75 {
            self.add_warning(WarningType::LowFPS, 
                format!("FPS below target: {:.1}/{:.1}", stats.current_fps, self.target_fps),
                WarningSeverity::Warning);
        }
        
        // Memory uyarıları
        if stats.memory_usage_mb > 500.0 {
            self.add_warning(WarningType::HighMemoryUsage, 
                format!("High memory usage: {:.1}MB", stats.memory_usage_mb),
                WarningSeverity::Warning);
        }
        
        // Render time uyarıları
        if stats.render_time_ms > 16.67 { // 60 FPS threshold
            self.add_warning(WarningType::SlowRenderTime, 
                format!("Slow render time: {:.2}ms", stats.render_time_ms),
                WarningSeverity::Warning);
        }
        
        // Dropped frames
        let drop_rate = if self.total_frames > 0 {
            (self.dropped_frames as f32 / self.total_frames as f32) * 100.0
        } else {
            0.0
        };
        
        if drop_rate > 5.0 {
            self.add_warning(WarningType::DroppedFrames, 
                format!("High frame drop rate: {:.1}%", drop_rate),
                WarningSeverity::Critical);
        }
    }
    
    fn add_warning(&mut self, warning_type: WarningType, message: String, severity: WarningSeverity) {
        // Aynı tip uyarıların spam'ini engelle
        let now = Instant::now();
        let recent_cutoff = now - Duration::from_secs(5);
        
        let has_recent_warning = self.performance_warnings.iter()
            .any(|w| w.warning_type == warning_type && w.timestamp > recent_cutoff);
            
        if !has_recent_warning {
            self.performance_warnings.push(PerformanceWarning {
                warning_type,
                message,
                timestamp: now,
                severity,
            });
            
            // Eski uyarıları temizle
            self.performance_warnings.retain(|w| w.timestamp > now - Duration::from_secs(60));
        }
    }
    
    // === Getters ===
    
    pub fn get_current_fps(&self) -> f32 {
        self.current_fps
    }
    
    pub fn get_target_fps(&self) -> f32 {
        self.target_fps
    }
    
    pub fn get_frame_time_ms(&self) -> f32 {
        if self.current_fps > 0.0 {
            1000.0 / self.current_fps
        } else {
            0.0
        }
    }
    
    pub fn get_render_time_ms(&self) -> f32 {
        self.last_render_duration.as_secs_f32() * 1000.0
    }
    
    pub fn get_warnings(&self) -> &Vec<PerformanceWarning> {
        &self.performance_warnings
    }
    
    pub fn get_memory_samples(&self) -> &VecDeque<MemorySample> {
        &self.memory_samples
    }
    
    // === Settings ===
    
    pub fn set_target_fps(&mut self, target_fps: f32) {
        self.target_fps = target_fps.max(1.0).min(240.0); // Reasonable limits
    }
    
    pub fn set_max_samples(&mut self, max_samples: usize) {
        self.max_samples = max_samples;
        
        // Mevcut samples'ı sınırla
        while self.frame_times.len() > max_samples {
            self.frame_times.pop_front();
        }
    }
    
    pub fn clear_statistics(&mut self) {
        self.frame_times.clear();
        self.memory_samples.clear();
        self.performance_warnings.clear();
        self.total_frames = 0;
        self.dropped_frames = 0;
        self.peak_render_time = Duration::from_millis(0);
    }
    
    // === Utility Methods ===
    
    pub fn is_performance_good(&self) -> bool {
        let stats = self.get_performance_stats();
        stats.current_fps >= self.target_fps * 0.8 && 
        stats.render_time_ms < 16.67 &&
        stats.memory_usage_mb < 200.0
    }
    
    pub fn get_performance_grade(&self) -> PerformanceGrade {
        let stats = self.get_performance_stats();
        let fps_ratio = stats.current_fps / self.target_fps;
        
        if fps_ratio >= 0.95 && stats.render_time_ms < 10.0 {
            PerformanceGrade::Excellent
        } else if fps_ratio >= 0.8 && stats.render_time_ms < 16.67 {
            PerformanceGrade::Good
        } else if fps_ratio >= 0.5 {
            PerformanceGrade::Fair
        } else {
            PerformanceGrade::Poor
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum PerformanceGrade {
    Excellent,
    Good,
    Fair,
    Poor,
}