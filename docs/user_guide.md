# mdforge ユーザーガイド

このドキュメントは、`mdforge` を**利用する側**（アプリケーション開発者）が、
- 拡張 Markdown の仕様（ブロック / インライン）を定義し、
- 入力をパース・検証し、
- DOM 中間表現へ変換する
までの基本フローを理解できるようにまとめたものです。

## 1. mdforge でできること

`mdforge` は、拡張 Markdown の「許可する構文」を先に定義してから、文章を扱います。

主な機能:

1. **スキーマ定義**: ブロック拡張 / インライン拡張と引数型を定義
2. **シグネチャ出力**: LLM に渡すための仕様文字列を生成
3. **パイプライン実行**: parse → validate → eval → render_dom
4. **DOM 中間表現**: レンダラ経由で `VNode` を生成

> 現在の実装では `parse` / `validate` / `eval` / `render_dom` は最小実装（スタブ）です。将来の厳密検証・変換の拡張を前提とした API になっています。

---

## 2. クイックスタート

### 2.1 Forge を組み立てる

```rust
use mdforge::{ArgType, Forge};

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
```

### 2.2 LLM に仕様を伝える

```rust
let sig = forge.signature();
println!("{}", sig);
```

出力例:

```text
Block: card
:::card title=<string> level=<int?>
Body: markdown

Inline: badge
{badge level=<int>}
```

この文字列をそのままプロンプトに埋め込むことで、LLM の出力を許可構文に寄せやすくなります。

---

## 3. 拡張の定義 API

### 3.1 ブロック拡張

ブロックは `ForgeBuilder::block(name)` で開始し、必要に応じて引数や本文仕様を追加して `register()` します。

- `arg(name, ArgType::... .required()/.optional())`: 引数仕様を追加
- `body_markdown()`: 本文に Markdown を許可
- `register()`: 定義を `ForgeBuilder` に戻す

### 3.2 インライン拡張

インラインは `ForgeBuilder::inline(name)` で開始し、引数を追加して `register()` します。

### 3.3 引数型 (`ArgType`)

- `ArgType::Int`
- `ArgType::String`
- `ArgType::StaticEnum(&[...])`
- `ArgType::DynamicEnum("namespace")`

`required()` / `optional()` で必須・任意を宣言します。

---

## 4. パイプライン実行

`mdforge` の想定フローは次のとおりです。

1. `parse(input)`
2. `validate(&doc)`
3. `eval(&doc, &dynamic_ctx)`
4. `render_dom(&doc, &ctx, renderer)`

サンプル:

```rust
use mdforge::{EvalContext, Forge};

fn run_pipeline(forge: &Forge, input: &str, renderer: &dyn mdforge::forge::DomRenderer) {
    let doc = match forge.parse(input) {
        Ok(doc) => doc,
        Err(diags) => {
            eprintln!("parse error: {:?}", diags);
            return;
        }
    };

    if let Err(diags) = forge.validate(&doc) {
        eprintln!("validate error: {:?}", diags);
        return;
    }

    let ctx = EvalContext::default();
    let evaluated = match forge.eval(&doc, &ctx) {
        Ok(ctx) => ctx,
        Err(diags) => {
            eprintln!("eval error: {:?}", diags);
            return;
        }
    };

    let vnodes = match forge.render_dom(&doc, &evaluated, renderer) {
        Ok(nodes) => nodes,
        Err(diags) => {
            eprintln!("render error: {:?}", diags);
            return;
        }
    };

    println!("generated nodes: {}", vnodes.len());
}
```

> 注: 現状では上記各処理はスタブで、成功時に空の結果を返します。

---

## 5. 動的列挙 (`DynamicEnum`) と `EvalContext`

`DynamicEnum("namespace")` を利用する場合、`EvalContext` に namespace ごとの許可値集合を渡します。

```rust
use std::collections::{HashMap, HashSet};
use mdforge::EvalContext;

let mut dynamic_values = HashMap::new();
dynamic_values.insert(
    "role".to_string(),
    ["admin", "editor", "viewer"]
        .into_iter()
        .map(str::to_string)
        .collect::<HashSet<_>>(),
);

let ctx = EvalContext { dynamic_values };
```

将来的には `eval` 段階でこの情報を使って動的列挙の値チェックを実施できます。

---

## 6. DOM レンダラの実装

`render_dom` を利用するには `DomRenderer` を実装します。

```rust
use mdforge::{BlockNode, EvalContext, InlineExt, VElement, VNode};

struct MyRenderer;

impl mdforge::forge::DomRenderer for MyRenderer {
    fn render_block(&self, block: &BlockNode, _ctx: &EvalContext, children: Vec<VNode>) -> VNode {
        VNode::Element(VElement {
            tag: format!("x-{}", block.name),
            attrs: vec![],
            children,
        })
    }

    fn render_inline(&self, inline: &InlineExt, _ctx: &EvalContext) -> VNode {
        VNode::Element(VElement {
            tag: format!("x-inline-{}", inline.name),
            attrs: vec![],
            children: vec![],
        })
    }
}
```

`VNode` は以下の 2 種類です。

- `VNode::Text(String)`
- `VNode::Element(VElement { tag, attrs, children })`

---

## 7. エラーハンドリング

各段階の失敗時は `Vec<Diagnostic>` が返されます。

`Diagnostic` には次の情報が含まれます。

- `level` (`Error` / `Warning`)
- `code`（`UnknownBlock` など）
- `message`
- `span`（入力中の位置）
- `suggestion`

想定される `ErrorCode`:

- `UnknownBlock`
- `UnknownInline`
- `MissingRequiredArg`
- `UnknownArg`
- `InvalidType`
- `InvalidStaticEnumValue`
- `InvalidDynamicEnumValue`
- `BlockNotClosed`

---

## 8. 運用のコツ（LLM 連携）

1. **毎回 `signature()` をプロンプトに含める**
2. LLM 出力をそのまま採用せず、必ず parse/validate/eval を通す
3. 診断結果 (`Diagnostic`) をユーザー向けメッセージに変換して再入力を促す
4. `DynamicEnum` は実データ（DB/設定）から `EvalContext` を毎リクエスト構築する

---

## 9. 現在の制約

- コア API は揃っているが、内部実装は段階的に拡張予定
- 現時点では変換結果に依存した本番ロジックは慎重に導入すること
- まずは `signature()` による生成制約用途から使い始めるのがおすすめ

以上です。
