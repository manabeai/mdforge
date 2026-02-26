use crate::{Diagnostic, Document, MdEvent, Node};

use super::{
    args::eval_dynamic_args, inline::parse_inline_exts, BlockSpec, EvalContext, InlineSpec,
};

pub fn eval_document(
    doc: &Document,
    blocks: &[BlockSpec],
    inlines: &[InlineSpec],
    dynamic_ctx: &EvalContext,
) -> Result<EvalContext, Vec<Diagnostic>> {
    let mut diagnostics = Vec::new();
    eval_nodes(&doc.nodes, blocks, inlines, dynamic_ctx, &mut diagnostics);

    if diagnostics.is_empty() {
        Ok(dynamic_ctx.clone())
    } else {
        Err(diagnostics)
    }
}

fn eval_nodes(
    nodes: &[Node],
    blocks: &[BlockSpec],
    inlines: &[InlineSpec],
    ctx: &EvalContext,
    diagnostics: &mut Vec<Diagnostic>,
) {
    for node in nodes {
        match node {
            Node::Markdown(events) => {
                for event in events {
                    let MdEvent::Text(text) = event;
                    for inline in parse_inline_exts(text) {
                        if let Some(spec) = inlines.iter().find(|s| s.name == inline.name) {
                            eval_dynamic_args(
                                &inline.args,
                                &spec.args,
                                inline.span,
                                ctx,
                                diagnostics,
                            );
                        }
                    }
                }
            }
            Node::Block(block) => {
                if let Some(spec) = blocks.iter().find(|s| s.name == block.name) {
                    eval_dynamic_args(
                        &block.args,
                        &spec.args,
                        block.span.clone(),
                        ctx,
                        diagnostics,
                    );
                }
                eval_nodes(&block.body, blocks, inlines, ctx, diagnostics);
            }
        }
    }
}
