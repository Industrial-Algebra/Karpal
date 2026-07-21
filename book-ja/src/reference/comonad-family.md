# コモナドファミリー

モナド階層の双対: コモナド的文脈と抽出。

Monad が文脈に値を *注入* し文脈を生成する計算を *逐次化* できるのに対し、Comonad は文脈から値を *抽出* し、文脈を消費する関数を構造全体に *拡張* できます。Karpal のコモナドファミリーは、線形階層に配置された五つのトレイトと三つの特殊化された枝で構成されます。

## 階層


Functor → Extend → Comonad → ComonadEnv  
Functor → Extend → Comonad → ComonadStore \*  
Functor → Extend → Comonad → ComonadTraced \*


**\* 設計メモ:** `ComonadStore` と `ComonadTraced` は (`Comonad` ではなく) `HKT` をスーパートレイトとして必要とします。これは `StoreF` と `TracedF` が内部で `Box<dyn Fn>` を使い、これが汎用 `Functor` シグネチャと互換性のない `'static` 境界を課すためです。`Functor` は `Extend` と `Comonad` のスーパートレイトであるため、これらの型は完全なコモナド連鎖を実装できません。代わりに、トレイト上に `peek`/`trace` を使ったデフォルトメソッドとして直接 `extract` メソッドを提供します。

## Extend


### Extend

Chain の双対。協調的で文脈認識的な計算を可能にします。


#### シグネチャ

``` rust
pub trait Extend: Functor {
    fn extend<A, B>(wa: Self::Of<A>, f: impl Fn(&Self::Of<A>) -> B) -> Self::Of<B>
    where
        A: Clone;

    fn duplicate<A>(wa: Self::Of<A>) -> Self::Of<Self::Of<A>>
    where
        A: Clone,
        Self::Of<A>: Clone;
}
```

文脈 `W<A>` 内の値と、完全な文脈を検査できる関数 `&W<A> -> B` を与えると、`extend` はその関数を構造のすべての「位置」で適用し、`W<B>` を生成します。`duplicate` メソッドにはデフォルト実装があります: `Self::extend(wa, |w| w.clone())`。

#### 法則


結合律


extend(f, extend(g, w)) == extend(\|w\| f(&extend(g, w.clone())), w)


#### 実装

| 型コンストラクタ | `Of<A>`          | 備考                                               |
|------------------|------------------|-----------------------------------------------------|
| `IdentityF`      | `A`              | 自明に値に `f` を適用                  |
| `OptionF`        | `Option<A>`      | `Some` なら `f` を適用; さもなくば `None`     |
| `NonEmptyVecF`   | `NonEmptyVec<A>` | 各接尾辞に `f` を適用 (alloc ゲート)            |
| `EnvF<E>`        | `(E, A)`         | ペアに `f` を適用し、環境を保持 |

#### 例

``` rust
use karpal_core::prelude::*;

// NonEmptyVec extend: 各接尾辞に要約関数を適用
let nev = NonEmptyVec::new(1, vec![2, 3]);
let sums = NonEmptyVecF::extend(nev, |w| w.iter().sum::<i32>());
// 接尾辞: [1,2,3], [2,3], [3]  =>  和: 6, 5, 3
assert_eq!(sums, NonEmptyVec::new(6, vec![5, 3]));

// Option extend
let doubled = OptionF::extend(Some(3), |opt| match opt {
    Some(x) => x * 2,
    None => 0,
});
assert_eq!(doubled, Some(6));

// duplicate: 構造をそれ自身の内側に埋め込む
let nested = OptionF::duplicate(Some(42));
assert_eq!(nested, Some(Some(42)));
```


## Comonad


### Comonad

Monad の圏論的双対。文脈から値を抽出します。


#### シグネチャ

``` rust
pub trait Comonad: Extend {
    fn extract<A: Clone>(wa: &Self::Of<A>) -> A;
}
```

Comonad は文脈から値を `extract` でき、文脈認識関数を構造全体に `extend` できます。`Monad::pure` が値を最小の文脈に注入するのに対し、`Comonad::extract` は既存の文脈から値を引き出します。

#### 法則


左単位律


extract(&extend(w, f)) == f(&w)


右単位律


extend(w, \|w\| extract(w)) == w


結合律は `Extend` から継承されます。

#### 実装

| 型コンストラクタ | `Of<A>`          | 備考                                                 |
|------------------|------------------|-------------------------------------------------------|
| `IdentityF`      | `A`              | 値を直接返す                            |
| `OptionF`        | `Option<A>`      | `None` でパニック (部分コモナド)                    |
| `NonEmptyVecF`   | `NonEmptyVec<A>` | 先頭要素を返す (alloc ゲート)                |
| `EnvF<E>`        | `(E, A)`         | `A` 成分を返し、環境を破棄 |

#### 例

``` rust
use karpal_core::prelude::*;

// NonEmptyVec から抽出: 常に先頭を返す
let nev = NonEmptyVec::new(10, vec![20, 30]);
assert_eq!(NonEmptyVecF::extract(&nev), 10);

// Env から抽出: 環境を破棄し、値を保持
assert_eq!(EnvF::<&str>::extract(&("config", 42)), 42);

// 左単位律の実例:
let f = |w: &NonEmptyVec<i32>| w.head + 1;
let extended = NonEmptyVecF::extend(nev.clone(), f);
assert_eq!(NonEmptyVecF::extract(&extended), f(&nev));
```


## ComonadEnv


### ComonadEnv\<E\>

環境値にアクセスできる Comonad。Reader/MonadReader の双対。


