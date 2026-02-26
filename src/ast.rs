use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MdEvent {
    Text(String),
}

#[derive(Debug, Clone, PartialEq)]
pub enum ArgValue {
    Int(i64),
    String(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ArgType {
    Int,
    String,
    StaticEnum(&'static [&'static str]),
    DynamicEnum(&'static str),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArgSpec {
    pub arg_type: ArgType,
    pub required: bool,
}

impl ArgType {
    pub fn required(self) -> ArgSpec {
        ArgSpec {
            arg_type: self,
            required: true,
        }
    }

    pub fn optional(self) -> ArgSpec {
        ArgSpec {
            arg_type: self,
            required: false,
        }
    }

    pub fn signature_label(&self, required: bool) -> String {
        let core = match self {
            ArgType::Int => "<int>".to_string(),
            ArgType::String => "<string>".to_string(),
            ArgType::StaticEnum(values) => format!("<{}>", values.join("|")),
            ArgType::DynamicEnum(name) => format!("<dynamic:{}>", name),
        };

        if required {
            core
        } else {
            format!("{}?>", core.trim_end_matches('>'))
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Document {
    pub nodes: Vec<Node>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Node {
    Markdown(Vec<MdEvent>),
    Block(BlockNode),
}

#[derive(Debug, Clone, PartialEq)]
pub struct BlockNode {
    pub name: String,
    pub args: HashMap<String, ArgValue>,
    pub body: Vec<Node>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct InlineExt {
    pub name: String,
    pub args: HashMap<String, ArgValue>,
    pub span: Span,
}
