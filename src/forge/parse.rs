use crate::{BlockNode, Diagnostic, Document, ErrorCode, Level, MdEvent, Node, Span};

use super::args::parse_args;

pub fn parse_document(input: &str) -> Result<Document, Vec<Diagnostic>> {
    let lines: Vec<&str> = input.lines().collect();
    let mut diagnostics = Vec::new();
    let (nodes, _, _) = parse_lines(&lines, 0, 0, &mut diagnostics, false);

    if diagnostics.is_empty() {
        Ok(Document { nodes })
    } else {
        Err(diagnostics)
    }
}

fn parse_lines(
    lines: &[&str],
    mut index: usize,
    mut offset: usize,
    diagnostics: &mut Vec<Diagnostic>,
    stop_on_close: bool,
) -> (Vec<Node>, usize, bool) {
    let mut nodes = Vec::new();
    let mut markdown = String::new();

    while index < lines.len() {
        let line = lines[index];
        let line_len_with_nl = line.len() + 1;

        if line.trim() == ":::" {
            if stop_on_close {
                if !markdown.is_empty() {
                    nodes.push(Node::Markdown(vec![MdEvent::Text(markdown)]));
                }
                return (nodes, offset + line_len_with_nl, true);
            }

            markdown.push_str(line);
            markdown.push('\n');
            offset += line_len_with_nl;
            index += 1;
            continue;
        }

        if let Some(rest) = line.strip_prefix(":::") {
            if !markdown.is_empty() {
                nodes.push(Node::Markdown(vec![MdEvent::Text(markdown)]));
                markdown = String::new();
            }

            let open_span = Span {
                start: offset,
                end: offset + line.len(),
            };
            let parts = rest.split_whitespace().collect::<Vec<_>>();
            if parts.is_empty() {
                diagnostics.push(Diagnostic {
                    level: Level::Error,
                    code: ErrorCode::UnknownBlock,
                    message: "block name is missing".to_string(),
                    span: open_span,
                    suggestion: None,
                });
                offset += line_len_with_nl;
                index += 1;
                continue;
            }

            let name = parts[0].to_string();
            let args = parse_args(&parts[1..], open_span.start, diagnostics);

            index += 1;
            offset += line_len_with_nl;
            let (body, new_offset, closed) = parse_lines(lines, index, offset, diagnostics, true);
            if !closed {
                diagnostics.push(Diagnostic {
                    level: Level::Error,
                    code: ErrorCode::BlockNotClosed,
                    message: format!("block '{}' is not closed", name),
                    span: open_span.clone(),
                    suggestion: Some("add a closing ::: line".to_string()),
                });
                nodes.push(Node::Block(BlockNode {
                    name,
                    args,
                    body,
                    span: open_span,
                }));
                return (nodes, offset, false);
            }

            nodes.push(Node::Block(BlockNode {
                name,
                args,
                body,
                span: open_span,
            }));

            let consumed_bytes = new_offset - offset;
            let consumed_lines = lines[index..]
                .iter()
                .scan(0usize, |sum, l| {
                    *sum += l.len() + 1;
                    Some(*sum)
                })
                .position(|s| s == consumed_bytes)
                .map(|p| p + 1)
                .unwrap_or(lines.len() - index);

            index += consumed_lines;
            offset = new_offset;
            continue;
        }

        markdown.push_str(line);
        markdown.push('\n');
        offset += line_len_with_nl;
        index += 1;
    }

    if !markdown.is_empty() {
        nodes.push(Node::Markdown(vec![MdEvent::Text(markdown)]));
    }

    (nodes, offset, false)
}
