/// Utilities for parsing `[[card reference]]` links in card content.

/// Parse all `[[...]]` references from `content`.
/// Returns `(byte_start, byte_end, ref_text)` for each occurrence.
pub fn parse_refs<'a>(content: &'a str) -> Vec<(usize, usize, &'a str)> {
    let mut result = Vec::new();
    let bytes = content.as_bytes();
    let len = bytes.len();
    let mut i = 0;
    while i + 1 < len {
        if bytes[i] == b'[' && bytes[i + 1] == b'[' {
            let inner_start = i + 2;
            if let Some(close) = content[inner_start..].find("]]") {
                let end = inner_start + close + 2;
                let ref_text = &content[inner_start..inner_start + close];
                if !ref_text.trim().is_empty() {
                    result.push((i, end, ref_text));
                }
                i = end;
                continue;
            }
        }
        i += 1;
    }
    result
}

/// If the cursor (byte offset) is inside an incomplete `[[...` pattern,
/// return `(byte_pos_of_[[, query_string_so_far)`.
pub fn ref_query_at_cursor(content: &str, cursor: usize) -> Option<(usize, String)> {
    let before = &content[..cursor.min(content.len())];
    if let Some(open) = before.rfind("[[") {
        let after = &before[open + 2..];
        // Only active if no closing ]] between [[ and cursor
        if !after.contains("]]") {
            return Some((open, after.to_string()));
        }
    }
    None
}

/// Split a ref string `"board / title"` → `(Some("board"), "title")`
/// or `"title"` → `(None, "title")`.
pub fn parse_ref_parts(ref_text: &str) -> (Option<&str>, &str) {
    if let Some(pos) = ref_text.find(" / ") {
        (Some(ref_text[..pos].trim()), ref_text[pos + 3..].trim())
    } else {
        (None, ref_text.trim())
    }
}

/// Preprocess content for Markdown rendering: replace `[[ref text]]` with
/// `[ref text](card-ref:ref text)` so the existing markdown link renderer handles it.
pub fn preprocess_refs_for_markdown(content: &str) -> String {
    let refs = parse_refs(content);
    if refs.is_empty() {
        return content.to_string();
    }
    let mut result = String::with_capacity(content.len() + refs.len() * 20);
    let mut last = 0;
    for (start, end, ref_text) in &refs {
        result.push_str(&content[last..*start]);
        result.push('[');
        result.push_str(ref_text);
        result.push_str("](card-ref:");
        result.push_str(ref_text);
        result.push(')');
        last = *end;
    }
    result.push_str(&content[last..]);
    result
}

/// Encode a ref text into a "card-ref:" URL.
pub fn encode_card_ref(ref_text: &str) -> String {
    format!("card-ref:{}", ref_text)
}

/// Decode a "card-ref:" URL back to the ref text.
pub fn decode_card_ref(url: &str) -> Option<&str> {
    url.strip_prefix("card-ref:")
}

/// Format an autocomplete completion string.
/// If same board, returns `[[title]]`; if cross-board, returns `[[board / title]]`.
pub fn format_ref(board_name: Option<&str>, title: &str) -> String {
    match board_name {
        Some(board) => format!("[[{} / {}]]", board, title),
        None => format!("[[{}]]", title),
    }
}
