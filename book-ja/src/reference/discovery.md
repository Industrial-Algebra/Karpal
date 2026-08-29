# ディスカバリランタイムリファレンス

`karpal-discovery` クレート (フェーズ 19) はエージェントファーストなディスカバリランタイムです: 型付き構造カタログ、キュレーション済みセマンティックオーバーレイ、プロジェクト検査、インポートシンボル分析、圏論的プランナー、そして代数的プローブ — `karpal` バイナリは Lonis 準拠の `SubprocessProvider` です(CLI の使い方は[ガイド](../guide/karpal-discovery.md)を参照)。

これは **2 番目の Lonis バーティカル**です(1 番目は `amari-discovery`)。Amari のディスカバリがホログラフィックな想起とトロピカルランキングをドッグフーディングするのに対し、Karpal のものは **Karpal 自身の型クラス**をドッグフーディングします — プランナーのスコア集約は `Semigroup`/`Monoid` そのもの、ランキングは `BoundedLattice`、プランは `Free` モナドです。ライブラリが自身のディスカバリを駆動できなければ、他の誰かのを駆動できるとは言えません。

## アーキテクチャ

```
        ┌────────────────────────── karpal-discovery (std 専用) ────────────────────────┐
        │                                                                                │
  extract.rs ─► Catalog ──┬──► overlay.rs   ConceptOverlay (83 個のキュレーション済み概念) │
  (syn AST 走査)          │   (include_str! + ドリフトゲート)                              │
        │                 ├──► imports.rs    ImportsReport (use 文の分析)                 │
  inspect.rs ─► ProjectSnapshot (Cargo.toml、cargo なし)                                  │
        │                 ├──► planner.rs    recommend() / plan() — ドッグフーディング    │
        │                 │        karpal-core, karpal-algebra, karpal-free              │
  probes.rs   ────────────┴──► probe_catalog() / run_probe() — ドッグフーディング         │
                 karpal-proof, karpal-recursion, karpal-diagram, karpal-schubert-types   │
        └────────────────────────────────────────────────────────────────────────────────┘
                                    │  (lonis フィーチャ、オプション)
                            payload.rs → Block<karpal.*> → main.rs: karpal バイナリ、
                            9 つのツールを持つ SubprocessProvider
```

分析基盤は **lonis 非依存**です: カタログ、オーバーレイ、検査、インポート、プランナー、プローブは素のライブラリコードです。出力層(`Block` ラッピング、ワイヤプロトコル、バイナリ)だけが `lonis` フィーチャでゲートされます — Lonis 0.1.0 以降これは crates.io のレジストリ依存なので、クレートとバイナリは公開可能です。

## 構造カタログ (19-A)

`extract_workspace(root) -> Catalog` は本物の `syn` パースでワークスペースを走査し、決定論的かつ内容ハッシュ可能な形で記録します:

- **クレート**: 名前、バージョン、説明、フィーチャ、依存、モジュール。
- **公開項目**: トレイト (スーパートレイト、メソッド、関連項目)、関数 (シグネチャ)、構造体、列挙型、型エイリアス、マクロ — `#[macro_export]` 付きの宣言的 `macro_rules!`、およびプロシージャルマクロは**インポート可能な名前**の下に収録されます(`#[proc_macro_derive(VerifySemigroup)]` 関数は `VerifySemigroup` として収録。関数名はプロシージャルマクロクレートの外に出ません)。
- **再エクスポート**: リネームを含む `pub use` の葉 (`MonteCarloVerifier as AmariMonteCarloVerifier`)。glob と `std`/`core`/`alloc` 由来はスキップ。
- **実装グラフ**: `Catalog::implementors_of("Functor")` → ワークスペース全体でそれを実装する型。

## 概念オーバーレイ (19-B)

`load_concept_overlay() -> ConceptOverlay` は `include_str!` で埋め込まれた**チェックイン済みの手動キュレーション TOML** をデシリアライズします — crates.io インストールにソースチェックアウトは不要です。各 `ConceptRecord` は以下を持ちます:

