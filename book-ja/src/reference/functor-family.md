# 関手ファミリー

共変関手階層: Functor から Monad まで。

## 階層

Functor ファミリーは次第に強力な抽象化の線形連鎖を形成します。各トレイトは上位のものを拡張します:

``` rust
Functor           // fmap: A -> B を F<A> -> F<B> に持ち上げる
  |
  v
Apply             // ap: F<A -> B> を F<A> に適用し、F<B> を生成
  |
  v
Applicative       // pure: 値 A を F<A> に持ち上げる
  |
  +--- Chain      // chain: モナド束縛 (flatMap)
  |      |
  v      v
  Monad           // Applicative + Chain (ブランケット実装、追加メソッドなし)
```

`Monad` はブランケット実装として提供されます: `Applicative` と `Chain` の両方を実装する任意の型は自動的に `Monad` を実装します。明示的な `impl Monad for ...` ブロックを書く必要はありません。


### Functor

共変関手: 関数 `A -> B` を `F<A> -> F<B>` に持ち上げます。


#### シグネチャ

``` rust
pub trait Functor: HKT {
    fn fmap<A, B>(fa: Self::Of<A>, f: impl Fn(A) -> B) -> Self::Of<B>;
}
```

#### 法則


**単位律:** 恒等関数でマップしても何も変わりません。  
`F::fmap(fa, |x| x) == fa`


**合成律:** 二つの関数を順にマップするのは、それらの合成をマップするのと同じです。  
`F::fmap(F::fmap(fa, f), g) == F::fmap(fa, |x| g(f(x)))`


#### 実装

| 型コンストラクタ | 備考                                              |
|------------------|----------------------------------------------------|
| `OptionF`        | `Option::map` に委譲                         |
| `ResultF<E>`     | `Result::map` に委譲                         |
| `VecF`           | `alloc` または `std` フィーチャーが必要                  |
| `IdentityF`      | `f` を直接適用: `f(fa)`                      |
| `NonEmptyVecF`   | `alloc` または `std` フィーチャーが必要                  |
| `EnvF<E>`        | タプル `(E, A)` の第二要素でマップ |

#### 例

``` rust
use karpal_std::prelude::*;

// 具体的な使用
let doubled = OptionF::fmap(Some(5), |x| x * 2);
assert_eq!(doubled, Some(10));

let lengths = VecF::fmap(vec!["hello", "world"], |s| s.len());
assert_eq!(lengths, vec![5, 5]);

// 任意の Functor について汎用的
fn increment<F: Functor>(fa: F::Of<i32>) -> F::Of<i32> {
    F::fmap(fa, |x| x + 1)
}

assert_eq!(increment::<OptionF>(Some(9)), Some(10));
assert_eq!(increment::<VecF>(vec![1, 2]), vec![2, 3]);
```


### Apply

包まれた関数を包まれた値に適用できる Functor。


#### シグネチャ

``` rust
pub trait Apply: Functor {
    fn ap<A, B, F>(ff: Self::Of<F>, fa: Self::Of<A>) -> Self::Of<B>
    where
        A: Clone,
        F: Fn(A) -> B;
}
```

一部の実装 (`VecF` など) は各値に複数の関数を適用し、値を複数回消費するため、`A: Clone` 境界が必要です。

#### 法則


**結合的合成:** 合成された関数を適用するのは、適用を合成するのと同じです。  
`ap(ap(fmap(compose, f), g), x) == ap(f, ap(g, x))`


#### 実装

| 型コンストラクタ | 備考                                                  |
|------------------|--------------------------------------------------------|
| `OptionF`        | 両方が `Some` なら関数を適用; さもなくば `None`  |
| `ResultF<E>`     | 両方が `Ok` なら関数を適用; 最初の `Err` が勝つ    |
| `VecF`           | 直積: 各関数を各値に適用 |
| `IdentityF`      | 直接適用: `ff(fa)`                           |
| `NonEmptyVecF`   | 直積 (`alloc` または `std` が必要)          |

#### 例

``` rust
use karpal_std::prelude::*;

// 包まれた関数を包まれた値に適用
let f: Option<fn(i32) -> i32> = Some(|x| x * 2);
let result = OptionF::ap(f, Some(21));
assert_eq!(result, Some(42));

// Vec: 関数と値の直積
let fs: Vec<fn(i32) -> i32> = vec![|x| x + 1, |x| x * 10];
let result = VecF::ap(fs, vec![1, 2, 3]);
assert_eq!(result, vec![2, 3, 4, 10, 20, 30]);
```


### Applicative

純粋な値を関手に持ち上げられる Apply。


#### シグネチャ

``` rust
pub trait Applicative: Apply {
    fn pure<A>(a: A) -> Self::Of<A>;
}
```

#### 法則


**単位律:** 純粋な恒等関数を適用しても何も変わりません。  
`ap(pure(id), v) == v`


**準同型律:** 関数と値を持ち上げて適用するのは、結果を直接持ち上げるのと同じです。  
`ap(pure(f), pure(x)) == pure(f(x))`


**交換律:** 値が純粋な場合、持ち上げの順序は問いません。  
`ap(u, pure(y)) == ap(pure(|f| f(y)), u)`


#### 実装

| 型コンストラクタ | `pure(a)` が返す           |
|------------------|-----------------------------|
| `OptionF`        | `Some(a)`                   |
| `ResultF<E>`     | `Ok(a)`                     |
| `VecF`           | `vec![a]`                   |
| `IdentityF`      | `a`                         |
| `NonEmptyVecF`   | `NonEmptyVec::singleton(a)` |

