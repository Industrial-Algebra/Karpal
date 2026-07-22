# 半群とモノイド

値の結合のための代数的型クラス。

Semigroup と Monoid は Karpal の基礎的な代数的抽象化です。`Semigroup` は同じ型の二つの値を結合する結合的 二項演算を提供します。`Monoid` は `Semigroup` を単位元で拡張し、空のコレクションをデフォルト値に畳み込むような操作を可能にします。


### Semigroup

結合的 二項演算を持つ型。


#### シグネチャ

``` rust
/// 結合的二項演算を持つ型。
pub trait Semigroup {
    fn combine(self, other: Self) -> Self;
}
```

`combine` メソッドは両方の値の所有権を取り、同じ型の新しい値を生成します。`self` を消費するため、隠れたエイリアスはなく — 実装はアロケーションを再利用できます (Karpal の `String` と `Vec` の実装はまさにそうします)。

#### 法則


結合律

型 `T: Semigroup` のすべての `a`、`b`、`c` について:

``` rust
a.combine(b).combine(c) == a.combine(b.combine(c))
```

演算のグループ化は問いません。これが `Semigroup` が満たすべき唯一の法則です。


#### 実装

| 型                              | `combine` の振る舞い                                                     | フィーチャーゲート     |
|-----------------------------------|---------------------------------------------------------------------------|------------------|
| `i8`, `i16`, `i32`, `i64`, `i128` | 加算 (`self + other`)                                                 | なし (`no_std`)  |
| `u8`, `u16`, `u32`, `u64`, `u128` | 加算 (`self + other`)                                                 | なし (`no_std`)  |
| `f32`, `f64`                      | 加算 (`self + other`)                                                 | なし (`no_std`)  |
| `String`                          | 結合 (`push_str`)                                                | `std` または `alloc` |
| `Vec<T>`                          | 結合 (`extend`)                                                  | `std` または `alloc` |
| `Option<T: Semigroup>`            | 両方が `Some` なら内側の値を結合; そうでなければ `Some` の側を保持 | なし (`no_std`)  |
| `NonEmptyVec<T>`                  | 結合 (先頭 + 末尾をマージ)                                       | `std` または `alloc` |

#### 例

``` rust
use karpal_core::semigroup::Semigroup;

// 数値の加算
assert_eq!(3i32.combine(4), 7);

// 文字列の結合
assert_eq!(
    "hello ".to_string().combine("world".to_string()),
    "hello world"
);

// Vec の結合
assert_eq!(vec![1, 2].combine(vec![3, 4]), vec![1, 2, 3, 4]);

// Option は内側の Semigroup を持ち上げる
assert_eq!(Some(3i32).combine(Some(4)), Some(7));
assert_eq!(Some(3i32).combine(None), Some(3));
assert_eq!(None::<i32>.combine(Some(4)), Some(4));
```


### Monoid

単位元を持つ Semigroup。


#### シグネチャ

``` rust
use crate::semigroup::Semigroup;

/// 単位元を持つ `Semigroup`。
pub trait Monoid: Semigroup {
    fn empty() -> Self;
}
```

`empty` メソッドはその型の `combine` 演算の単位元を返します。任意の値と `empty()` を (どちら側で) 結合しても、その値が変更なく返されなければなりません。

#### 法則


左単位律

型 `T: Monoid` のすべての `a` について:

``` rust
T::empty().combine(a) == a
```


右単位律

型 `T: Monoid` のすべての `a` について:

``` rust
a.combine(T::empty()) == a
```


`Semigroup` の結合律と合わせて、これら二つの法則により `(T, combine, empty)` は代数的な意味でモノイドになります。

#### 実装

| 型                              | `empty()` の値                | フィーチャーゲート     |
|-----------------------------------|--------------------------------|------------------|
| `i8`, `i16`, `i32`, `i64`, `i128` | `0`                            | なし (`no_std`)  |
| `u8`, `u16`, `u32`, `u64`, `u128` | `0`                            | なし (`no_std`)  |
| `f32`, `f64`                      | `0.0`                          | なし (`no_std`)  |
| `String`                          | `String::new()` (空文字列) | `std` または `alloc` |
| `Vec<T>`                          | `Vec::new()` (空の vec)       | `std` または `alloc` |
| `Option<T: Semigroup>`            | `None`                         | なし (`no_std`)  |

`NonEmptyVec<T>` は `Semigroup` を実装しますが **モノイドではありません** — 定義上常に少なくとも一つの要素を含むため、有効な単位元が存在しません。

#### 例

``` rust
use karpal_core::semigroup::Semigroup;
use karpal_core::monoid::Monoid;

// 数値の単位元
assert_eq!(i32::empty(), 0);
assert_eq!(i32::empty().combine(42), 42);
assert_eq!(42i32.combine(i32::empty()), 42);

// 文字列の単位元
assert_eq!(String::empty(), "");

// Vec の単位元
assert_eq!(Vec::<i32>::empty(), Vec::<i32>::new());

// Option の単位元
assert_eq!(Option::<i32>::empty(), None);
```


## Foldable と Monoid

`Monoid` トレイトは [Foldable](foldable-traversable.md) 型クラスで中心的な役割を果たします。`Foldable` は `fold_map` を定義します。これは構造の各要素を `Monoid` を返す関数でマップし、すべての結果を `combine` と `empty` を使って結合します:

``` rust
pub trait Foldable: HKT {
    fn fold_right<A, B>(fa: Self::Of<A>, init: B, f: impl Fn(A, B) -> B) -> B;

    fn fold_map<A, M: Monoid>(fa: Self::Of<A>, f: impl Fn(A) -> M) -> M {
        Self::fold_right(fa, M::empty(), |a, acc| f(a).combine(acc))
    }
}
```

`fold_map` のデフォルト実装は `M::empty()` を初期アキュムレータとして右畳み込みし、各マップされた要素をアキュムレータと結合します。`Monoid` は結合律と単位律を保証するため、畳み込み方向に関わらず結果は well-defined です。

#### 例: コレクションの合計

``` rust
use karpal_core::prelude::*;

// 恒等関数での fold_map は要素を合計する。
// なぜなら i32 の Semigroup 実装は加算を使うから。
let total = VecF::fold_map(vec![1, 2, 3], |a: i32| a);
assert_eq!(total, 6);
```

#### 例: 文字列の収集

``` rust
use karpal_core::prelude::*;

// 各数値を文字列表現にマップし、結合する。
// String の Semigroup は結合し、その Monoid は "" から始まる。
let result = VecF::fold_map(vec![1, 2, 3], |a: i32| a.to_string());
assert_eq!(result, "123".to_string());
```

このパターン — マップして結合 — が `fold_map` の本質であり、関数型プログラミングで `Monoid` がそれほど重要な理由です。コレクションを単一の要約値に還元したいときはいつでも、`Monoid` が汎用的に行うための構造を提供します。


Karpal は Apache-2.0 + CLA でライセンスされています。[GitHub で見る](https://github.com/Industrial-Algebra/Karpal)。
