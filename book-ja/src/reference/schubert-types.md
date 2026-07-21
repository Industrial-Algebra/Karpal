# シューベルト型

シューベルト交点型システム — `karpal-schubert-types` (フェーズ 14 A–C)。


## 概要

型はグラスマン多様体 Gr(k, n) 内のシューベルト類 σ<sub>λ</sub> であり、型の互換性はリトルウッド・リチャードソン交点係数によって計算されます。二つの型は、それらのシューベルト類が非自明に交わる (σ<sub>A</sub> · σ<sub>B</sub> ≠ 0) ときに互換です。LR 係数は *多重度* — 個別の強制経路の数 — を与えます。


## SchubertType

グラスマン多様体内の分割 (ヤング図形) で添字付けられたシューベルト類:

``` rust
use karpal_schubert_types::SchubertType;

// Gr(2,4) 内の σ₁ — 固定された 2-平面と交わる直線
let sigma_1 = SchubertType::new(vec![1], (2, 4)).expect("valid");

// Gr(2,4) 内の σ₂₂ — 点類
let sigma_22 = SchubertType::new(vec![2, 2], (2, 4)).expect("valid");

// 分割の要素がボックス上限を超える → エラー
assert!(SchubertType::new(vec![3], (2, 4)).is_err());

assert_eq!(sigma_1.codimension(), 1); // 分割要素の和
assert_eq!(sigma_22.codimension(), 4);
```


## Intersection

`check_intersection(a, b)` は `amari-enumerative` を介して交点積を計算し、結果を分類します:

| 種類              | 意味                                          |
|-------------------|----------------------------------------------|
| `StructuralZero`  | 全余次元がグラスマン多様体の次元を超える |
| `GeometricZero`   | 次元は正しいが交点がない |
| `Positive`        | 既知の多重度を持つ空でない交点    |
| `Underdetermined` | 計算が結果を解決できなかった         |

``` rust
use karpal_schubert_types::{check_intersection, IntersectionKind, SchubertType};

let s1 = SchubertType::new(vec![1], (2, 4)).unwrap();
let s22 = SchubertType::new(vec![2, 2], (2, 4)).unwrap();

// σ₁ · σ₁ は正の次元を持つ
let result = check_intersection(&s1, &s1);
assert_eq!(result.kind(), IntersectionKind::Positive);

// σ₂₂ · σ₂₂ は構造的ゼロ (余次元 8 > 次元 4)
let zero = check_intersection(&s22, &s22);
assert_eq!(zero.kind(), IntersectionKind::StructuralZero);
assert_eq!(zero.multiplicity(), 0);
```


## SchubertTyped & SchubertProven

`SchubertTyped` はシューベルト類を Rust の型に関連付けます。`SchubertProven<M, T>` は `karpal_proof::Proven<P, T>` のシューベルト版です:

``` rust
use karpal_schubert_types::{SchubertProven, SchubertType, SchubertTyped};

// マーカー型を宣言
struct Sigma1;

impl SchubertTyped for Sigma1 {
    fn schubert_type() -> SchubertType {
        SchubertType::new(vec![1], (2, 4)).expect("σ₁")
    }
}

// 値を型レベルの証明で包む
let proven = SchubertProven::::new("my_data");
assert_eq!(*proven.value(), "my_data");

// 別の型との互換性を確認
assert!(proven.check_against::().is_some());

// 取り出す
assert_eq!(proven.into_inner(), "my_data");
```


## 連鎖合成

`compose_checks::<A, B, C>()` は LR 規則を介して型互換性の連鎖を検証します:

``` rust
use karpal_schubert_types::compose_checks;

// A → B → C 合成連鎖を検証
let chain = compose_checks::();
assert!(chain.is_some());
```


## 外部検証

シューベルト計算の性質は `karpal-verify` のオブリゲーションバンドルとしてエクスポートされます:

``` rust
use karpal_schubert_types::verification::verify_schubert;

let report = verify_schubert();
assert_eq!(report.obligations.len(), 3);

for obl in &report.obligations {
    assert!(obl.certificate.is_some());
}
```
