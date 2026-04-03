//! Filter controls component.
//! @plan PLAN-20260329-ISSUES-MODE.P12
//! @plan PLAN-20260329-ISSUES-MODE.P14
//! @requirement REQ-ISS-008

use iocraft::prelude::*;

use crate::domain::{IssueFilter, IssueFilterState};
use crate::theme::{ResolvedColors, ThemeColors};

/// Props for the filter controls pane.
#[derive(Default, Props)]
pub struct FilterControlsProps {
    /// Current draft filter values.
    pub draft_filter: IssueFilter,
    /// Whether the controls are visible.
    pub visible: bool,
    /// Theme colors.
    pub colors: ThemeColors,
}

/// Filter controls — compact horizontal band showing current filter values and action hints.
/// @plan PLAN-20260329-ISSUES-MODE.P14
/// @requirement REQ-ISS-008
#[component]
pub fn FilterControls(props: &FilterControlsProps) -> impl Into<AnyElement<'static>> {
    if !props.visible {
        return element! {
            Box(width: 0u32, height: 0u32) {}
        };
    }

    let rc = ResolvedColors::from_theme(Some(&props.colors));

    let state_label = match props.draft_filter.state {
        Some(IssueFilterState::Open) | None => "open",
        Some(IssueFilterState::Closed) => "closed",
        Some(IssueFilterState::All) => "all",
    };

    let author_label = if props.draft_filter.author.is_empty() {
        "any".to_string()
    } else {
        props.draft_filter.author.clone()
    };

    let assignee_label = if props.draft_filter.assignee.is_empty() {
        "any".to_string()
    } else {
        props.draft_filter.assignee.clone()
    };

    let labels_label = if props.draft_filter.labels.is_empty() {
        "any".to_string()
    } else {
        props.draft_filter.labels.join(", ")
    };

    let search_label = if props.draft_filter.query_text.is_empty() {
        String::new()
    } else {
        format!("  search:{}", props.draft_filter.query_text)
    };

    element! {
        Box(
            flex_direction: FlexDirection::Column,
            width: 100pct,
            border_style: BorderStyle::Round,
            border_color: rc.bright,
            background_color: rc.bg,
            padding_left: 1u32,
            padding_right: 1u32,
        ) {
            // Filter values row
            Box(height: 1u32) {
                Text(content: "Filter: ", color: rc.dim)
                Text(content: "state:", color: rc.dim)
                Text(content: state_label, color: rc.fg)
                Text(content: "  author:", color: rc.dim)
                Text(content: author_label, color: rc.fg)
                Text(content: "  assignee:", color: rc.dim)
                Text(content: assignee_label, color: rc.fg)
                Text(content: "  labels:", color: rc.dim)
                Text(content: labels_label, color: rc.fg)
                Text(content: search_label, color: rc.fg)
            }
            // Actions hint row
            Box(height: 1u32) {
                Text(content: "Enter apply  ", color: rc.dim)
                Text(content: "c clear  ", color: rc.dim)
                Text(content: "Esc cancel", color: rc.dim)
            }
        }
    }
}
