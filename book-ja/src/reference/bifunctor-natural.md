# Bifunctor と NaturalTransformation

二パラメータ関手と構造保存変換。

これら二つの抽象化はメインの Functor 階層に並行しながらも異なる懸念に対処します。`Bifunctor` は *二つの* 型パラメータを持つ型コンストラクタ上のマッピングを一般化し (`HKT2` を使用)、`NaturalTransformation` は含まれる値を検査することなく二つの単一パラメータ型コンストラクタ間で変換する方法を提供します。


### Bifunctor

二パラメータ型コンストラクタの両方の型パラメータでマップします。


#### シグネチャ

``` rust
/// Bifunctor: 二パラメータ型コンストラクタの両方の型パラメータでマップする。
///
/// 法則:
/// - 単位律: `bimap(id, id, fab) == fab`
/// - 合成律: `bimap(f . g, h . i, fab) == bimap(f, h, bimap(g, i, fab))`
pub trait Bifunctor: HKT2 {
    fn bimap<A, B, C, D>(
        fab: Self::P<A, B>,
        f: impl Fn(A) -> C,
        g: impl Fn(B) -> D,
    ) -> Self::P<C, D>;

    fn first<A, B, C>(fab: Self::P<A, B>, f: impl Fn(A) -> C) -> Self::P<C, B> {
        Self::bimap(fab, f, |b| b)
    }

    fn second<A, B, D>(fab: Self::P<A, B>, g: impl Fn(B) -> D) -> Self::P<A, D> {
        Self::bimap(fab, |a| a, g)
    }
}
```

`bimap` メソッドは二つの関数を同時に適用します — 各型パラメータに一つ。`first` と `second` メソッドは一つのパラメータだけをマップし、もう一方を変更しない便利なショートカットです。両方とも `bimap` に基づくデフォルト実装を持ちます。

`Bifunctor` は二パラメータ高階型トレイトである `HKT2` を拡張することに注意してください。`HKT` が `type Of<T>` を持つのに対し、`HKT2` は `type P<A, B>` を持ち、二つの型パラメータを反映します。

#### 法則


単位律

二つの恒等関数でマップした値は変更なく返されなければなりません:

``` rust
F::bimap(fab, |a| a, |b| b) == fab
```


合成律

合成された関数でマップすることは二段階でマップすることと等しくなければなりません:

``` rust
F::bimap(fab, |a| f(g(a)), |b| h(i(b)))
    == F::bimap(F::bimap(fab, g, i), f, h)
```


#### 実装

| マーカー型 | `P<A, B>` の解決結果 | 振る舞い                                                  | フィーチャーゲート    |
|-------------|-----------------------|-----------------------------------------------------------|-----------------|
| `ResultBF`  | `Result<B, A>`        | `f` は `Err` 側をマップ、`g` は `Ok` 側をマップ | なし (`no_std`) |
| `TupleF`    | `(A, B)`              | `f` は第一要素をマップ、`g` は第二要素をマップ | なし (`no_std`) |

`ResultBF` は第一型パラメータを `Err` の位置に、第二を `Ok` の位置に置くことに注意してください (`P<A, B> = Result<B, A>`)。これは第二パラメータが「主」であるという Bifunctor の慣習と一致し、`ResultF<E>` が `Ok` 値を関手の対象として扱う仕方に合致します。

#### 例