- `id`、表示名 `name`、`summary`、検索用エイリアス `aliases`;
- **`problem_shapes`** — ユーザーやエージェントが問題を述べる言い方で書かれた問題の形 (「依存する効果的なステップを逐次実行したい」「局所的に整合なデータを大域的な切断に貼り合わせる」);
- `math_concepts`、修飾付き `symbol_refs` (`karpal-core::Functor`)、`StabilityTier`、`CostHint`。

概念は有向 `ConceptRelation` で結ばれます: `generalizes`(検証済みトレイトのスーパートレイト関係を映す)、`composes_with`、`alternative_to`(例: `arrow-apply` ≅ `monad`)、`dual_of`(`comonad` ↔ `monad`、反変階層)。

**ドリフトゲート** — `ConceptOverlay::validate(&catalog)` — がオーバーレイの定義的保証です: すべての `symbol_ref` は実在するカタログ項目に解決され、すべての関係端点は既知の id でなければならず、概念が錨なしに浮いたり id が重複してはなりません。CI テストが埋め込みオーバーレイを実ワークスペースに対して検証するため、キュレーションがコードを追い越すことはできません。

## プロジェクト検査 (19-C)

`inspect_workspace(root) -> ProjectSnapshot` は読み取り専用の TOML/TOML-lock パースです(`cargo` 起動なし、変更なし): ワークスペースメタデータ (メンバー、resolver)、クレートごとのパッケージメタデータ、**ソース識別**付きの依存 (`Registry`/`Path`/`Git`/`Workspace`)、フィーチャ、ターゲット (明示的 + 従来規約による自動発見)、推論されたプラットフォーム制約 (`no_std` リンケージモード)、そして `Cargo.lock` から解決済み依存。`content_hash` がスナップショット全体をカバーします。

## インポートシンボル分析 (19-B/C)

`analyze_imports(root, &catalog) -> ImportsReport` は対象プロジェクトの `use` 文をパースしカタログに対して解決します:

- **`resolved`** — 修飾シンボル参照 (項目種別、ローカル名 (エイリアス含む)、ファイル広がり、出現回数)。解決は葉トレラント(`use karpal_core::functor::Functor` は `Functor` にマッチ)かつ再エクスポート対応です。
- **`unresolved`** — カタログ内クレートを名指すが項目がないインポート: **ドリフトシグナル**。これを Karpal 自身のワークスペースに向けたところ、3 つの実在するカタログの盲点 (定数、derive 名、再エクスポート) が浮上し、すべて修正されました。
- **`globs`** — パスごとに記録、展開はしない。

`ImportsReport::concepts_used(&overlay)` は解決済みシンボルをキュレーション済み概念に結合します — 「このプロジェクトは圏論的に何をしているのか?」への答えです。

## プランナー (19-F)

`recommend(goal, &overlay)` と `plan(goal, &recommendation)` は、Karpal 自身の抽象をランタイムで動かします:

| 段階 | 基盤 (実際のトレイト使用) |
|---|---|
| スコア集約 | `karpal-core` `Semigroup`/`Monoid` — 証拠は `Monoid::combine` で蓄積。完全一致の id/名前は部分一致より上位 |
| パレートランキング | `karpal-algebra` `Lattice`/`BoundedLattice` — 厳密支配は束の join そのもの (`a ⊔ c = a ∧ a ≠ c`)。順位は支配者数 → relevance → weight → id |
| プラン構築 | `karpal-free` `Free` モナド — プランは `lift_f` + `chain` で構築される `Free<PlanF, ()>`。構造的カタモルフィズムで消費され、隣接重複ステップは畳み込まれる |

