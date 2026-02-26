use std::collections::{HashMap, HashSet};

use mdforge::forge::DomRenderer;
use mdforge::{
    ArgType, BlockNode, Diagnostic, ErrorCode, EvalContext, Forge, InlineExt, VElement,
    VNode,
};

struct IntegrationRenderer;

impl DomRenderer for IntegrationRenderer {
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

fn assert_has_code(errors: &[Diagnostic], code: ErrorCode) {
    assert!(
        errors.iter().any(|d| d.code == code),
        "expected diagnostics to contain {:?}, got: {:?}",
        code,
        errors
    );
}

#[test]
fn integration_happy_path_nested_block_inline_and_dynamic_enum() {
    let forge = Forge::builder()
        .block("card")
        .arg("title", ArgType::String.required())
        .arg("kind", ArgType::StaticEnum(&["info", "warn"]).required())
        .arg("ref", ArgType::DynamicEnum("items").required())
        .body_markdown()
        .register()
        .inline("badge")
        .arg("level", ArgType::Int.required())
        .register()
        .build();

    let input = ":::card title=hello kind=info ref=item-1\nBody {badge level=2}\n:::\n";
    let doc = forge.parse(input).expect("parse should succeed");

    forge.validate(&doc).expect("validate should succeed");

    let mut dynamic_values = HashMap::new();
    dynamic_values.insert(
        "items".to_string(),
        HashSet::from(["item-1".to_string(), "item-2".to_string()]),
    );
    let ctx = EvalContext { dynamic_values };
    forge.eval(&doc, &ctx).expect("eval should succeed");

    let dom = forge
        .render_dom(&doc, &ctx, &IntegrationRenderer)
        .expect("render should succeed");

    assert_eq!(dom.len(), 1);
    match &dom[0] {
        VNode::Element(root) => {
            assert_eq!(root.tag, "block:card");
            assert_eq!(root.children.len(), 3);
            assert!(matches!(&root.children[0], VNode::Text(t) if t == "Body "));
            assert!(matches!(&root.children[1], VNode::Element(el) if el.tag == "inline:badge"));
            assert!(matches!(&root.children[2], VNode::Text(t) if t == "\n"));
        }
        other => panic!("expected block root element, got {:?}", other),
    }
}

#[test]
fn integration_validation_reports_error_kinds_for_invalid_args() {
    let forge = Forge::builder()
        .block("card")
        .arg("title", ArgType::String.required())
        .arg("kind", ArgType::StaticEnum(&["info", "warn"]).required())
        .register()
        .inline("badge")
        .arg("level", ArgType::Int.required())
        .register()
        .build();

    let input = ":::card kind=invalid unknown=x\nBody {badge level=not-int wrong=1}\n:::\n";
    let doc = forge.parse(input).expect("parse should succeed");
    let errors = forge.validate(&doc).expect_err("validate should fail");

    assert_has_code(&errors, ErrorCode::MissingRequiredArg);
    assert_has_code(&errors, ErrorCode::UnknownArg);
    assert_has_code(&errors, ErrorCode::InvalidStaticEnumValue);
    assert_has_code(&errors, ErrorCode::InvalidType);
}

#[test]
fn integration_boundary_empty_document_roundtrips_all_stages() {
    let forge = Forge::builder().build();
    let doc = forge.parse("").expect("empty parse should succeed");

    forge.validate(&doc).expect("empty validate should succeed");
    forge
        .eval(&doc, &EvalContext::default())
        .expect("empty eval should succeed");

    let dom = forge
        .render_dom(&doc, &EvalContext::default(), &IntegrationRenderer)
        .expect("empty render should succeed");

    assert!(dom.is_empty());
}
