use std::collections::{HashMap, HashSet};

use crate::{ast::ArgSpec, Diagnostic, Document, VNode};

mod args;
mod eval;
mod inline;
mod parse;
mod render;
mod signature;
mod validate;

pub trait DomRenderer {
    fn render_block(
        &self,
        block: &crate::BlockNode,
        ctx: &EvalContext,
        children: Vec<VNode>,
    ) -> VNode;

    fn render_inline(&self, inline: &crate::InlineExt, ctx: &EvalContext) -> VNode;
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

    pub fn signature(&self) -> String {
        signature::build_signature(&self.blocks, &self.inlines)
    }

    pub fn parse(&self, input: &str) -> Result<Document, Vec<Diagnostic>> {
        parse::parse_document(input)
    }

    pub fn validate(&self, doc: &Document) -> Result<(), Vec<Diagnostic>> {
        validate::validate_document(doc, &self.blocks, &self.inlines)
    }

    pub fn eval(
        &self,
        doc: &Document,
        dynamic_ctx: &EvalContext,
    ) -> Result<EvalContext, Vec<Diagnostic>> {
        eval::eval_document(doc, &self.blocks, &self.inlines, dynamic_ctx)
    }

    pub fn render_dom(
        &self,
        doc: &Document,
        ctx: &EvalContext,
        renderer: &dyn DomRenderer,
    ) -> Result<Vec<VNode>, Vec<Diagnostic>> {
        render::render_document(doc, &self.blocks, &self.inlines, ctx, renderer)
    }
}

#[derive(Debug, Clone, Default)]
pub struct ForgeBuilder {
    blocks: Vec<BlockSpec>,
    inlines: Vec<InlineSpec>,
}

impl ForgeBuilder {
    pub fn block(self, name: impl Into<String>) -> BlockBuilder {
        BlockBuilder {
            parent: self,
            name: name.into(),
            args: Vec::new(),
            body_markdown: false,
        }
    }

    pub fn inline(self, name: impl Into<String>) -> InlineBuilder {
        InlineBuilder {
            parent: self,
            name: name.into(),
            args: Vec::new(),
        }
    }

    pub fn build(self) -> Forge {
        Forge {
            blocks: self.blocks,
            inlines: self.inlines,
        }
    }
}

#[derive(Debug, Clone)]
pub struct BlockBuilder {
    parent: ForgeBuilder,
    name: String,
    args: Vec<(String, ArgSpec)>,
    body_markdown: bool,
}

impl BlockBuilder {
    pub fn arg(mut self, name: impl Into<String>, spec: ArgSpec) -> Self {
        self.args.push((name.into(), spec));
        self
    }

    pub fn body_markdown(mut self) -> Self {
        self.body_markdown = true;
        self
    }

    pub fn register(self) -> ForgeBuilder {
        let mut parent = self.parent;
        parent.blocks.push(BlockSpec {
            name: self.name,
            args: self.args,
            body_markdown: self.body_markdown,
        });
        parent
    }
}

#[derive(Debug, Clone)]
pub struct InlineBuilder {
    parent: ForgeBuilder,
    name: String,
    args: Vec<(String, ArgSpec)>,
}

impl InlineBuilder {
    pub fn arg(mut self, name: impl Into<String>, spec: ArgSpec) -> Self {
        self.args.push((name.into(), spec));
        self
    }

    pub fn register(self) -> ForgeBuilder {
        let mut parent = self.parent;
        parent.inlines.push(InlineSpec {
            name: self.name,
            args: self.args,
        });
        parent
    }
}

#[cfg(test)]
mod tests;
