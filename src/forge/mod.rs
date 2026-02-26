use std::collections::{HashMap, HashSet};

use crate::{ast::ArgSpec, BlockNode, Diagnostic, Document, InlineExt, VNode};

mod args;
mod builder;
mod eval;
mod parse;
mod render;
mod signature;
mod validate;

pub use builder::ForgeBuilder;

pub trait DomRenderer {
    fn render_block(&self, block: &BlockNode, ctx: &EvalContext, children: Vec<VNode>) -> VNode;

    fn render_inline(&self, inline: &InlineExt, ctx: &EvalContext) -> VNode;
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct EvalContext {
    pub dynamic_values: HashMap<String, HashSet<String>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlockSpec {
    pub name: String,
    pub args: Vec<(String, ArgSpec)>,
    pub body_markdown: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InlineSpec {
    pub name: String,
    pub args: Vec<(String, ArgSpec)>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Forge {
    blocks: Vec<BlockSpec>,
    inlines: Vec<InlineSpec>,
}

impl Forge {
    pub fn builder() -> ForgeBuilder {
        ForgeBuilder::default()
    }

    pub fn parse(&self, input: &str) -> Result<Document, Vec<Diagnostic>> {
        parse::parse_document(self, input)
    }

    pub fn validate(&self, doc: &Document) -> Result<(), Vec<Diagnostic>> {
        validate::validate_document(self, doc)
    }

    pub fn eval(
        &self,
        doc: &Document,
        dynamic_ctx: &EvalContext,
    ) -> Result<EvalContext, Vec<Diagnostic>> {
        eval::eval_document(self, doc, dynamic_ctx)
    }

    pub fn render_dom(
        &self,
        doc: &Document,
        ctx: &EvalContext,
        renderer: &dyn DomRenderer,
    ) -> Result<Vec<VNode>, Vec<Diagnostic>> {
        render::render_document(self, doc, ctx, renderer)
    }

    pub fn signature(&self) -> String {
        signature::build_signature(self)
    }
}

#[cfg(test)]
mod tests;
