# Alt ファミリー

フォールバックと選択コンビネータ: Alt、Plus、Alternative。

## 階層


Functor  →  Alt  →  Plus  →  (+ Applicative) →  Alternative  (ブランケット)


関手階層の Alt 枝はフォールバックと選択を表現するためのコンビネータを提供します。`Alt` は結合的な選択演算を与え、`Plus` は単位元 (zero/empty) を追加し、`Alternative` はブランケット実装を介して `Plus` と `Applicative` を組み合わせます。

## Alt


### Alt


結合的な選択演算を持つ Functor。


#### シグネチャ

``` rust
pub trait Alt: Functor {
    fn alt<A>(fa1: Self::Of<A>, fa2: Self::Of<A>) -> Self::Of<A>;
}
```

`alt` は同じ関手型の二つの値を取り、一つを返します。両方が「成功」のときは最初のものを優先します。正確な意味論は実装に依存します: `OptionF` では `.or()`、`VecF` では結合です。

#### 法則


結合律


alt(alt(a, b), c) == alt(a, alt(b, c))


分配律


fmap(f, alt(a, b)) == alt(fmap(f, a), fmap(f, b))


#### 実装

| 型コンストラクタ | `alt` の振る舞い                                                                |
|------------------|-----------------------------------------------------------------------------------|
| `OptionF`        | `fa1.or(fa2)` — 最初の `Some` を返す、両方が `None` なら `None`            |
| `ResultF<E>`     | `fa1.or(fa2)` — 最初の `Ok` を返す、最初が `Err` なら二番目の値を返す |
| `VecF`           | 結合 — `fa1` に `fa2` の全要素を追加                          |
| `NonEmptyVecF`   | 結合 — `fa1` に `fa2` の先頭と末尾を追加                     |

`VecF` と `NonEmptyVecF` は `alloc` または `std` フィーチャーが必要です。

#### 例

``` rust
use karpal_std::prelude::*;

// フォールバック: 最初のソースを試し、二番目にフォールバック
let primary: Option<i32> = None;
let fallback: Option<i32> = Some(42);

let result = OptionF::alt(primary, fallback);
assert_eq!(result, Some(42));

// 両方が存在する場合、最初が勝つ
let result = OptionF::alt(Some(1), Some(2));
assert_eq!(result, Some(1));

// Vec: 結合
let combined = VecF::alt(vec![1, 2], vec![3, 4]);
assert_eq!(combined, vec![1, 2, 3, 4]);
```


## Plus


### Plus


zero/empty 要素を持つ Alt。


#### シグネチャ

``` rust
pub trait Plus: Alt {
    fn zero<A>() -> Self::Of<A>;
}
```

`zero` は `alt` の単位元を生成します。Alt の法則と合わせて、これが関手型上のモノイド構造を与えます。

#### 法則


左単位律


alt(zero(), a) == a


右単位律


alt(a, zero()) == a


消滅律


fmap(f, zero()) == zero()


#### 実装

| 型コンストラクタ | `zero()` が返す            |
|------------------|-----------------------------|
| `OptionF`        | `None`                      |
| `VecF`           | `Vec::new()` (空のベクタ) |

`ResultF<E>` は `E` 値なしに `Result<A, E>` を生成できないため **`Plus` を実装しません**。`NonEmptyVecF` も、空でないベクタは定義上空になり得ないため実装を持ちません。

`VecF` は `alloc` または `std` フィーチャーが必要です。

#### 例

``` rust
use karpal_std::prelude::*;

// Option の zero() は None
let empty: Option<i32> = OptionF::zero();
assert_eq!(empty, None);

// Vec の zero() は空のベクタ
let empty_vec: Vec<i32> = VecF::zero();
assert_eq!(empty_vec, Vec::<i32>::new());

// 左単位律: alt(zero(), a) == a
let a = Some(10);
assert_eq!(OptionF::alt(OptionF::zero(), a), a);

// 右単位律: alt(a, zero()) == a
assert_eq!(OptionF::alt(a, OptionF::zero()), a);
```


## Alternative


### Alternative


追加メソッドなしの Applicative + Plus (ブランケット実装)。


#### シグネチャ

``` rust
pub trait Alternative: Applicative + Plus {}

impl<F: Applicative + Plus> Alternative for F {}
```

`Alternative` は `Applicative` と `Plus` を組み合わせるマーカートレイトです。新しいメソッドを導入しません — `Applicative` と `Plus` の両方を実装する任意の型はブランケット実装経由で自動的に `Alternative` を実装します。

#### 法則

Alternative は Alt、Plus、Applicative のすべての法則を継承し、二つを追加します:


分配律


ap(alt(f, g), x) == alt(ap(f, x), ap(g, x))


消滅律


ap(zero(), x) == zero()


#### 実装

| 型コンストラクタ | 備考                                                                                                            |
|------------------|------------------------------------------------------------------------------------------------------------------|
| `OptionF`        | `Applicative` と `Plus` の両方を実装するため、`Alternative` は自動的に提供                             |
| `VecF`           | `Applicative` と `Plus` の両方を実装するため、`Alternative` は自動的に提供 (`alloc` または `std` が必要) |

#### 例

``` rust
use karpal_std::prelude::*;

// Alternative は選択 (Alt/Plus) とアプリカティブ計算 (Applicative) を組み合わせる。

// 分配律: ap(alt(f, g), x) == alt(ap(f, x), ap(g, x))
let f: Option<fn(i32) -> i32> = Some(|a| a + 1);
let g: Option<fn(i32) -> i32> = Some(|a| a * 2);
let x = Some(10);

let left  = OptionF::ap(OptionF::alt(f, g), x);
let right = OptionF::alt(OptionF::ap(f, x), OptionF::ap(g, x));
assert_eq!(left, right);  // どちらも Some(11)

// 消滅律: ap(zero(), x) == zero()
let no_fn: Option<fn(i32) -> i32> = OptionF::zero();
let result = OptionF::ap(no_fn, Some(5));
assert_eq!(result, None);
```


## 関連項目

- [**Functor ファミリー**](functor-family.md) — Alt が構築される Functor → Apply → Applicative → Chain → Monad 枝。
- [**半群とモノイド**](semigroup-monoid.md) — 値レベルの類似物: Semigroup は結合的な `combine` を提供し、Monoid は `empty` 単位元を追加。関手レベルでの Alt/Plus の関係を反映。
- [**Foldable と Traversable**](foldable-traversable.md) — コンテナを潰し、逐次化するトレイト。Alt や Plus と自然に合成します。


Karpal は Apache-2.0 + CLA でライセンスされています。[GitHub で見る](https://github.com/Industrial-Algebra/Karpal)。
