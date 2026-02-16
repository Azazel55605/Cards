use iced::widget::svg;
use iced::{Color, Theme};

pub struct SvgStyle {
    pub color: Color,
}

impl svg::Catalog for SvgStyle {
    type Class<'a> = Theme;

    fn default<'a>() -> Self::Class<'a> {
        <Theme as Default>::default()
    }

    fn style(&self, _class: &Self::Class<'_>, _status: svg::Status) -> svg::Style {
        svg::Style {
            color: Some(self.color),
        }
    }
}

impl From<SvgStyle> for Box<dyn Fn(&Theme, svg::Status) -> svg::Style> {
    fn from(style: SvgStyle) -> Self {
        Box::new(move |_theme, _status| svg::Style {
            color: Some(style.color),
        })
    }
}
