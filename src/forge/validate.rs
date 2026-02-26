use crate::{BlockNode, Diagnostic, Document, ErrorCode, InlineExt, Level, MdEvent, Node};

use super::{args::validate_args, inline::parse_inline_exts, BlockSpec, InlineSpec};

pub fn validate_document(
    doc: &Document,
    blocks: &[BlockSpec],
    inlines: &[InlineSpec],
) -> Result<(), Vec<Diagnostic>> {
    let mut diagnostics = Vec::new();
    validate_nodes(&doc.nodes, blocks, inlines, &mut diagnostics);
    if diagnostics.is_empty() {
        Ok(())
    } else {
        Err(diagnostics)
    }
}

fn validate_nodes(
    nodes: &[Node],
    blocks: &[BlockSpec],
    inlines: &[InlineSpec],
    diagnostics: &mut Vec<Diagnostic>,
) {
    for node in nodes {
        match node {
            Node::Markdown(events) => {
                for event in events {
                    let MdEvent::Text(text) = event;
                    for inline in parse_inline_exts(text) {
                        validate_inline(&inline, inlines, diagnostics);
                    }
                }
            }
            Node::Block(block) => {
                validate_block(block, blocks, diagnostics);
                validate_nodes(&block.body, blocks, inlines, diagnostics);
            }
        }
    }
}

fn validate_block(block: &BlockNode, blocks: &[BlockSpec], diagnostics: &mut Vec<Diagnostic>) {
    let Some(spec) = blocks.iter().find(|s| s.name == block.name) else {
        diagnostics.push(Diagnostic {
            level: Level::Error,
            code: ErrorCode::UnknownBlock,
            message: format!("unknown block '{}'", block.name),
            span: block.span.clone(),
            suggestion: None,
        });
        return;
    };

    validate_args(&block.args, &spec.args, block.span.clone(), diagnostics);
}

fn validate_inline(inline: &InlineExt, inlines: &[InlineSpec], diagnostics: &mut Vec<Diagnostic>) {
    let Some(spec) = inlines.iter().find(|s| s.name == inline.name) else {
        diagnostics.push(Diagnostic {
            level: Level::Error,
            code: ErrorCode::UnknownInline,
            message: format!("unknown inline '{}'", inline.name),
            span: inline.span.clone(),
            suggestion: None,
        });
        return;
    };

    validate_args(&inline.args, &spec.args, inline.span.clone(), diagnostics);
}
