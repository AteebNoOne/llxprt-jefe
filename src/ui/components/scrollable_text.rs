//! Scrollable text viewport with scrollbar.
//!
//! Renders multi-line text within a viewport that fills available space.
//! Uses a window of visible lines based on scroll offset, with a scrollbar track.

use iocraft::prelude::*;

/// Props for the scrollable text viewport.
#[derive(Default, Props)]
pub struct ScrollableTextProps {
    /// The full text content to display (may contain newlines).
    pub content: String,
    /// Current scroll offset in lines (0 = top).
    pub scroll_offset: usize,
    /// Text color.
    pub color: Option<Color>,
    /// Scrollbar track color (dimmed).
    pub track_color: Option<Color>,
    /// Scrollbar thumb color (bright).
    pub thumb_color: Option<Color>,
}

/// Compute scrollbar thumb position and size.
fn scrollbar_geometry(total: usize, visible: usize, offset: usize) -> (usize, usize) {
    if total <= visible || visible == 0 {
        return (0, visible);
    }
    let thumb_size = (visible * visible / total).max(1).min(visible);
    let max_offset = total.saturating_sub(visible);
    let scrollable_rows = visible.saturating_sub(thumb_size);
    let thumb_pos = if max_offset > 0 {
        (offset * scrollable_rows / max_offset).min(scrollable_rows)
    } else {
        0
    };
    (thumb_pos, thumb_size)
}

/// Scrollable text viewport — renders visible lines and a scrollbar.
///
/// The parent Box must constrain height (e.g. `flex_grow: 1.0` in a column).
/// Only lines within the scroll window are rendered; iocraft's layout handles the rest.
#[component]
pub fn ScrollableText(props: &ScrollableTextProps) -> impl Into<AnyElement<'static>> {
    let fg = props.color.unwrap_or(Color::Reset);
    let track_color = props.track_color.unwrap_or(Color::DarkGrey);
    let thumb_color = props.thumb_color.unwrap_or(Color::White);

    let all_lines: Vec<&str> = props.content.lines().collect();
    let total = all_lines.len();
    let max_visible = 200;
    let offset = props.scroll_offset.min(total.saturating_sub(1));

    let visible_lines: Vec<&str> = all_lines
        .iter()
        .skip(offset)
        .take(max_visible)
        .copied()
        .collect();

    let sb_height = visible_lines.len().min(max_visible);
    let (thumb_pos, thumb_size) = scrollbar_geometry(total, sb_height, offset);
    let show_scrollbar = total > 1;

    element! {
        Box(flex_direction: FlexDirection::Row, width: 100pct, height: 100pct) {
            Box(flex_direction: FlexDirection::Column, flex_grow: 1.0) {
                #(visible_lines.iter().map(|line| {
                    element! {
                        Box(height: 1u32) {
                            Text(content: line.to_string(), color: fg, wrap: TextWrap::NoWrap)
                        }
                    }
                }).collect::<Vec<_>>())
            }
            #(if show_scrollbar {
                vec![element! {
                    Box(flex_direction: FlexDirection::Column, width: 1u32) {
                        #((0..sb_height).map(|row| {
                            let is_thumb = row >= thumb_pos && row < thumb_pos + thumb_size;
                            let ch = if is_thumb { "┃" } else { "│" };
                            let color = if is_thumb { thumb_color } else { track_color };
                            element! {
                                Box(height: 1u32) {
                                    Text(content: ch.to_string(), color: color)
                                }
                            }
                        }).collect::<Vec<_>>())
                    }
                }]
            } else {
                vec![]
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scrollbar_geometry_no_scroll() {
        let (pos, size) = scrollbar_geometry(5, 10, 0);
        assert_eq!(pos, 0);
        assert_eq!(size, 10);
    }

    #[test]
    fn test_scrollbar_geometry_at_top() {
        let (pos, size) = scrollbar_geometry(100, 20, 0);
        assert_eq!(pos, 0);
        assert!(size >= 1);
        assert!(size <= 20);
    }

    #[test]
    fn test_scrollbar_geometry_at_bottom() {
        let (pos, size) = scrollbar_geometry(100, 20, 80);
        assert!(pos + size <= 20);
    }
}
