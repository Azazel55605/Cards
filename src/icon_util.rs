/// Utility module for converting icondata to SVG for iced
use icondata_core::Icon;

/// Convert an icondata Icon to SVG bytes for use with iced::widget::svg
pub fn icon_to_svg(icon: Icon) -> Vec<u8> {
    let view_box = icon.view_box.unwrap_or("0 0 16 16");
    let svg = format!(
        r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="{}" fill="currentColor">{}</svg>"#,
        view_box, icon.data
    );
    svg.into_bytes()
}


