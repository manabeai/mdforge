# mdforge 設計書 v0.1

## 1. 目的

**mdforge** は、LLM に独自拡張 Markdown を書かせ、それを厳密に検証し、最終的に DOM 構造へコンパイルする Rust ライブラリである。

## 2. 解決したい課題

生成 AI に Markdown を書かせると、以下の問題が発生する。

- 構文が壊れる
- 未定義参照が出る
- 値の型が間違う
- 表現を制御できない

mdforge は以下の仕組みでこれを制御する。

- 拡張構文の定義
- 型付き引数検証
- dynamic enum 検証
- 構造化エラー
- DOM 出力

## 3. 非目標

- Markdown 完全再実装はしない
- HTML レンダリングエンジンにはならない
- フロントエンドフレームワークにはならない

## 4. 全体アーキテクチャ

```text
拡張Markdown
  ↓
parse（ブロック・インライン抽出）
  ↓
validate（構文 + 型）
  ↓
eval（dynamic enum）
  ↓
render_dom（VNode生成）
  ↓
to_html / JSON / React変換
```

## 5. 言語仕様

### 5.1 ブロック拡張

構文:

```markdown
:::name key=value key=value
body
:::
```

条件:

- 行頭から開始
- 終端は単独 `:::`
- 本文内にインライン拡張を許可

### 5.2 インライン拡張

構文:

```markdown
{name key=value key=value}
```

条件:

- 通常 Markdown 内どこでも出現可能
- ネストは初期バージョンでは禁止
- 自己完結型のみ

### 5.3 引数型

```rust
pub enum ArgType {
    Int,
    String,
    StaticEnum(&'static [&'static str]),
    DynamicEnum(&'static str),
}
```

検証段階:

| 型 | 検証タイミング |
| --- | --- |
| Int | validate |
| String | validate |
| StaticEnum | validate |
| DynamicEnum | eval |

## 6. AST 設計

### 6.1 Document

```rust
pub struct Document {
    pub nodes: Vec<Node>,
}
```

### 6.2 Node

```rust
pub enum Node {
    Markdown(Vec<MdEvent>),
    Block(BlockNode),
}
```

### 6.3 BlockNode

```rust
pub struct BlockNode {
    pub name: String,
    pub args: HashMap<String, ArgValue>,
    pub body: Vec<Node>,
    pub span: Span,
}
```

### 6.4 Inline

```rust
pub struct InlineExt {
    pub name: String,
    pub args: HashMap<String, ArgValue>,
    pub span: Span,
}
```

## 7. DOM 中間表現

### 7.1 VNode

```rust
pub enum VNode {
    Element(VElement),
    Text(String),
}
```

### 7.2 VElement

```rust
pub struct VElement {
    pub tag: String,
    pub attrs: Vec<(String, String)>,
    pub children: Vec<VNode>,
}
```

## 8. Renderer 設計

```rust
pub trait DomRenderer {
    fn render_block(
        &self,
        block: &BlockNode,
        ctx: &EvalContext,
        children: Vec<VNode>,
    ) -> VNode;

    fn render_inline(
        &self,
        inline: &InlineExt,
        ctx: &EvalContext,
    ) -> VNode;
}
```

## 9. 検証とフィードバック

### 9.1 Diagnostic

```rust
pub struct Diagnostic {
    pub level: Level,
    pub code: ErrorCode,
    pub message: String,
    pub span: Span,
    pub suggestion: Option<String>,
}
```

### 9.2 ErrorCode

```rust
pub enum ErrorCode {
    UnknownBlock,
    UnknownInline,
    MissingRequiredArg,
    UnknownArg,
    InvalidType,
    InvalidStaticEnumValue,
    InvalidDynamicEnumValue,
    BlockNotClosed,
}
```

## 10. AI 指示用シグネチャ生成

```rust
pub fn signature(&self) -> String;
```

出力例:

```text
Block: card
:::card title=<string> level=<int?>
Body: markdown

Inline: badge
{badge level=<int>}
```

Spec と常に一致する。

## 11. DynamicEnum 評価

```rust
pub struct EvalContext {
    pub dynamic_values: HashMap<String, HashSet<String>>,
}
```

eval 段階で:

- DynamicEnum 値の存在チェック
- エラー時に候補一覧提示

## 12. 公開 API

```rust
let forge = Forge::builder()
    .block("card")
        .arg("title", ArgType::String.required())
        .body_markdown()
        .register()
    .inline("badge")
        .arg("level", ArgType::Int.required())
        .register()
    .build();

let doc = forge.parse(input)?;
forge.validate(&doc)?;
let ctx = forge.eval(&doc, &dynamic_ctx)?;
let dom = forge.render_dom(&doc, &ctx, &renderer)?;
```

## 13. 将来拡張

- typed_block
- inline container 形式
- custom ArgType validator
- JSON ターゲット
- React 用 VNode 出力
- WebAssembly 対応

## 14. 成功基準

mdforge は:

- LLM 出力を確実に検証できる
- 拡張構文を厳密に制御できる
- HTML 以外のターゲットへ拡張可能
- 単なる Markdown 拡張ではなく「DSL コンパイラ」である

## 最終定義

mdforge は、Markdown を基盤とした、型安全・評価可能・DOM 出力可能な拡張 DSL コンパイラ。
