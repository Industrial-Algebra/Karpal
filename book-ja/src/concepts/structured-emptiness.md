# 構造化された空 (Structured Emptiness)

## 問題

標準ライブラリは空を単一の概念として扱います: `None`、`Err`、`0`、`empty()`。空である *理由* は失われます。

幾何学的計算 — および他の多くの領域 — では、根本的に異なる *種類* の空が存在します:

| 種類 | 意味 | 例 |
|------|------|------|
| 構造的ゼロ | 問いを立てることすらできない | codim > dim |
| 幾何学的ゼロ | 問いは妥当だが解がない | LR 係数 = 0 |
| 正 | n 個の解が存在する | LR 係数 = n |
| 未決定 | 無限に多くの解が存在する | codim < dim |

## 束 Ω

Karpal はブール真理値をより豊かな束で置き換えます:

```
Denied < Granted(0) < Granted(1) < ... < Granted(∞)
```

これは **ハイティング代数** です — 含意を持つ有界束であり、排中律が成り立ちません (一般に `¬¬a ≠ a`)。

## 実装

具体的な実現はグラスマン多様体上のシューベルト計算によるものです。Gr(k, n) 内の二つのシューベルト類 σ_λ と σ_μ について:

- それらの交点積はリトルウッド・リチャードソン係数によって計算されます
- 結果は `StructuralZero`、`GeometricZero`、`Positive`、または `Underdetermined` に分類されます
- 交点の合成は束の meet (最悪の場合の伝播) を用います

```rust
use karpal_schubert_types::{check_intersection, IntersectionKind, SchubertType};

let s1 = SchubertType::new(vec![1], (2, 4)).unwrap();
let s22 = SchubertType::new(vec![2, 2], (2, 4)).unwrap();

assert_eq!(check_intersection(&s1, &s1).kind(), IntersectionKind::Positive);
assert_eq!(check_intersection(&s22, &s22).kind(), IntersectionKind::StructuralZero);
```

## より深い主張

> 計算が結果を返さない理由は、結果そのものと同じくらい重要である。ゼロは単一の値ではなく — 値の空間であり、その空間の幾何学が情報を持つ。

完全な数学的取り扱いについては [設計文書](https://github.com/Industrial-Algebra/Karpal/blob/develop/docs/design/structured-emptiness.md) を参照してください。