#### 例

``` rust
use karpal_std::prelude::*;

// 値を任意の Applicative 文脈に持ち上げる
let opt: Option<i32> = OptionF::pure(42);
assert_eq!(opt, Some(42));

let v: Vec<i32> = VecF::pure(42);
assert_eq!(v, vec![42]);

// 汎用的な持ち上げ
fn wrap<F: Applicative>(x: i32) -> F::Of<i32> {
    F::pure(x)
}

assert_eq!(wrap::<OptionF>(7), Some(7));
assert_eq!(wrap::<VecF>(7), vec![7]);
```


### Chain

モナド束縛 (flatMap) を持つ Apply。各ステップが前の結果に依存する逐次計算を可能にします。


#### シグネチャ

``` rust
pub trait Chain: Apply {
    fn chain<A, B>(fa: Self::Of<A>, f: impl Fn(A) -> Self::Of<B>) -> Self::Of<B>;
}
```

関数 `f` は単なる `B` ではなく `Self::Of<B>` を返すことに注意してください。これが `chain` を `fmap` と区別します: コールバック自体が包まれた値を生成し、`chain` が結果を平坦化します。

#### 法則


**結合律:** 束縛は結合的です — 入れ子にしても同じです。  
`chain(chain(m, f), g) == chain(m, |x| chain(f(x), g))`


#### 実装

| 型コンストラクタ | 備考                                                             |
|------------------|-------------------------------------------------------------------|
| `OptionF`        | `Option::and_then` に委譲                                   |
| `ResultF<E>`     | `Result::and_then` に委譲                                   |
| `VecF`           | `flat_map`: 各要素が Vec を生成し、結果は結合 |
| `IdentityF`      | 直接適用: `f(fa)`                                       |
| `NonEmptyVecF`   | 空でない結果を結合 (`alloc` または `std` が必要)        |

#### 例

``` rust
use karpal_std::prelude::*;

// Option: None で短絡
fn safe_sqrt(x: f64) -> Option<f64> {
    if x >= 0.0 { Some(x.sqrt()) } else { None }
}

let result = OptionF::chain(Some(16.0), safe_sqrt);
assert_eq!(result, Some(4.0));

let result = OptionF::chain(Some(-1.0), safe_sqrt);
assert_eq!(result, None);

// Vec: flatMap (各要素がリストに展開)
let result = VecF::chain(vec![1, 2, 3], |x| vec![x, x * 10]);
assert_eq!(result, vec![1, 10, 2, 20, 3, 30]);
```


### Monad

Applicative + Chain。追加メソッドのないブランケット実装。


#### シグネチャ

``` rust
pub trait Monad: Applicative + Chain {}

impl<F: Applicative + Chain> Monad for F {}
```

`Monad` は **マーカートレイト** です。新しいメソッドを追加しません; 単に型が `Applicative` (`pure` 用) と `Chain` (`chain` 用) の両方を実装することを証明します。ブランケット `impl` により、`impl Monad for MyType` を書くことはありません — `Applicative` と `Chain` を実装すれば `Monad` は自動的に付いてきます。

#### 法則

Applicative と Chain の法則に加え、Monad は以下を満たさなければなりません:


**左単位律:** `pure` で値を持ち上げて束縛するのは、関数を直接呼ぶのと同じです。  
`chain(pure(a), f) == f(a)`


**右単位律:** `pure` で束縛しても何も変わりません。  
`chain(m, pure) == m`


#### 実装

`Applicative` と `Chain` の両方を実装するすべての型が自動的に `Monad` になります:

| 型コンストラクタ | 備考                                    |
|------------------|------------------------------------------|
| `OptionF`        | ブランケット実装                             |
| `ResultF<E>`     | ブランケット実装                             |
| `VecF`           | ブランケット実装 (`alloc` または `std` が必要) |
| `IdentityF`      | ブランケット実装                             |
| `NonEmptyVecF`   | ブランケット実装 (`alloc` または `std` が必要) |

#### 例

``` rust
use karpal_std::prelude::*;

// トレイト境界として Monad を使い、pure と chain の両方を要求
fn bind_and_wrap<M: Monad>(x: i32) -> M::Of<String>
where
    M::Of<i32>: Clone,
{
    M::chain(M::pure(x), |n| M::pure(format!("value: {}", n)))
}

assert_eq!(bind_and_wrap::<OptionF>(42), Some("value: 42".to_string()));

// do_! マクロは chain 呼び出しに脱糖されるため、Monad が必要
let result = do_! { OptionF;
    x = Some(10);
    y = Some(x + 20);
    Some(x + y)
};
assert_eq!(result, Some(40));
```


## 関連項目

- [**Alt ファミリー**](alt-family.md) -- 選択と失敗という異なる方向に Functor を拡張する Alt / Plus / Alternative 枝。
- [**マクロ**](macros.md) -- Chain と Applicative の計算に快適な構文を提供する `do_!` と `ado_!` マクロ。
- [**Foldable と Traversable**](foldable-traversable.md) -- 構造の畳み込みと走査。Applicative と自然に組み合わさります。
- [**はじめての利用**](../getting-started.md) -- HKT、Functor、モナド記法のチュートリアル入門。


Karpal は Apache-2.0 + CLA でライセンスされています。[GitHub で見る](https://github.com/Industrial-Algebra/Karpal)。