#### シグネチャ

``` rust
pub trait ComonadEnv<E>: Comonad {
    fn ask<A>(wa: &Self::Of<A>) -> E;
    fn local<A>(wa: Self::Of<A>, f: impl Fn(E) -> E) -> Self::Of<A>;
}
```

`ask` はコモナド値から環境を取得します。`local` は焦点の値を変更せずに環境を変換します。

#### 法則


local は extract を保存


extract(local(wa, f)) == extract(wa)


#### 実装

| 型コンストラクタ | `Of<A>`  | 備考                                             |
|------------------|----------|---------------------------------------------------|
| `EnvF<E>`        | `(E, A)` | `ask` は `E` を返す; `local` は `f` で `E` を変換 |

#### 例

``` rust
use karpal_core::prelude::*;

let w = ("hello", 42);

// ask: 環境を取得
assert_eq!(EnvF::<&str>::ask(&w), "hello");

// local: 環境を変換し、値を保持
let w2 = (10i32, "value");
let result = EnvF::<i32>::local(w2, |e| e * 2);
assert_eq!(result, (20, "value"));

// 法則: local は抽出される値を変更しない
assert_eq!(
    EnvF::<i32>::extract(&EnvF::<i32>::local((5, 99), |e| e + 1)),
    EnvF::<i32>::extract(&(5, 99))
);
```


## ComonadStore


### ComonadStore\<S\>

位置と覗き見の概念を持つコモナド。State の双対。


#### シグネチャ

``` rust
pub trait ComonadStore<S>: HKT {
    fn pos<A>(wa: &Self::Of<A>) -> S;
    fn peek<A>(s: S, wa: &Self::Of<A>) -> A;

    /// 焦点の値を抽出 (`peek(pos(wa), wa)` と等価)。
    fn extract<A>(wa: &Self::Of<A>) -> A
    where
        S: Clone;
}
```

`pos` はストア内の現在位置 (インデックス、キー、カーソル) を返します。`peek` は任意の位置の値を取得します。デフォルトの `extract` メソッドは `peek(pos(wa), wa)` として定義されます。

**設計メモ:** `ComonadStore` は `Comonad` ではなく `HKT` をスーパートレイトとして必要とします。`StoreF<S>` は `(Box<dyn Fn(S) -> A>, S)` として表現され、`S` に `'static` 境界を必要とします。汎用 `Functor` トレイトはこの境界を持たないため、`StoreF` は `Functor` を実装できず、したがって `Extend` や `Comonad` を実装できません。代わりに `extract` メソッドがこのトレイトに直接提供されます。

#### 法則


peek-pos 単位律


peek(pos(wa), wa) == extract(wa)


#### 実装

| 型コンストラクタ | `Of<A>`                    | 備考                                      |
|------------------|----------------------------|--------------------------------------------|
| `StoreF<S>`      | `(Box<dyn Fn(S) -> A>, S)` | alloc ゲート; `S: Clone + 'static` が必要 |

#### 例

``` rust
use karpal_core::prelude::*;

// Store は (参照関数、現在位置) のペア
let store: (Box<dyn Fn(i32) -> String>, i32) =
    (Box::new(|s| format!("value_{}", s)), 42);

// pos: 現在位置を取得
assert_eq!(StoreF::<i32>::pos(&store), 42);

// peek: 任意の位置の値を参照
assert_eq!(StoreF::<i32>::peek(10, &store), "value_10");

// extract: 現在位置を覗く
assert_eq!(StoreF::<i32>::extract(&store), "value_42");
```


## ComonadTraced


### ComonadTraced\<M: Monoid\>

モノイダルなトレース/アキュムレータを持つコモナド。Writer の双対。


#### シグネチャ

``` rust
pub trait ComonadTraced<M: Monoid>: HKT {
    fn trace<A>(m: M, wa: &Self::Of<A>) -> A;

    /// 焦点の値を抽出 (`trace(M::empty(), wa)` と等価)。
    fn extract<A>(wa: &Self::Of<A>) -> A;
}
```

`trace` はモノイダル入力でコモナド値を問い合わせます。デフォルトの `extract` メソッドはモノイダル単位 (`M::empty()`) でトレースし、蓄積されたトレースなしの「現在の」値を得ます。

**設計メモ:** `ComonadStore` と同様に、このトレイトは `Comonad` ではなく `HKT` をスーパートレイトとして必要とします。`TracedF<M>` は `Box<dyn Fn(M) -> A>` として表現され、汎用 `Functor` シグネチャと互換性のない `'static` 境界を課します。

#### 法則


単位トレース


trace(M::empty(), wa) == extract(wa)


#### 実装

| 型コンストラクタ | `Of<A>`               | 備考                                               |
|------------------|-----------------------|-----------------------------------------------------|
| `TracedF<M>`     | `Box<dyn Fn(M) -> A>` | alloc ゲート; `M: Monoid + Clone + 'static` が必要 |

#### 例

``` rust
use karpal_core::prelude::*;

// Traced コモナドはモノイドから値への関数
let w: Box<dyn Fn(i32) -> String> = Box::new(|m| format!("traced_{}", m));

// trace: 特定のモノイダル値で問い合わせ
assert_eq!(TracedF::<i32>::trace(5, &w), "traced_5");

// extract: モノイダル単位でトレース (i32::empty() == 0)
assert_eq!(TracedF::<i32>::extract(&w), "traced_0");
```


Karpal は Apache-2.0 + CLA でライセンスされています。[GitHub で見る](https://github.com/Industrial-Algebra/Karpal)。
