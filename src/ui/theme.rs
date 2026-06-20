use ratatui::style::{Color, Modifier, Style};

pub const TEXT: Color = Color::White;
pub const DIM: Color = Color::DarkGray;
pub const BRIGHT: Color = Color::White;

pub fn title() -> Style {
    Style::default().fg(TEXT).add_modifier(Modifier::BOLD)
}

pub fn border(focused: bool) -> Style {
    if focused {
        Style::default().fg(TEXT)
    } else {
        Style::default().fg(DIM)
    }
}

pub fn text() -> Style {
    Style::default().fg(TEXT)
}

pub fn dim() -> Style {
    Style::default().fg(DIM)
}

pub fn selected() -> Style {
    Style::default()
        .fg(BRIGHT)
        .add_modifier(Modifier::BOLD | Modifier::UNDERLINED)
}

pub fn stage_color(depth_ratio: f32) -> Color {
    let t = depth_ratio.clamp(0.0, 1.0);
    if t < 0.33 {
        Color::White
    } else if t < 0.66 {
        Color::Gray
    } else {
        Color::DarkGray
    }
}

pub fn stage_active() -> Style {
    Style::default()
        .fg(BRIGHT)
        .add_modifier(Modifier::BOLD | Modifier::UNDERLINED)
}
