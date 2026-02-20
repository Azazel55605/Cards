use iced::widget::button;
use iced::{Background, Border, Color, Shadow, Theme as IcedTheme};

#[derive(Clone)]
pub struct CardButtonStyle {
    pub background: Color,
    pub background_hovered: Color,
    pub text_color: Color,
    pub border_color: Color,
    pub shadow_color: Color,
}

#[derive(Clone)]
pub struct HoverVisibleButtonStyle {
    pub background: Color,
    pub background_hovered: Color,
    pub icon_color: Color,
    pub icon_color_hovered: Color,
    pub border_color: Color,
}

impl button::Catalog for HoverVisibleButtonStyle {
    type Class<'a> = IcedTheme;

    fn default<'a>() -> Self::Class<'a> {
        <IcedTheme as Default>::default()
    }

    fn style(&self, _class: &Self::Class<'_>, status: button::Status) -> button::Style {
        match status {
            button::Status::Active | button::Status::Disabled => button::Style {
                background: Some(Background::Color(self.background)),
                text_color: Color::from_rgba(
                    self.icon_color.r,
                    self.icon_color.g,
                    self.icon_color.b,
                    0.0,  // Invisible when not hovered
                ),
                border: Border {
                    color: self.border_color,
                    width: 0.0,
                    radius: 4.0.into(),
                },
                shadow: Shadow::default(),
            },
            button::Status::Hovered | button::Status::Pressed => button::Style {
                background: Some(Background::Color(self.background_hovered)),
                text_color: self.icon_color_hovered,
                border: Border {
                    color: self.border_color,
                    width: 0.0,
                    radius: 4.0.into(),
                },
                shadow: Shadow::default(),
            },
        }
    }
}

impl From<HoverVisibleButtonStyle> for Box<dyn Fn(&IcedTheme, button::Status) -> button::Style> {
    fn from(style: HoverVisibleButtonStyle) -> Self {
        Box::new(move |_theme, status| {
            match status {
                button::Status::Active | button::Status::Disabled => button::Style {
                    background: Some(Background::Color(style.background)),
                    text_color: Color::from_rgba(
                        style.icon_color.r,
                        style.icon_color.g,
                        style.icon_color.b,
                        0.0,  // Invisible when not hovered
                    ),
                    border: Border {
                        color: style.border_color,
                        width: 0.0,
                        radius: 4.0.into(),
                    },
                    shadow: Shadow::default(),
                },
                button::Status::Hovered | button::Status::Pressed => button::Style {
                    background: Some(Background::Color(style.background_hovered)),
                    text_color: style.icon_color_hovered,
                    border: Border {
                        color: style.border_color,
                        width: 0.0,
                        radius: 4.0.into(),
                    },
                    shadow: Shadow::default(),
                },
            }
        })
    }
}

impl button::Catalog for CardButtonStyle {
    type Class<'a> = IcedTheme;

    fn default<'a>() -> Self::Class<'a> {
        <IcedTheme as Default>::default()
    }

    fn style(&self, _class: &Self::Class<'_>, status: button::Status) -> button::Style {
        match status {
            button::Status::Active | button::Status::Disabled => button::Style {
                background: Some(Background::Color(self.background)),
                text_color: self.text_color,
                border: Border {
                    color: self.border_color,
                    width: 1.0,
                    radius: 4.0.into(),
                },
                shadow: Shadow {
                    color: self.shadow_color,
                    offset: iced::Vector::new(0.0, 2.0),
                    blur_radius: 4.0,
                },
            },
            button::Status::Hovered | button::Status::Pressed => button::Style {
                background: Some(Background::Color(self.background_hovered)),
                text_color: self.text_color,
                border: Border {
                    color: self.border_color,
                    width: 1.0,
                    radius: 4.0.into(),
                },
                shadow: Shadow {
                    color: self.shadow_color,
                    offset: iced::Vector::new(0.0, 2.0),
                    blur_radius: 4.0,
                },
            },
        }
    }
}

impl From<CardButtonStyle> for Box<dyn Fn(&IcedTheme, button::Status) -> button::Style> {
    fn from(style: CardButtonStyle) -> Self {
        Box::new(move |_theme, status| {
            match status {
                button::Status::Active | button::Status::Disabled => button::Style {
                    background: Some(Background::Color(style.background)),
                    text_color: style.text_color,
                    border: Border {
                        color: style.border_color,
                        width: 1.0,
                        radius: 4.0.into(),
                    },
                    shadow: Shadow {
                        color: style.shadow_color,
                        offset: iced::Vector::new(0.0, 2.0),
                        blur_radius: 4.0,
                    },
                },
                button::Status::Hovered | button::Status::Pressed => button::Style {
                    background: Some(Background::Color(style.background_hovered)),
                    text_color: style.text_color,
                    border: Border {
                        color: style.border_color,
                        width: 1.0,
                        radius: 4.0.into(),
                    },
                    shadow: Shadow {
                        color: style.shadow_color,
                        offset: iced::Vector::new(0.0, 2.0),
                        blur_radius: 4.0,
                    },
                },
            }
        })
    }
}
