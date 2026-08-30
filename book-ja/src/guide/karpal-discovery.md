# `karpal` バイナリによるディスカバリ

`karpal-discovery` はフェーズ 19 のエージェントファーストなディスカバリランタイム、2 番目の [Lonis](https://github.com/Industrial-Algebra/Lonis) バーティカルです。付属の `karpal` バイナリは、人間向け CLI であると同時に Lonis 準拠の `SubprocessProvider` でもあります。AI エージェントのハーネスは一様なプロトコル経由で発見・呼び出しでき、人間は同じコマンドを直接使えます。

`karpal-index` が文字列スキャンでソースを走査するのに対し、`karpal-discovery` は本物の `syn` AST パースによって**型付きで決定論的なカタログ**を構築し、その上に**キュレーション済みの数学的オーバーレイ**(問題の形と関係性を持つ 83 個の概念)を重ねます。そしてエージェントが実際に尋ねる水準で質問に答えます。「*依存する効果的なステップを逐次実行したい*」→ `monad`。

## インストール

バイナリは `lonis` フィーチャ付きでビルドします:

```sh
cargo install --path karpal-discovery --features lonis --bin karpal
```

ライブラリ自体はフィーチャなしで利用可能です(CLI と Lonis 出力層はオプション):

```toml
[dependencies]
karpal-discovery = "0.9"
```

## Lonis プロバイダプロトコル

`karpal` は ADR-0006 v0 プロバイダサーフェスを話します — `lonis` 自身が話すのと同じプロトコルなので、どの Lonis ホストでも専用の接着剤なしに利用できます:

```bash
$ karpal --mode json manifest
{"name":"karpal","version":"0.9.0","provider_type":"external-executable",
 "protocol_version":"0","tools":["karpal.search","karpal.detail",...],
 "display_name":"Karpal Discovery"}

$ karpal --mode json tools list
$ karpal --mode json tools describe karpal.recommend
{"name":"karpal:recommend","description":"Recall and Pareto-rank curated concepts...",
 "determinism":"deterministic","side_effects":"read-only","cost":"low"}
```

起動は ADR-0003: stdin に JSON、stdout にブロック配列、stderr に構造化された `ToolError`。インプロセスのホストは `lonis_core::SubprocessProvider` を使い、シェルからも同じように使えます:

```bash
$ echo '{"workspace": ".", "query": "Functor"}' | karpal call karpal.search
```

すべてのツールは決定論的・読み取り専用・低コストで、各ツールのコントラクトに明記されています。

## 9 つのツール

### `karpal.search` — カタログ項目

```json
{"workspace": ".", "query": "Functor"}
```

ワークスペースカタログ内のすべての公開項目名(トレイト、関数、構造体、列挙型、型エイリアス、マクロ、定数)に対する大文字小文字を区別しない部分一致検索です。

### `karpal.detail` — 1 項目の完全な結合

```json
{"workspace": ".", "item": "Functor", "crate": "karpal-core"}
```

項目をドキュメント、実装者(トレイト実装グラフから)、それに紐づくオーバーレイ概念とともに返します。`Functor` なら:

```json
{
  "item": {"name": "Functor", "crate_name": "karpal-core",
           "module_path": "karpal_core::functor", "kind": "trait"},
  "docs": "Covariant functor: lifts a function `A -> B` into `F<A> -> F<B>`.",
  "implementors": ["CofreeF", "ComposeF", "EnvF", "FixF", "FreeF",
                   "IdentityF", "NonEmptyVecF", "OptionF", "ResultF", "VecF"],
  "concepts": [{"id": "functor", "stability": "stable", ...}]
}
```

### `karpal.concepts` — オーバーレイの閲覧

```json
{"query": "sheaf"}
```

id、名前、エイリアス、数学的概念、そして**問題の形 (problem shapes)** にわたってキュレーション済み概念を検索します。空のクエリは 83 個すべてを一覧表示します。

### `karpal.imports` — プロジェクトが実際に使うもの

```json
{"workspace": "."}
```

ワークスペースの `use` 文を自身のカタログに対して分析します: 解決済みシンボル(ファイルごとの計数付き)、未解決のインポート(ドリフトシグナル — 削除・改名された項目への古い参照)、そして使用中のキュレーション済み**概念**。

### `karpal.recommend` — ゴールのためのリコールとランキング

```json
{"goal": "sequence dependent effectful steps"}
```

プランナーのリコール層: 直接マッチが候補を播種し関係グラフが展開します。ランキングは (relevance, weight) 上のパレート支配です。自然言語のゴールも機能します — サマリとエイリアスがトークン単位で索引されるため (0.9.1)、「least upper bound join」は `lattice` を想起します。問題を尋ねれば概念が得られます — `monad` が第一位、証拠付き:

```json
{
  "concept_id": "monad",
  "stability": "stable",
  "relevance": 14,
  "evidence": ["match: problem shape",
               "relation: composes_with free-monad ↔ monad",
               "relation: dual_of comonad ↔ monad"]
}
```

### `karpal.plan` — 準備・探索・検証

```json
{"goal": "monad"}
```

ゴールのための候補プラン: ゴールへの orient、上位ランクの概念の探索(3 個まで)、ドリフトゲートの検証。プランは `Free` モナドとして構築され正規化されます(隣接する重複ステップは畳み込まれます)。

### `karpal.probe_list` / `karpal.probe_describe` / `karpal.probe_run` — 代数的プローブ

```json
{"id": "schubert-intersection"}
```

5 つのプローブはどれも本物のライブラリコードを実行します: `Option` 上の関手・モナド則、`karpal-proof` の法則チェッカー、シューベルト交叉(**構造化された空のテーゼが動く** — `Positive` 対 `StructuralZero`)、再帰スキームの一致(`hylo ≡ cata ∘ ana`)、マックレーンのコヒーレンス証人。`karpal.probe_run` は実証した各チェックを報告します。

## `karpal-index` 互換モード

従来の `karpal-index` の起動は互換モードでそのまま動きます:

```bash
$ karpal --index-compat search Functor --json
$ karpal --index-compat hierarchy Monad --json
```

JSON の形は新しいカタログの上に載せた従来のものです(`ApiItem`、`Hierarchy`、見つからなければ `null`)。相違点はクレートの README に文書化されています: `path` はモジュールパス(行番号なし)、`subtraits` は空 — 従来からそうです。

## 次へ

- [ディスカバリランタイムリファレンス](../reference/discovery.md) — アーキテクチャ、ライブラリ API、プランナーが Karpal 自身の型クラスをいかにドッグフーディングするか。
- [karpal-index ガイド](./karpal-index.md) — これを継承する従来のバイナリ。
