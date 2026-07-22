# モノイダル図式

ストリング図式、モノイダル圏、コヒーレンス証拠、検証 — `karpal-diagram` (フェーズ 13)。


## モノイダル圏トレイト

`karpal-diagram` は `karpal-arrow` の上に構築された四つのモノイダル圏トレイトを提供します:

| トレイト    | スーパートレイト | 主要メソッド                                            |
|------------|-------------|-------------------------------------------------------|
| `Tensor`   | `Arrow`     | `tensor(left, right)`、結合子、左右の単位子 |
| `Braiding` | `Tensor`    | `braid<A,B>()` — テンソル因子の入れ替え                  |
| `Symmetry` | `Braiding`  | `braid ∘ braid = id`                                  |
| `Trace`    | `Tensor`    | `trace(morphism)` — フィードバックワイヤーを閉じる             |

``` rust
use karpal_arrow::FnA;
use karpal_diagram::{Braiding, Tensor, Trace};

// テンソル積
let parallel = FnA::tensor(
    FnA::arr(|x: i32| x * 2),
    FnA::arr(|x: i32| x + 1),
);
assert_eq!(parallel((3, 4)), (6, 5));

// 組み紐 (Braiding)
let swap = FnA::braid::();
assert_eq!(swap((7, true)), (true, 7));

// トレース (フィードバックを閉じる)
let traced = FnA::trace::(FnA::arr(|(a, d)| (a + d, d)));
assert_eq!(traced(7), 7);
```


## ストリング図式 DSL

`Diagram` は以下のノード種を持つ実行時ストリング図式表現です:

- `Identity` — アリティ n の恒等ワイヤー
- `Box { label }` — ラベル付き射
- `Sequence(a, b)` — 縦合成 (`a.then(b)`)
- `Parallel(a, b)` — 横合成 (`a.parallel(b)`)
- `Swap { left, right }` — 組み紐ノード
- `Cup { arity }` — コンパクト閉の単位 (I → A\* ⊗ A)
- `Cap { arity }` — コンパクト閉の余単位 (A ⊗ A\* → I)

``` rust
use karpal_diagram::Diagram;

let circuit = Diagram::box_("f", 1, 1)
    .parallel(Diagram::box_("g", 1, 1))
    .then(Diagram::swap(1, 1))
    .then(Diagram::box_("h", 2, 2));

// テスト描画
println!("{}", circuit.render_text());

// SVG 描画
println!("{}", circuit.render_svg());
```


## 図式の正規化

図式は以下の書き換え規則を用いて標準形に正規化されます:

| 規則                         | 効果                                  |
|------------------------------|-----------------------------------------|
| `FlattenSequence`            | 入れ子の `Sequence` ノードを平坦化         |
| `FlattenParallel`            | 入れ子の `Parallel` ノードを平坦化         |
| `ElideIdentitySequenceStage` | 逐次における恒等を削除             |
| `CollapseIdentityParallel`   | すべて恒等の並列分岐を潰す |
| `CancelAdjacentSwaps`        | swap(A,B) ; swap(B,A) → id              |
| `YankCupCap`                 | (cup ⊗ id) ; (id ⊗ cap) → id            |

``` rust
let yanked = Diagram::cup(1)
    .parallel(Diagram::identity(1))
    .then(Diagram::identity(1).parallel(Diagram::cap(1)));

let trace = yanked.normalize_with_trace();
assert_eq!(trace.normalized, Diagram::identity(1));
assert!(trace.applied(NormalizationRule::YankCupCap));

// 正規化による同値性チェック
let a = Diagram::swap(1, 2).then(Diagram::swap(2, 1));
assert!(a.equivalent_to(&Diagram::identity(3)));
```


## 型レベルのコヒーレンス証拠

モノイダルコヒーレンス法則は `karpal-proof::Justifies` 証拠としてエンコードされます:

| 証拠            | 法則                                   |
|--------------------|---------------------------------------|
| `PentagonIdentity` | (α⊗id) ; α ; (id⊗α) = α ; α           |
| `TriangleIdentity` | ρ⊗id = α ; (id⊗λ)                     |
| `HexagonIdentity`  | braid ; α⁻¹ ; braid ; α⁻¹ = α ; braid |

``` rust
use karpal_diagram::coherence::verify_hexagon;
use karpal_proof::rewrite::Rewrite;

let _proof: Rewrite<((i32, u8), bool), ((u8, bool), i32), _> =
    verify_hexagon::();
```


## 図式的書き換えブリッジ

実行時の図式正規化は `ByNormalization` と `ByYanking` を介して型レベルの証明に接続します:

``` rust
use karpal_diagram::coherence::{equivalent_proved, prove_yanking, ByYanking};
use karpal_proof::rewrite::Rewrite;

// 正規化で同値性を証明
let a = Diagram::swap(1, 2).then(Diagram::swap(2, 1));
let witness: Rewrite<_, _, _> =
    equivalent_proved::<(), ()>(&a, &Diagram::identity(3)).unwrap();

// ヤンキングを証明
let yank_proof: Rewrite<_, _, ByYanking> = prove_yanking::<(), ()>(2);
```


## 検証統合

コヒーレンス証明書は `karpal-verify` に接続します:

``` rust
use karpal_diagram::coherence::coherence_certificates;

let certs = coherence_certificates();
assert_eq!(certs.len(), 3); // pentagon、triangle、hexagon
for cert in &certs {
    assert_eq!(cert.backend, "karpal-diagram-coherence");
}
```
