use iced::Color;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Theme {
    #[default]
    Light,
    Dark,
}

impl Theme {
    pub fn background(&self) -> Color {
        match self {
            Theme::Light => Color::WHITE,
            Theme::Dark => Color::from_rgb8(30, 30, 30),
        }
    }

    pub fn sidebar_background(&self) -> Color {
        match self {
            Theme::Light => Color::from_rgb8(245, 245, 245),
            Theme::Dark => Color::from_rgb8(50, 50, 50),
        }
    }

    pub fn sidebar_shadow(&self) -> Color {
        match self {
            Theme::Light => Color::from_rgba(0.0, 0.0, 0.0, 0.15),
            Theme::Dark => Color::from_rgba(0.0, 0.0, 0.0, 0.4),
        }
    }

    pub fn button_background(&self) -> Color {
        match self {
            Theme::Light => Color::from_rgb8(255, 255, 255),
            Theme::Dark => Color::from_rgb8(60, 60, 60),
        }
    }

    pub fn button_background_hovered(&self) -> Color {
        match self {
            Theme::Light => Color::from_rgb8(230, 235, 240),
            Theme::Dark => Color::from_rgb8(80, 85, 90),
        }
    }

    pub fn button_border(&self) -> Color {
        match self {
            Theme::Light => Color::from_rgb8(200, 200, 200),
            Theme::Dark => Color::from_rgb8(80, 80, 80),
        }
    }

    pub fn button_text(&self) -> Color {
        match self {
            Theme::Light => Color::from_rgb8(51, 51, 51),
            Theme::Dark => Color::from_rgb8(240, 240, 240),
        }
    }

    pub fn button_shadow(&self) -> Color {
        match self {
            Theme::Light => Color::from_rgba(0.0, 0.0, 0.0, 0.08),
            Theme::Dark => Color::from_rgba(0.0, 0.0, 0.0, 0.3),
        }
    }

    pub fn dot_color(&self) -> Color {
        match self {
            Theme::Light => Color::from_rgba(0.0, 0.0, 0.0, 0.15),
            Theme::Dark => Color::from_rgba(0.4, 0.4, 0.4, 0.2),
        }
    }

    pub fn separator_color(&self) -> Color {
        match self {
            Theme::Light => Color::from_rgb8(210, 210, 210),
            Theme::Dark => Color::from_rgb8(70, 70, 70),
        }
    }

    pub fn icon_color(&self) -> Color {
        match self {
            Theme::Light => Color::from_rgb8(51, 51, 51),
            Theme::Dark => Color::from_rgb8(220, 220, 220),
        }
    }

    // Card colors
    pub fn card_background(&self) -> Color {
        match self {
            Theme::Light => Color::from_rgb8(255, 255, 255),
            Theme::Dark => Color::from_rgb8(55, 55, 55),
        }
    }

    pub fn card_border(&self) -> Color {
        match self {
            Theme::Light => Color::from_rgb8(200, 200, 200),
            Theme::Dark => Color::from_rgb8(80, 80, 80),
        }
    }

    pub fn card_text(&self) -> Color {
        match self {
            Theme::Light => Color::from_rgb8(51, 51, 51),
            Theme::Dark => Color::from_rgb8(230, 230, 230),
        }
    }

    pub fn toggle(&self) -> Self {
        match self {
            Theme::Light => Theme::Dark,
            Theme::Dark => Theme::Light,
        }
    }
}