``` rust
use karpal_core::bifunctor::Bifunctor;
use karpal_core::hkt::{ResultBF, TupleF};

// Result 上の bimap: Ok と Err の両側を変換
let r: Result<i32, &str> = Ok(5);
let result = ResultBF::bimap(r, |s| s.len(), |n| n * 2);
assert_eq!(result, Ok(10));

let r: Result<i32, &str> = Err("hello");
let result = ResultBF::bimap(r, |s| s.len(), |n| n * 2);
assert_eq!(result, Err(5));

// タプル上の bimap: 両要素を変換
assert_eq!(TupleF::bimap((1, "hi"), |x| x + 1, |s| s.len()), (2, 2));

// first と second: 片側だけマップ
assert_eq!(TupleF::first((1, "hi"), |x| x * 2), (2, "hi"));
assert_eq!(TupleF::second((1, "hi"), |s| s.len()), (1, 2));

// Result の first は Err 側をマップ
let r: Result<i32, &str> = Err("hi");
assert_eq!(ResultBF::first(r, |s| s.len()), Err(2));

// Result の second は Ok 側をマップ
let r: Result<i32, &str> = Ok(5);
assert_eq!(ResultBF::second(r, |n| n * 3), Ok(15));
```


### NaturalTransformation

二つの型コンストラクタ間の構造保存マッピング。


#### シグネチャ

``` rust
/// 自然変換: 構造を保存する二つの関手間のマッピング。
///
/// 法則:
/// - 自然性: `fmap_G(f, transform(fa)) == transform(fmap_F(f, fa))`
pub trait NaturalTransformation<F: HKT, G: HKT> {
    fn transform<A>(fa: F::Of<A>) -> G::Of<A>;
}
```

`NaturalTransformation` は含まれる型 `A` を知ったり気にしたりすることなく、ある型コンストラクタの値を別のものに変換します。このトレイトは二つの `HKT` 型コンストラクタ `F` (ソース) と `G` (ターゲット) でパラメータ化され、実装する構造体が変換の名前付き証拠として機能します。

`transform` メソッドは `A` について汎用的であるため、含まれる値を検査・変更できません — コンテナを再構築することしかできません。これが自然律が捉える主要な性質です。

#### 法則


自然律

`transform` の結果に関数 `f` をマップすることは、元のものに `f` をマップしてから変換することと等しくなければなりません:

``` rust
G::fmap(NT::transform(fa), f) == NT::transform(F::fmap(fa, f))
```

言い換えると、先にマップしてから変換しても、先に変換してからマップしても同じです。図式は可換です。


#### 実装

| 構造体            | ソース (`F`) | ターゲット (`G`) | 振る舞い                                             | フィーチャーゲート     |
|-------------------|--------------|--------------|------------------------------------------------------|------------------|
| `OptionToVec`     | `OptionF`    | `VecF`       | `None` は `vec![]`、`Some(a)` は `vec![a]` | `std` または `alloc` |
| `VecHeadToOption` | `VecF`       | `OptionF`    | 最初の要素を取る; 空 `Vec` は `None`  | `std` または `alloc` |

#### 例

``` rust
use karpal_core::natural::{NaturalTransformation, OptionToVec, VecHeadToOption};

// OptionToVec: Option をゼロまたは一要素の Vec に変換
assert_eq!(OptionToVec::transform(Some(42)), vec![42]);
assert_eq!(OptionToVec::transform(None::<i32>), Vec::<i32>::new());

// VecHeadToOption: 最初の要素を Option として抽出
assert_eq!(VecHeadToOption::transform(vec![1, 2, 3]), Some(1));
assert_eq!(VecHeadToOption::transform(Vec::<i32>::new()), None);
```

#### 自然律の検証

自然律は任意の関数 `f` についてチェックできます。`OptionToVec` の具体的な例を示します:

``` rust
use karpal_core::functor::Functor;
use karpal_core::hkt::{OptionF, VecF};
use karpal_core::natural::{NaturalTransformation, OptionToVec};

let x: Option<i32> = Some(5);
let f = |a: i32| a + 1;

// マップしてから変換
let left = OptionToVec::transform(OptionF::fmap(x, f));

// 変換してからマップ
let right = VecF::fmap(OptionToVec::transform(x), f);

assert_eq!(left, right); // どちらも vec![6]
```


Karpal は Apache-2.0 + CLA でライセンスされています。[GitHub で見る](https://github.com/Industrial-Algebra/Karpal)。
