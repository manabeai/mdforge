use crate::ast::ArgSpec;

use super::{BlockSpec, Forge, InlineSpec};

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
