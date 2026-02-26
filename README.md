# mdforge

`mdforge` は、拡張 Markdown を**定義してから扱う**ための Rust ライブラリです。  
LLM が生成したテキストを、型付き引数・構文ルール・診断情報で安全に扱い、DOM 風の中間表現 (`VNode`) へ変換できます。

## 特徴

- 拡張ブロック / インライン構文を `ForgeBuilder` で宣言
- 引数型 (`Int` / `String` / `StaticEnum` / `DynamicEnum`) を定義して検証
- `parse -> validate -> eval -> render_dom` のパイプライン API
- `Diagnostic` (`ErrorCode`, `Span`, `suggestion`) によるエラー収集
- LLM 指示向けの仕様文字列 (`signature`) を生成

## インストール

`Cargo.toml` に追加してください。

```toml
[dependencies]
mdforge = "0.1.0"
```

## クイックスタート

```rust
use mdforge::{ArgType, EvalContext, Forge, VNode};

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

let input = ":::card title=hello level=1\ntext {badge level=2}\n:::\n";

let doc = forge.parse(input).expect("parse");
forge.validate(&doc).expect("validate");
let ctx = forge.eval(&doc, &EvalContext::default()).expect("eval");

struct NoopRenderer;
impl mdforge::forge::DomRenderer for NoopRenderer {
    fn render_block(
        &self,
        _block: &mdforge::BlockNode,
        _ctx: &EvalContext,
        children: Vec<VNode>,
    ) -> VNode {
        VNode::Text(format!("block children={} ", children.len()))
    }

    fn render_inline(&self, _inline: &mdforge::InlineExt, _ctx: &EvalContext) -> VNode {
        VNode::Text("inline".to_string())
    }
}

let nodes = forge
    .render_dom(&doc, &ctx, &NoopRenderer)
    .expect("render_dom");

println!("nodes: {}", nodes.len());
println!("signature:\n{}", forge.signature());
```

## 主要 API

- `Forge::builder()` / `ForgeBuilder`
- `Forge::parse`
- `Forge::validate`
- `Forge::eval`
- `Forge::render_dom`
- `Forge::signature`

詳しい使い方は以下を参照:

- `docs/user_guide.md`
- `docs/design_v0.1.md`

## ライセンス

このプロジェクトは [MIT License](./LICENSE) のもとで提供されます。
