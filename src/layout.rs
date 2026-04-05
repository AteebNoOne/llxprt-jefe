//! Layout constants and coordinate calculation functions.
//!
//! Extracted from main.rs to isolate geometry logic and enable focused unit testing.

/// Left column width (repository list).
pub const LEFT_COL_WIDTH: u16 = 22;
/// Right column width (preview pane).
pub const RIGHT_COL_WIDTH: u16 = 36;
/// Height consumed by status bar + keybind bar.
pub const OUTER_BARS_HEIGHT: u16 = 2;
/// Terminal widget chrome rows: top border + header row + bottom border.
pub const TERMINAL_WIDGET_CHROME_ROWS: u16 = 3;
/// Terminal widget chrome columns: left + right border.
pub const TERMINAL_WIDGET_CHROME_COLS: u16 = 2;

/// Check if fullscreen mode is enabled.
#[must_use]
pub fn is_fullscreen_enabled() -> bool {
    std::env::var("JEFE_WINDOWED").ok().as_deref() != Some("1")
}

/// Calculate effective render dimensions.
#[must_use]
pub fn effective_render_size(cols: u16, rows: u16) -> (u16, u16) {
    if is_fullscreen_enabled() {
        (cols, rows)
    } else {
        (cols.saturating_sub(2).max(1), rows.saturating_sub(2).max(1))
    }
}

/// Compute PTY viewport size and its origin within the fullscreen render grid.
///
/// Layout mirrors dashboard proportions:
/// - top status bar (1 row)
/// - bottom keybind bar (1 row)
/// - middle column split: agent list 25%, terminal 75%
/// - terminal widget chrome: border + header + border
#[must_use]
pub fn compute_pty_layout(term_cols: u16, term_rows: u16) -> (u16, u16, u16, u16) {
    let (render_cols, render_rows) = effective_render_size(term_cols, term_rows);

    let content_rows = render_rows.saturating_sub(OUTER_BARS_HEIGHT);
    let middle_cols = render_cols.saturating_sub(LEFT_COL_WIDTH + RIGHT_COL_WIDTH);

    // Round to nearest (half-up) to match taffy's cumulative rounding of
    // percentage-based flex children.  Simple truncation (`*25/100`) under-
    // counts by 1 row whenever the product has a fractional part ≥ 0.5,
    // which shifts the mouse-coordinate origin for the terminal pane.
    let agent_rows = content_rows
        .saturating_mul(25)
        .saturating_add(50)
        .saturating_div(100);
    let terminal_slot_rows = content_rows.saturating_sub(agent_rows);

    let pty_rows = terminal_slot_rows
        .saturating_sub(TERMINAL_WIDGET_CHROME_ROWS)
        .max(2);
    let pty_cols = middle_cols
        .saturating_sub(TERMINAL_WIDGET_CHROME_COLS)
        .max(2);

    let pane_col0 = LEFT_COL_WIDTH.saturating_add(1);
    let pane_row0 = 1u16.saturating_add(agent_rows).saturating_add(2);

    (pty_rows, pty_cols, pane_col0, pane_row0)
}

#[cfg(test)]
mod tests {
    use super::*;

    // Note: tests that depend on `is_fullscreen_enabled()` are sensitive to the
    // JEFE_WINDOWED env var at test-run time. We test the pure computation paths
    // directly and verify the windowed subtraction logic with explicit math.

    #[test]
    fn effective_render_size_passthrough_in_fullscreen() {
        // In fullscreen mode, dimensions pass through unchanged.
        if is_fullscreen_enabled() {
            assert_eq!(effective_render_size(120, 40), (120, 40));
            assert_eq!(effective_render_size(80, 24), (80, 24));
        }
    }

    #[test]
    fn effective_render_size_windowed_subtraction() {
        // Windowed mode subtracts 2 from each dimension, clamping to 1.
        if !is_fullscreen_enabled() {
            assert_eq!(effective_render_size(120, 40), (118, 38));
            assert_eq!(effective_render_size(2, 2), (1, 1));
            assert_eq!(effective_render_size(1, 1), (1, 1));
        }
    }

    #[test]
    fn compute_pty_layout_pane_origin() {
        let (_, _, pane_col0, _) = compute_pty_layout(120, 40);
        assert_eq!(pane_col0, LEFT_COL_WIDTH + 1);
    }

    #[test]
    fn compute_pty_layout_dimensions_always_at_least_two() {
        for (cols, rows) in [(120, 40), (10, 10), (0, 0), (60, 20)] {
            let (pty_rows, pty_cols, _, _) = compute_pty_layout(cols, rows);
            assert!(pty_rows >= 2, "pty_rows < 2 for ({cols}, {rows})");
            assert!(pty_cols >= 2, "pty_cols < 2 for ({cols}, {rows})");
        }
    }

    #[test]
    fn agent_rows_rounding_half_up() {
        // For 38 content rows (40 - 2 bars in fullscreen), 25% = 9.5 → rounds to 10.
        // Formula: (38 * 25 + 50) / 100 = 10
        if is_fullscreen_enabled() {
            let (_, _, _, pane_row0) = compute_pty_layout(120, 40);
            // pane_row0 = 1 (status bar) + agent_rows + 2 (chrome top border + header)
            assert_eq!(pane_row0, 1 + 10 + 2);
        }
    }

    #[test]
    fn compute_pty_layout_pane_row0_positive() {
        let (_, _, _, pane_row0) = compute_pty_layout(120, 40);
        assert!(pane_row0 > 0);
    }
}
