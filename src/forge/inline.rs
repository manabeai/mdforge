use std::collections::HashMap;

use crate::{ArgValue, InlineExt, Span};

pub fn parse_inline_exts(text: &str) -> Vec<InlineExt> {
    let mut out = Vec::new();
    let bytes = text.as_bytes();
    let mut i = 0;

    while i < bytes.len() {
        if bytes[i] != b'{' {
            i += 1;
            continue;
        }

        let start = i;
        let mut j = i + 1;
        while j < bytes.len() && bytes[j] != b'}' {
            j += 1;
        }
        if j >= bytes.len() {
            break;
        }

        let content = &text[i + 1..j];
        let parts = content.split_whitespace().collect::<Vec<_>>();
        if !parts.is_empty() {
            let mut args = HashMap::new();
            for token in &parts[1..] {
                if let Some((key, raw_value)) = token.split_once('=') {
                    let value = if let Ok(int_value) = raw_value.parse::<i64>() {
                        ArgValue::Int(int_value)
                    } else {
                        ArgValue::String(raw_value.to_string())
                    };
                    args.insert(key.to_string(), value);
                }
            }

            out.push(InlineExt {
                name: parts[0].to_string(),
                args,
                span: Span { start, end: j + 1 },
            });
        }

        i = j + 1;
    }

    out
}
