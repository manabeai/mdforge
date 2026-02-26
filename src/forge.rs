use std::collections::{HashMap, HashSet};

use crate::{ast::ArgSpec, BlockNode, Diagnostic, Document, InlineExt, VNode};

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

    pub fn signature(&self) -> String {
        let mut lines = Vec::new();

        for block in &self.blocks {
            lines.push(format!("Block: {}", block.name));
            let args = block
                .args
                .iter()
                .map(|(name, spec)| {
                    format!("{}={}", name, spec.arg_type.signature_label(spec.required))
                })
                .collect::<Vec<_>>()
                .join(" ");

            let head = if args.is_empty() {
                format!(":::{}", block.name)
            } else {
                format!(":::{} {}", block.name, args)
            };
            lines.push(head);
            if block.body_markdown {
                lines.push("Body: markdown".to_string());
            }
            lines.push(String::new());
        }

        for inline in &self.inlines {
            lines.push(format!("Inline: {}", inline.name));
            let args = inline
                .args
                .iter()
                .map(|(name, spec)| {
                    format!("{}={}", name, spec.arg_type.signature_label(spec.required))
                })
                .collect::<Vec<_>>()
                .join(" ");
            let body = if args.is_empty() {
                format!("{{{}}}", inline.name)
            } else {
                format!("{{{} {}}}", inline.name, args)
            };
            lines.push(body);
            lines.push(String::new());
        }

        while matches!(lines.last(), Some(last) if last.is_empty()) {
            lines.pop();
        }

        lines.join("\n")
    }

    pub fn parse(&self, _input: &str) -> Result<Document, Vec<Diagnostic>> {
        Ok(Document { nodes: Vec::new() })
    }

    pub fn validate(&self, _doc: &Document) -> Result<(), Vec<Diagnostic>> {
        Ok(())
    }

    pub fn eval(
        &self,
        _doc: &Document,
        dynamic_ctx: &EvalContext,
    ) -> Result<EvalContext, Vec<Diagnostic>> {
        Ok(dynamic_ctx.clone())
    }

    pub fn render_dom(
        &self,
        _doc: &Document,
        _ctx: &EvalContext,
        _renderer: &dyn DomRenderer,
    ) -> Result<Vec<VNode>, Vec<Diagnostic>> {
        Ok(Vec::new())
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
mod tests {
    use crate::ArgType;

    use super::Forge;

    #[test]
    fn signature_matches_expected_shape() {
        let forge = Forge::builder()
            .block("card")
            .arg("title", ArgType::String.required())
            .arg("level", ArgType::Int.optional())
            .body_markdown()
            .register()
            .inline("badge")
            .arg("level", ArgType::Int.required())
            .register()
            .build();

        let expected = [
            "Block: card",
            ":::card title=<string> level=<int?>",
            "Body: markdown",
            "",
            "Inline: badge",
            "{badge level=<int>}",
        ]
        .join("\n");

        assert_eq!(forge.signature(), expected);
    }
}
