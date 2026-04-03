//! Unified issue detail + comments view.
//! @plan PLAN-20260329-ISSUES-MODE.P12
//! @plan PLAN-20260329-ISSUES-MODE.P14
//! @requirement REQ-ISS-009

use iocraft::prelude::*;

use crate::domain::{IssueDetail, IssueState};
use crate::state::{DetailSubfocus, InlineState};
use crate::theme::{ResolvedColors, ThemeColors};

use super::scrollable_text::ScrollableText;

/// Insert a caret character at the given byte-offset cursor position in the text.
/// Uses the bright color caret (▏) consistent with form field editing.
/// The `cursor` parameter is a byte offset (matching the state representation).
fn render_text_with_caret(value: &str, cursor: usize) -> String {
    let byte_idx = cursor.min(value.len());
    // Ensure we're at a char boundary
    let byte_idx = if byte_idx == 0 || byte_idx >= value.len() {
        byte_idx
    } else {
        // Snap to nearest char boundary at or before byte_idx
        value[..byte_idx]
            .char_indices()
            .last()
            .map_or(0, |(i, c)| i + c.len_utf8())
    };
    format!("{}▏{}", &value[..byte_idx], &value[byte_idx..])
}

/// Build the scrollable content string for the body + comments + new-comment area.
/// This is rendered through the ScrollableText component so it never expands layout.
#[allow(clippy::too_many_lines)]
fn build_detail_content(
    detail: &IssueDetail,
    subfocus: DetailSubfocus,
    inline_state: &InlineState,
    comments_loading: bool,
) -> String {
    let nl = String::from(char::from(0x0Au8));
    let mut lines: Vec<String> = Vec::new();

    // ── Body section ────────────────────────────────────────────────
    let body_focused = subfocus == DetailSubfocus::Body;
    let body_label = if body_focused { "> Body" } else { "  Body" };
    lines.push(body_label.to_string());

    let (body_text, body_editing) = match inline_state {
        InlineState::Editor {
            target: crate::state::EditorTarget::IssueBody,
            text,
            cursor,
        } => (render_text_with_caret(text, *cursor), true),
        _ => (detail.body.clone(), false),
    };

    if body_editing {
        lines.push("[editing]".to_string());
    }
    for line in body_text.lines() {
        let prefix = if body_editing { "  │ " } else { "    " };
        lines.push(format!("{prefix}{line}"));
    }
    if body_text.is_empty() {
        let prefix = if body_editing { "  │ " } else { "    " };
        lines.push(format!("{prefix}(empty)"));
    }
    if body_editing {
        lines.push("  Ctrl+Enter save | Esc cancel".to_string());
    }

    lines.push("─────────────────────────────────────────".to_string());

    // ── Comments section ────────────────────────────────────────────
    lines.push("Comments".to_string());
    if comments_loading {
        lines.push("  Loading comments...".to_string());
    } else if detail.comments.is_empty() {
        lines.push("  No comments yet.".to_string());
    } else {
        for (idx, comment) in detail.comments.iter().enumerate() {
            let comment_focused = subfocus == DetailSubfocus::Comment(idx);
            let prefix = if comment_focused { "> " } else { "  " };
            lines.push(format!(
                "{}@{}  {}",
                prefix, comment.author_login, comment.created_at
            ));

            // Check for editor targeting this comment
            let (cmt_text, cmt_editing) = match inline_state {
                InlineState::Editor {
                    target: crate::state::EditorTarget::Comment { comment_index },
                    text,
                    cursor,
                } if *comment_index == idx => (render_text_with_caret(text, *cursor), true),
                _ => (comment.body.clone(), false),
            };

            if cmt_editing {
                lines.push("  [editing]".to_string());
            }
            for line in cmt_text.lines() {
                let prefix = if cmt_editing { "    │ " } else { "      " };
                lines.push(format!("{prefix}{line}"));
            }
            if cmt_text.is_empty() {
                let prefix = if cmt_editing { "    │ " } else { "      " };
                lines.push(format!("{prefix}(empty)"));
            }
            if cmt_editing {
                lines.push("    Ctrl+Enter save | Esc cancel".to_string());
            }

            // Check for reply composer
            if let InlineState::Composer {
                target: crate::state::ComposerTarget::Reply { comment_index, .. },
                text,
                cursor,
            } = inline_state
                && *comment_index == idx
            {
                lines.push("    [Reply]".to_string());
                let reply_text = render_text_with_caret(text, *cursor);
                for line in reply_text.lines() {
                    lines.push(format!("    │ {line}"));
                }
                if reply_text.is_empty() {
                    lines.push("    │ ".to_string());
                }
                lines.push("    Ctrl+Enter save | Esc cancel".to_string());
            }

            lines.push(String::new()); // blank line between comments
        }
    }

    lines.push("─────────────────────────────────────────".to_string());

    // ── New Comment section ─────────────────────────────────────────
    let nc_focused = subfocus == DetailSubfocus::NewComment;
    let nc_label = if nc_focused {
        "> New Comment"
    } else {
        "  New Comment"
    };
    lines.push(nc_label.to_string());

    if let InlineState::Composer {
        target: crate::state::ComposerTarget::NewComment,
        text,
        cursor,
    } = inline_state
    {
        let composer_text = render_text_with_caret(text, *cursor);
        for line in composer_text.lines() {
            lines.push(format!("  │ {line}"));
        }
        if composer_text.is_empty() {
            lines.push("  │ ".to_string());
        }
        lines.push("  Ctrl+Enter submit | Esc cancel".to_string());
    } else {
        lines.push("  Press c to add a comment".to_string());
    }

    lines.join(&nl)
}

