#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VNode {
    Element(VElement),
    Text(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VElement {
    pub tag: String,
    pub attrs: Vec<(String, String)>,
    pub children: Vec<VNode>,
}
