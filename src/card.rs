use iced::{Color, Point, Rectangle};
use iced::widget::text_editor;

#[derive(Debug)]
pub struct Card {
    pub id: usize,
    pub current_position: Point,
    pub target_position: Point,
    pub width: f32,
    pub height: f32,
    pub icon: CardIcon,
    pub color: Color,
    pub is_dragging: bool,
    pub content: text_editor::Content,
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
            icon: self.icon,
            color: self.color,
            is_dragging: self.is_dragging,
            content: text_editor::Content::with_text(&self.content.text()),
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
}

impl CardIcon {
    pub fn svg_path(&self) -> &'static str {
        match self {
            CardIcon::Default => "src/icons/default.svg",
            CardIcon::Star => "src/icons/star.svg",
            CardIcon::Heart => "src/icons/heart.svg",
            CardIcon::Circle => "src/icons/circle.svg",
            CardIcon::Square => "src/icons/square.svg",
        }
    }

    pub fn all() -> &'static [CardIcon] {
        &[CardIcon::Default, CardIcon::Star, CardIcon::Heart, CardIcon::Circle, CardIcon::Square]
    }
}

impl Card {
    pub fn new(id: usize, position: Point) -> Self {
        Self {
            id,
            current_position: position,
            target_position: position,
            width: 200.0,
            height: 150.0,
            icon: CardIcon::Default,
            color: Color::from_rgb8(100, 150, 255), // Default blue
            is_dragging: false,
            content: text_editor::Content::new(),
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
    }
}