/// Props for the issue detail view.
#[derive(Default, Props)]
pub struct IssueDetailViewProps {
    /// Full issue detail (metadata, body, comments).
    pub issue_detail: Option<IssueDetail>,
    /// Which sub-element is focused within the detail view.
    pub detail_subfocus: DetailSubfocus,
    /// Active inline editor/composer state.
    pub inline_state: InlineState,
    /// Whether comments are loading.
    pub comments_loading: bool,
    /// Whether this pane is focused.
    pub focused: bool,
    /// Scroll offset for the content viewport.
    pub scroll_offset: usize,
    /// Theme colors.
    pub colors: ThemeColors,
}

/// Issue detail view — fixed metadata header + scrollable body/comments viewport.
/// @plan PLAN-20260329-ISSUES-MODE.P14
/// @requirement REQ-ISS-009
#[component]
pub fn IssueDetailView(props: &IssueDetailViewProps) -> impl Into<AnyElement<'static>> {
    let rc = ResolvedColors::from_theme(Some(&props.colors));
    let border_style = if props.focused {
        BorderStyle::Double
    } else {
        BorderStyle::Round
    };

    let Some(detail) = props.issue_detail.as_ref() else {
        return element! {
            Box(
                flex_direction: FlexDirection::Column,
                width: 100pct,
                height: 100pct,
                border_style: border_style,
                border_color: rc.border,
                background_color: rc.bg,
            ) {
                Box(padding_left: 1u32, height: 1u32) {
                    Text(content: "No issue selected", color: rc.dim)
                }
            }
        };
    };

    let state_tag = match detail.state {
        IssueState::Open => "OPEN",
        IssueState::Closed => "CLOSED",
    };
    let state_color = match detail.state {
        IssueState::Open => rc.bright,
        IssueState::Closed => rc.dim,
    };

    let labels_str = if detail.labels.is_empty() {
        "-".to_string()
    } else {
        detail.labels.join(", ")
    };
    let assignees_str = if detail.assignees.is_empty() {
        "-".to_string()
    } else {
        detail.assignees.join(", ")
    };
    let milestone_str = detail.milestone.as_deref().unwrap_or("-").to_string();

    // Build the scrollable content for body + comments
    let content = build_detail_content(
        detail,
        props.detail_subfocus,
        &props.inline_state,
        props.comments_loading,
    );

    element! {
        Box(
            flex_direction: FlexDirection::Column,
            width: 100pct,
            height: 100pct,
            border_style: border_style,
            border_color: rc.border,
            background_color: rc.bg,
        ) {
            // ── Metadata header (fixed height) ──────────────────────────────
            Box(flex_direction: FlexDirection::Column, padding_left: 1u32, padding_right: 1u32) {
                Box(height: 1u32) {
                    Text(
                        content: format!("#{} {}", detail.number, detail.title),
                        color: rc.fg,
                    )
                }
                Box(height: 1u32) {
                    Text(content: state_tag, color: state_color)
                    Text(
                        content: format!(
                            "  by @{}  opened: {}  updated: {}",
                            detail.author_login, detail.created_at, detail.updated_at
                        ),
                        color: rc.dim,
                    )
                }
                Box(height: 1u32) {
                    Text(content: "labels: ", color: rc.dim)
                    Text(content: labels_str, color: rc.fg)
                    Text(content: "  assignees: ", color: rc.dim)
                    Text(content: assignees_str, color: rc.fg)
                    Text(content: "  milestone: ", color: rc.dim)
                    Text(content: milestone_str, color: rc.fg)
                }
                Box(height: 1u32) {
                    Text(content: detail.external_url.clone(), color: rc.dim)
                }
                Box(height: 1u32) {
                    Text(
                        content: "─────────────────────────────────────────",
                        color: rc.dim,
                    )
                }
            }

            // ── Scrollable body + comments viewport (fills remaining) ───────
            Box(flex_grow: 1.0, width: 100pct, padding_left: 1u32) {
                ScrollableText(
                    content: content,
                    scroll_offset: props.scroll_offset,
                    color: rc.fg,
                    track_color: rc.dim,
                    thumb_color: rc.bright,
                )
            }
        }
    }
}
