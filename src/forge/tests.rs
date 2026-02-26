use std::collections::{HashMap, HashSet};

use crate::{ArgType, VElement};

use super::{DomRenderer, EvalContext, Forge};
use crate::{BlockNode, InlineExt, VNode};

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

#[test]
fn parse_block_and_validate_success() {
    let forge = Forge::builder()
        .block("card")
        .arg("title", ArgType::String.required())
        .arg("level", ArgType::Int.optional())
        .body_markdown()
        .register()
        .build();

    let doc = forge
        .parse(":::card title=hello level=1\ncontent\n:::\n")
        .expect("parse should succeed");

    assert!(forge.validate(&doc).is_ok());
}

#[test]
fn parse_unclosed_block_returns_error() {
    let forge = Forge::builder().block("card").register().build();

    let diagnostics = forge
        .parse(":::card\ncontent")
        .expect_err("parse should fail for unclosed block");

    assert!(diagnostics
        .iter()
        .any(|d| d.code == crate::ErrorCode::BlockNotClosed));
}

#[test]
fn validate_reports_unknown_and_missing_args() {
    let forge = Forge::builder()
        .block("card")
        .arg("title", ArgType::String.required())
        .register()
        .build();

    let doc = forge
        .parse(":::card wrong=x\n:::\n")
        .expect("parse should succeed");

    let diagnostics = forge.validate(&doc).expect_err("validate should fail");

    assert!(diagnostics
        .iter()
        .any(|d| d.code == crate::ErrorCode::MissingRequiredArg));
    assert!(diagnostics
        .iter()
        .any(|d| d.code == crate::ErrorCode::UnknownArg));
}

#[test]
fn validate_static_enum_and_eval_dynamic_enum() {
    let forge = Forge::builder()
        .block("card")
        .arg("kind", ArgType::StaticEnum(&["a", "b"]).required())
        .arg("ref", ArgType::DynamicEnum("items").required())
        .register()
        .build();

    let doc = forge
        .parse(":::card kind=c ref=item-1\n:::\n")
        .expect("parse should succeed");
    let validate_errors = forge.validate(&doc).expect_err("static enum should fail");
    assert!(validate_errors
        .iter()
        .any(|d| d.code == crate::ErrorCode::InvalidStaticEnumValue));

    let mut dynamic_values = HashMap::new();
    dynamic_values.insert(
        "items".to_string(),
        HashSet::from(["item-2".to_string(), "item-3".to_string()]),
    );
    let eval_errors = forge
        .eval(&doc, &EvalContext { dynamic_values })
        .expect_err("dynamic enum should fail");
    assert!(eval_errors
        .iter()
        .any(|d| d.code == crate::ErrorCode::InvalidDynamicEnumValue));
}

struct TestRenderer;

impl DomRenderer for TestRenderer {
    fn render_block(&self, block: &BlockNode, _ctx: &EvalContext, children: Vec<VNode>) -> VNode {
        VNode::Element(VElement {
            tag: format!("block:{}", block.name),
            attrs: vec![],
            children,
        })
    }

    fn render_inline(&self, inline: &InlineExt, _ctx: &EvalContext) -> VNode {
        VNode::Element(VElement {
            tag: format!("inline:{}", inline.name),
            attrs: vec![],
            children: vec![],
        })
    }
}

#[test]
fn render_dom_splits_markdown_and_inline() {
    let forge = Forge::builder().inline("badge").register().build();
    let doc = forge
        .parse("hello {badge} world")
        .expect("parse should succeed");

    let dom = forge
        .render_dom(&doc, &EvalContext::default(), &TestRenderer)
        .expect("render should succeed");

    assert_eq!(dom.len(), 3);
    assert!(matches!(&dom[0], VNode::Text(t) if t == "hello "));
    assert!(matches!(&dom[1], VNode::Element(el) if el.tag == "inline:badge"));
    assert!(matches!(&dom[2], VNode::Text(t) if t == " world\n"));
}
