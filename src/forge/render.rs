use crate::{Diagnostic, Document, ErrorCode, Level, MdEvent, Node, VNode};

use super::{inline::parse_inline_exts, BlockSpec, DomRenderer, EvalContext, InlineSpec};

pub fn render_document(
    doc: &Document,
    blocks: &[BlockSpec],
    inlines: &[InlineSpec],
    ctx: &EvalContext,
    renderer: &dyn DomRenderer,
) -> Result<Vec<VNode>, Vec<Diagnostic>> {
    let mut diagnostics = Vec::new();
    let nodes = render_nodes(&doc.nodes, blocks, inlines, ctx, renderer, &mut diagnostics);

    if diagnostics.is_empty() {
        Ok(nodes)
    } else {
        Err(diagnostics)
    }
}

fn render_nodes(
    nodes: &[Node],
    blocks: &[BlockSpec],
    inlines: &[InlineSpec],
    ctx: &EvalContext,
    renderer: &dyn DomRenderer,
    diagnostics: &mut Vec<Diagnostic>,
) -> Vec<VNode> {
    let mut out = Vec::new();

    for node in nodes {
        match node {
            Node::Markdown(events) => {
                for event in events {
                    let MdEvent::Text(text) = event;
                    out.extend(render_markdown_text(
                        text,
                        inlines,
                        ctx,
                        renderer,
                        diagnostics,
                    ));
                }
            }
            Node::Block(block) => {
                if blocks.iter().any(|b| b.name == block.name) {
                    let children =
                        render_nodes(&block.body, blocks, inlines, ctx, renderer, diagnostics);
                    out.push(renderer.render_block(block, ctx, children));
                } else {
                    diagnostics.push(Diagnostic {
                        level: Level::Error,
                        code: ErrorCode::UnknownBlock,
                        message: format!("unknown block '{}'", block.name),
                        span: block.span.clone(),
                        suggestion: None,
                    });
                }
            }
        }
    }

    out
}

fn render_markdown_text(
    text: &str,
    inlines: &[InlineSpec],
    ctx: &EvalContext,
    renderer: &dyn DomRenderer,
    diagnostics: &mut Vec<Diagnostic>,
) -> Vec<VNode> {
    let mut out = Vec::new();
    let mut last = 0;

    for inline in parse_inline_exts(text) {
        let local_start = inline.span.start;
        let local_end = inline.span.end;

        if local_start > last {
            out.push(VNode::Text(text[last..local_start].to_string()));
        }

        if inlines.iter().any(|i| i.name == inline.name) {
            out.push(renderer.render_inline(&inline, ctx));
        } else {
            diagnostics.push(Diagnostic {
                level: Level::Error,
                code: ErrorCode::UnknownInline,
                message: format!("unknown inline '{}'", inline.name),
                span: inline.span,
                suggestion: None,
            });
        }

        last = local_end;
    }

    if last < text.len() {
        out.push(VNode::Text(text[last..].to_string()));
    }

    out
}
