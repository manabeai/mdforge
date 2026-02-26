use crate::{BlockNode, Diagnostic, Document, ErrorCode, InlineExt, Level, MdEvent, Node};

use super::{args::parse_inline_exts, args::validate_args, Forge};

pub(super) fn validate_document(forge: &Forge, doc: &Document) -> Result<(), Vec<Diagnostic>> {
    let mut diagnostics = Vec::new();
    validate_nodes(forge, &doc.nodes, &mut diagnostics);
    if diagnostics.is_empty() {
        Ok(())
    } else {
        Err(diagnostics)
    }
}

fn validate_nodes(forge: &Forge, nodes: &[Node], diagnostics: &mut Vec<Diagnostic>) {
    for node in nodes {
        match node {
            Node::Markdown(events) => {
                for event in events {
                    let MdEvent::Text(text) = event;
                    for inline in parse_inline_exts(text) {
                        validate_inline(forge, &inline, diagnostics);
                    }
                }
            }
            Node::Block(block) => {
                validate_block(forge, block, diagnostics);
                validate_nodes(forge, &block.body, diagnostics);
            }
        }
    }
}

fn validate_block(forge: &Forge, block: &BlockNode, diagnostics: &mut Vec<Diagnostic>) {
    let Some(spec) = forge.blocks.iter().find(|s| s.name == block.name) else {
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

fn validate_inline(forge: &Forge, inline: &InlineExt, diagnostics: &mut Vec<Diagnostic>) {
    let Some(spec) = forge.inlines.iter().find(|s| s.name == inline.name) else {
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
