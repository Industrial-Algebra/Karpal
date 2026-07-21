# 抽象代数

[半群とモノイド](semigroup-monoid.md) の上に構築されたより高次の代数構造。これらのトレイトは `karpal-algebra` クレートにあり、群・環・体・束・ベクトル空間で階層を拡張します。

## 概要

| トレイト            | 拡張        | 主要なアイデア                                                         |
|------------------|----------------|------------------------------------------------------------------|
| `Group`          | `Monoid`       | すべての要素が逆元を持つ: `a.combine(a.invert()) == empty()` |
| `AbelianGroup`   | `Group`        | マーカー — 演算が可換                                |
| `Semiring`       | (独立)  | 分配と吸収を持つ二つの演算 (加算/乗算)      |
| `Ring`           | `Semiring`     | 加法逆元: `a.add(a.negate()) == zero()`                 |
| `Field`          | `Ring`         | 非零要素の乗法逆元                    |
| `Lattice`        | (独立)  | 吸収律を持つ join (上限) と meet (下限)               |
| `BoundedLattice` | `Lattice`      | top と bottom 要素                                          |
| `HeytingAlgebra` | `BoundedLattice` | 含意を持つ有界束 (直観主義論理) |
| `Module<R>`      | `AbelianGroup` | 環上のスカラー倍                                        |
| `VectorSpace<F>` | `Module<F>`    | 体上のモジュール                                              |

## トレイト階層

``` text
Semigroup (karpal-core)         Semiring (独立)      Lattice (独立)
  |                               |                           |
Monoid (karpal-core)            Ring                        BoundedLattice
  |                               |                           |
Group                           Field                       HeytingAlgebra
  |
AbelianGroup (マーカー)           Module<R: Ring>: AbelianGroup
                                  |
                                VectorSpace<F: Field>
```

`Semigroup → Monoid → Group` の連鎖は既存の `karpal-core` 階層を拡張します。`Semiring` と `Lattice` は独立した階層です — 「どの演算が半群か?」という曖昧さを避けるため、独自の演算を定義します。

## ニュータイプラッパー

数値型のデフォルト `Semigroup` は加算を使います。`karpal-core` のニュータイプラッパーにより、異なる結合戦略を選択できます。これは「一つの型が複数の方法でモノイドになりうる」という問題への標準的なアプローチです。


### Sum と Product

加法的または乗法的な結合を選択します。`Sum<T>` は `+` を、`Product<T>` は `*` を使います。`empty()` はそれぞれ `0` と `1` です。


### Min と Max

順序付き型の最小または最大による結合を選択します。`Min<T>` は `min` を、`Max<T>` は `max` を使います。`empty()` はそれぞれ `T::MAX` と `T::MIN` です。


### First と Last

最初または最後の `Some` 値を選択します。`Option<T>` についてのみ実装され、Haskell の `Data.Monoid.First`/`Last` に一致します。


## 群階層


### Group

すべての要素が逆元を持つ Monoid。

`invert` は要素の逆元を返します。法則: `a.combine(a.invert()) == empty()`。提供される `combine_inverse` は `a.combine(b.invert())` を計算します。


### AbelianGroup

演算が可換な Group のマーカートレイト。新しいメソッドは追加しません — 可換性は (プロパティベースのテストで検証される) 法則要件です。


## 半環・環・体


### Semiring

二つの演算 (加算 `add` と乗算 `mul`) を持ち、分配律と吸収律を満たします。加算は可換モノイド、乗算はモノイドです。

法則: 分配律 `a.mul(b.add(c)) == a.mul(b).add(a.mul(c))`、乗法吸収 `a.mul(zero()) == zero()`。


### Ring

加法逆元を持つ Semiring。`negate` が各要素の加法逆元を与え、`a.add(a.negate()) == zero()` が成り立ちます。加算は AbelianGroup を形成します。


### Field

非零要素が乗法逆元を持つ Ring。`reciprocal` が非零要素の乗法逆元を与え、`a.mul(a.reciprocal()) == one()` (a ≠ zero) が成り立ちます。乗算は (zero を除いて) AbelianGroup を形成します。


## 束階層


### Lattice

join (上限・supremum) と meet (下限・infimum) を持つ構造で、吸収律を満たします。半順序集合で、任意の二要素が上限と下限を持ちます。

法則: 吸収律 `a.join(a.meet(b)) == a`、`a.meet(a.join(b)) == a`。


### BoundedLattice

top 要素と bottom 要素を持つ Lattice。`top()` は最大元、`bottom()` は最小元です。


### HeytingAlgebra

含意 `implies` と否定 `neg` を持つ BoundedLattice。直観主義論理の内部論理で、排中律が一般に成り立ちません (`¬¬a ≠ a`)。これは構造化された空 (structured emptiness) の基礎です — [構造化された空の概念](../concepts/structured-emptiness.md) を参照してください。


## モジュールとベクトル空間


### Module\<R: Ring\>

環 `R` 上のスカラー倍を持つ AbelianGroup。`scale(r, a)` が環元 `r` で群要素 `a` をスケールし、分配律と結合律を満たします。


### VectorSpace\<F: Field\>

体 `F` 上の Module。体は環なので、VectorSpace は Module の特殊化です。スカラー倍は体の乗法逆元の存在によりより豊かな構造を持ちます。


Karpal は Apache-2.0 + CLA でライセンスされています。[GitHub で見る](https://github.com/Industrial-Algebra/Karpal)。
