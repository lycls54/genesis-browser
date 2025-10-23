// Animation Controller - Single Responsibility: Animasyon kontrolü
//
// Bu controller sadece animasyonları yönetir:
// - Tab animasyonları
// - UI transition'ları
// - Loading animasyonları
// - Smooth scrolling

use std::collections::HashMap;

pub struct AnimationController {
    animations: HashMap<String, Animation>,
    global_time: f32,
    animation_speed: f32,
}

#[derive(Clone, Debug)]
pub struct Animation {
    pub start_time: f32,
    pub duration: f32,
    pub animation_type: AnimationType,
    pub easing: EasingFunction,
    pub start_value: f32,
    pub end_value: f32,
    pub current_value: f32,
}

#[derive(Clone, Debug)]
pub enum AnimationType {
    TabOpen,
    TabClose,
    FadeIn,
    FadeOut,
    SlideIn,
    SlideOut,
    Scale,
}

#[derive(Clone, Debug)]
pub enum EasingFunction {
    Linear,
    EaseIn,
    EaseOut,
    EaseInOut,
    Bounce,
}

impl Default for AnimationController {
    fn default() -> Self {
        Self {
            animations: HashMap::new(),
            global_time: 0.0,
            animation_speed: 1.0,
        }
    }
}

impl AnimationController {
    pub fn new() -> Self {
        Self::default()
    }
    
    pub fn update(&mut self, delta_time: f32) {
        self.global_time += delta_time * self.animation_speed;
        
        // Update all animations
        let mut finished_animations = Vec::new();
        
        for (id, animation) in &mut self.animations {
            let elapsed = self.global_time - animation.start_time;
            let progress = (elapsed / animation.duration).clamp(0.0, 1.0);
            
            let eased_progress = apply_easing(progress, &animation.easing);
            animation.current_value = lerp(animation.start_value, animation.end_value, eased_progress);
            
            if progress >= 1.0 {
                finished_animations.push(id.clone());
            }
        }
        
        // Remove finished animations
        for id in finished_animations {
            self.animations.remove(&id);
        }
    }
    
    pub fn start_animation(&mut self, id: String, animation_type: AnimationType, duration: f32, start_value: f32, end_value: f32) {
        let animation = Animation {
            start_time: self.global_time,
            duration,
            animation_type,
            easing: EasingFunction::EaseOut,
            start_value,
            end_value,
            current_value: start_value,
        };
        
        self.animations.insert(id, animation);
    }
    
    pub fn get_animation_value(&self, id: &str) -> Option<f32> {
        self.animations.get(id).map(|anim| anim.current_value)
    }
    
    pub fn is_animating(&self, id: &str) -> bool {
        self.animations.contains_key(id)
    }
    
    pub fn stop_animation(&mut self, id: &str) {
        self.animations.remove(id);
    }
}

fn apply_easing(t: f32, easing: &EasingFunction) -> f32 {
    match easing {
        EasingFunction::Linear => t,
        EasingFunction::EaseIn => t * t,
        EasingFunction::EaseOut => 1.0 - (1.0 - t).powi(2),
        EasingFunction::EaseInOut => if t < 0.5 { 2.0 * t * t } else { 1.0 - 2.0 * (1.0 - t).powi(2) },
        EasingFunction::Bounce => {
            if t < 1.0 / 2.75 {
                7.5625 * t * t
            } else if t < 2.0 / 2.75 {
                let t = t - 1.5 / 2.75;
                7.5625 * t * t + 0.75
            } else if t < 2.5 / 2.75 {
                let t = t - 2.25 / 2.75;
                7.5625 * t * t + 0.9375
            } else {
                let t = t - 2.625 / 2.75;
                7.5625 * t * t + 0.984375
            }
        }
    }
}

fn lerp(start: f32, end: f32, t: f32) -> f32 {
    start + (end - start) * t
}