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

    // Accent colors — driven by the accent Color passed in from config
    pub fn accent_from(&self, base: Color) -> Color {
        base
    }

    pub fn accent_dim_from(&self, base: Color) -> Color {
        // Darken by ~25%
        Color {
            r: (base.r * 0.75).min(1.0),
            g: (base.g * 0.75).min(1.0),
            b: (base.b * 0.75).min(1.0),
            a: base.a,
        }
    }

    pub fn accent_glow_from(&self, base: Color) -> Color {
        Color { a: 0.22, ..base }
    }

    pub fn accent_bg_from(&self, base: Color) -> Color {
        Color { a: 0.08, ..base }
    }

    // Legacy fixed-purple methods kept for backward compat; prefer *_from variants
    pub fn accent(&self) -> Color {
        Color::from_rgb8(124, 92, 252)
    }

    pub fn accent_dim(&self) -> Color {
        Color::from_rgb8(91, 63, 212)
    }

    pub fn accent_glow(&self) -> Color {
        Color::from_rgba8(124, 92, 252, 0.22)
    }

    pub fn accent_bg(&self) -> Color {
        Color::from_rgba8(124, 92, 252, 0.08)
    }

    pub fn toggle(&self) -> Self {
        match self {
            Theme::Light => Theme::Dark,
            Theme::Dark => Theme::Light,
        }
    }
}
