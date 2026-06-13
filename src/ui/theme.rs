//! Centralised palette and styles: a calm, dark, archival aesthetic with gold
//! accents for modern forms fading to muted tones for ancient ones.

use ratatui::style::{Color, Modifier, Style};

/// Gold accent for the modern anchor.
pub const GOLD: Color = Color::Rgb(212, 175, 55);
/// Bright gold for the active/selected element.
pub const GOLD_BRIGHT: Color = Color::Rgb(240, 205, 90);
/// Muted slate for borders.
pub const BORDER: Color = Color::Rgb(78, 78, 96);
/// Brighter border for the focused panel.
pub const BORDER_FOCUS: Color = Color::Rgb(150, 130, 70);
/// Default body text.
pub const TEXT: Color = Color::Rgb(205, 200, 185);
/// Dim/secondary text.
pub const DIM: Color = Color::Rgb(120, 116, 104);
/// Background tint for the active row.
pub const ACTIVE_BG: Color = Color::Rgb(48, 44, 30);

/// Style for panel titles.
pub fn title() -> Style {
    Style::default().fg(GOLD).add_modifier(Modifier::BOLD)
}

/// Style for a panel border, focused or not.
pub fn border(focused: bool) -> Style {
    Style::default().fg(if focused { BORDER_FOCUS } else { BORDER })
}

/// Default text style.
pub fn text() -> Style {
    Style::default().fg(TEXT)
}

/// Dim/secondary text style.
pub fn dim() -> Style {
    Style::default().fg(DIM)
}

/// Style for the selected list row.
pub fn selected() -> Style {
    Style::default()
        .fg(GOLD_BRIGHT)
        .bg(ACTIVE_BG)
        .add_modifier(Modifier::BOLD)
}

/// Colour for a timeline stage given its depth ratio (0.0 = modern .. 1.0 = root).
///
/// Linearly interpolates from warm gold to a muted grey-brown so the descent
/// into history visibly fades.
pub fn stage_color(depth_ratio: f32) -> Color {
    let t = depth_ratio.clamp(0.0, 1.0);
    let lerp = |a: u8, b: u8| (a as f32 + (b as f32 - a as f32) * t).round() as u8;
    Color::Rgb(lerp(212, 122), lerp(175, 112), lerp(55, 100))
}

/// Style for the active timeline stage.
pub fn stage_active() -> Style {
    Style::default()
        .fg(GOLD_BRIGHT)
        .add_modifier(Modifier::BOLD | Modifier::UNDERLINED)
}
