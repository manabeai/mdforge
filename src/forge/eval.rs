use crate::{Diagnostic, Document, MdEvent, Node};

use super::{args::eval_dynamic_args, args::parse_inline_exts, EvalContext, Forge};

pub(super) fn eval_document(
    forge: &Forge,
    doc: &Document,
    dynamic_ctx: &EvalContext,
) -> Result<EvalContext, Vec<Diagnostic>> {
    let mut diagnostics = Vec::new();
    eval_nodes(forge, &doc.nodes, dynamic_ctx, &mut diagnostics);
    if diagnostics.is_empty() {
        Ok(dynamic_ctx.clone())
    } else {
        Err(diagnostics)
    }
}

fn eval_nodes(forge: &Forge, nodes: &[Node], ctx: &EvalContext, diagnostics: &mut Vec<Diagnostic>) {
    for node in nodes {
        match node {
            Node::Markdown(events) => {
                for event in events {
                    let MdEvent::Text(text) = event;
                    for inline in parse_inline_exts(text) {
                        if let Some(spec) = forge.inlines.iter().find(|s| s.name == inline.name) {
                            eval_dynamic_args(
                                &inline.args,
                                &spec.args,
                                inline.span.clone(),
                                ctx,
                                diagnostics,
                            );
                        }
                    }
                }
            }
            Node::Block(block) => {
                if let Some(spec) = forge.blocks.iter().find(|s| s.name == block.name) {
                    eval_dynamic_args(
                        &block.args,
                        &spec.args,
                        block.span.clone(),
                        ctx,
                        diagnostics,
                    );
                }
                eval_nodes(forge, &block.body, ctx, diagnostics);
            }
        }
    }
}
