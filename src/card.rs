use iced::{Color, Point, Rectangle};
use crate::custom_text_editor::CustomTextEditor;

#[derive(Debug)]
pub struct Card {
    pub id: usize,
    pub current_position: Point,
    pub target_position: Point,
    pub width: f32,
    pub height: f32,
    pub target_width: f32,
    pub target_height: f32,
    pub icon: CardIcon,
    pub color: Color,
    pub is_dragging: bool,
    pub content: CustomTextEditor,
    pub is_editing: bool,
}

impl Clone for Card {
    fn clone(&self) -> Self {
        Self {
            id: self.id,
            current_position: self.current_position,
            target_position: self.target_position,
            width: self.width,
            height: self.height,
            target_width: self.target_width,
            target_height: self.target_height,
            icon: self.icon,
            color: self.color,
            is_dragging: self.is_dragging,
            content: self.content.clone(),
            is_editing: self.is_editing,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CardIcon {
    Default,
    Star,
    Heart,
    Circle,
    Square,
    Triangle,
    Check,
    Cross,
    Question,
    Exclamation,
    Plus,
    Minus,
}

impl CardIcon {
    pub fn svg_data(&self) -> &'static [u8] {
        match self {
            CardIcon::Default => include_bytes!("icons/default.svg"),
            CardIcon::Star => include_bytes!("icons/star.svg"),
            CardIcon::Heart => include_bytes!("icons/heart.svg"),
            CardIcon::Circle => include_bytes!("icons/circle.svg"),
            CardIcon::Square => include_bytes!("icons/square.svg"),
            CardIcon::Triangle => include_bytes!("icons/triangle.svg"),
            CardIcon::Check => include_bytes!("icons/check.svg"),
            CardIcon::Cross => include_bytes!("icons/cross.svg"),
            CardIcon::Question => include_bytes!("icons/question.svg"),
            CardIcon::Exclamation => include_bytes!("icons/exclamation.svg"),
            CardIcon::Plus => include_bytes!("icons/plus.svg"),
            CardIcon::Minus => include_bytes!("icons/minus.svg"),
        }
    }

    pub fn all() -> &'static [CardIcon] {
        &[
            CardIcon::Default,
            CardIcon::Star,
            CardIcon::Heart,
            CardIcon::Circle,
            CardIcon::Square,
            CardIcon::Triangle,
            CardIcon::Check,
            CardIcon::Cross,
            CardIcon::Question,
            CardIcon::Exclamation,
            CardIcon::Plus,
            CardIcon::Minus,
        ]
    }
}

impl Card {
    pub const MIN_WIDTH: f32 = 200.0;
    pub const MIN_HEIGHT: f32 = 150.0;

    pub fn new(id: usize, position: Point) -> Self {
        Self {
            id,
            current_position: position,
            target_position: position,
            width: Self::MIN_WIDTH,
            height: Self::MIN_HEIGHT,
            target_width: Self::MIN_WIDTH,
            target_height: Self::MIN_HEIGHT,
            icon: CardIcon::Default,
            color: Color::from_rgb8(100, 150, 255), // Default blue
            is_dragging: false,
            content: CustomTextEditor::new(),
            is_editing: false,
        }
    }

    pub fn bounds(&self) -> Rectangle {
        Rectangle {
            x: self.current_position.x,
            y: self.current_position.y,
            width: self.width,
            height: self.height,
        }
    }

    pub fn top_bar_bounds(&self) -> Rectangle {
        Rectangle {
            x: self.current_position.x,
            y: self.current_position.y,
            width: self.width,
            height: 30.0, // Top bar height
        }
    }

    pub fn icon_bounds(&self) -> Rectangle {
        Rectangle {
            x: self.current_position.x + 5.0,
            y: self.current_position.y + 5.0,
            width: 20.0,
            height: 20.0,
        }
    }

    pub fn content_bounds(&self) -> Rectangle {
        Rectangle {
            x: self.current_position.x,
            y: self.current_position.y + 30.0, // Below the top bar
            width: self.width,
            height: self.height - 30.0,
        }
    }

    pub fn resize_handle_bounds(&self) -> Rectangle {
        let handle_size = 16.0;
        Rectangle {
            x: self.current_position.x + self.width - handle_size,
            y: self.current_position.y + self.height - handle_size,
            width: handle_size,
            height: handle_size,
        }
    }

    /// Snap position to grid
    pub fn snap_to_grid(position: Point, grid_spacing: f32) -> Point {
        Point::new(
            (position.x / grid_spacing).round() * grid_spacing,
            (position.y / grid_spacing).round() * grid_spacing,
        )
    }

    pub fn update_animation(&mut self, delta_time: f32) {
        if delta_time <= 0.0 || delta_time > 0.1 {
            // Skip if delta_time is invalid (too large or negative)
            return;
        }

        // Animate position
        let distance = ((self.target_position.x - self.current_position.x).powi(2)
                       + (self.target_position.y - self.current_position.y).powi(2)).sqrt();

        if distance > 0.5 {
            // Smooth interpolation with easing
            let speed = 10.0; // Higher = faster snap
            let t = 1.0 - (-speed * delta_time).exp();

            self.current_position.x += (self.target_position.x - self.current_position.x) * t;
            self.current_position.y += (self.target_position.y - self.current_position.y) * t;
        } else {
            self.current_position = self.target_position;
        }

        // Animate size
        let width_diff = (self.target_width - self.width).abs();
        let height_diff = (self.target_height - self.height).abs();

        if width_diff > 0.5 || height_diff > 0.5 {
            // Smooth interpolation with easing for size
            let speed = 10.0; // Same speed as position for consistency
            let t = 1.0 - (-speed * delta_time).exp();

            self.width += (self.target_width - self.width) * t;
            self.height += (self.target_height - self.height) * t;
        } else {
            // Snap to target when close enough
            self.width = self.target_width;
            self.height = self.target_height;
        }
    }
}
