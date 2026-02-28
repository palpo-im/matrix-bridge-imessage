pub fn preview_text(text: &str, max_len: usize) -> String {
    if text.len() <= max_len {
        text.to_string()
    } else {
        format!("{}...", &text[..max_len.saturating_sub(3)])
    }
}

pub fn action_keyword(is_add: bool) -> &'static str {
    if is_add {
        "added"
    } else {
        "removed"
    }
}