リコールはキュレーション済み全フィールドへの直接マッチで播種し、関係グラフに沿って両方向 1 ホップ展開します。すべての項目は証拠を持ちます (`match: problem shape`、`relation: dual_of comonad ↔ monad`)。*問題*として述べられたゴール — 「依存する効果的なステップを逐次実行したい」— は `monad` を想起し、それを中心にプランを組みます。

0.9.1 からは部分文字列層の下に**トークン層**が加わり、概念のサマリとエイリアスがトークン単位で索引されます。自然言語のゴール (「最小上界の結び」→ `lattice`、「2 つの層の交換」→ `traversable`) は軽いステマー付きの語彙一致で想起されます。また何も信頼できる形で想起されなかった場合、ペイロードは自ら説明します — 「言い換えが悪い」のか「存在しない」のかを区別する `diagnostics` ノートと、語彙的に最も近い概念です。

**トポスに関する正直な注記:** `karpal-topos` の `SmallCategory`/`Presheaf`/米田の仕組みは型レベル (静的型上の GAT) であり、83 個のランタイム概念を通すのは装飾にしかなりません。ランタイムのケイパビリティ圏はオーバーレイの関係グラフそのものです。米田に基づくより深いリコールの物語は、トポスクレートが実戦検証されるまで見送られます — 1.0 候補です。

## プローブ (19-G)

`probe_catalog()` と `run_probe(id)` — 有界で、読み取り専用で、決定論的な実行がライブラリをドッグフーディングします:

| プローブ | ドッグフード | 実証すること |
|---|---|---|
| `functor-monad-laws` | `karpal-core` | `Option` 上の関手の同一性・合成とモナドの単位元・結合律 |
| `algebra-laws` | `karpal-proof`, `karpal-core`, `karpal-algebra` | 法則チェッカー自身: 結合律、単位元、プランナーの `Score` 束上の吸収律 |
| `schubert-intersection` | `karpal-schubert-types` | Gr(2,4) での `IntersectionKind` 判別: σ₁·σ₁ は **Positive**、σ₂₂·σ₂₂ は **StructuralZero** — [構造化された空](../concepts/structured-emptiness.md)が動く |
| `recursion-eval` | `karpal-recursion` | `ana` がペアノ数を構築、`cata` が解体、`hylo ≡ cata ∘ ana` |
| `coherence` | `karpal-diagram` | 五角形・三角形・六角形の `Rewrite` 証人 |

## ワイヤコントラクトとハードニング (19-H)

- **ゴールデンテスト**がバイナリの出力面を固定します: プロバイダ JSON はバイト単位 (静的内容)、ブロック出力は揮発性のタイムスタンプを正規化した `data` 上でゴールデン。再生成は意図的に (`KARPAL_UPDATE_GOLDENS=1`)。ゴールデンの変更はコントラクトの変更です。
- **`--index-compat`** は新しいカタログの上で従来の `karpal-index` の JSON の形を話します ([ガイド](../guide/karpal-discovery.md)参照)。
- **公開順序ドリフトゲート**が、すべてのワークスペースメンバーが `publish.yml` の公開列に現れることを要求します。

## ライブラリクイックスタート

```rust
use karpal_discovery::{extract_workspace, load_concept_overlay, analyze_imports, recommend, plan};

// 構造カタログ
let catalog = extract_workspace(std::path::Path::new("."));

// キュレーション済み概念 — カタログに対して検証 (ドリフトゲート)
let overlay = load_concept_overlay();
overlay.validate(&catalog).expect("no drift");

// このプロジェクトはどの概念を使っているか?
let report = analyze_imports(std::path::Path::new("."), &catalog);
for concept in report.concepts_used(&overlay) {
    println!("in use: {} ({})", concept.id, concept.summary);
}

// プランナー: 問題を尋ねれば、ランク付き概念とプランが得られる
let recommendation = recommend("sequence dependent effectful steps", &overlay);
let plan = plan("sequence dependent effectful steps", &recommendation);
```

ここにあるすべては読み取り専用・決定論的・オフラインです — `cargo` 起動なし、ネットワークなし、変更なし。
