# Foldable と Traversable

コンテナの内容を要約し、逐次化します。

## Foldable


### Foldable

要約値に畳み込める構造。


#### シグネチャ

``` rust
pub trait Foldable: HKT {
    fn fold_right<A, B>(fa: Self::Of<A>, init: B, f: impl Fn(A, B) -> B) -> B;

    fn fold_map<A, M: Monoid>(fa: Self::Of<A>, f: impl Fn(A) -> M) -> M {
        Self::fold_right(fa, M::empty(), |a, acc| f(a).combine(acc))
    }
}
```

`fold_right` が必須メソッドです。要素を右から左へ処理し、各ステップでアキュムレータを受け渡します。`fold_map` はデフォルトとして提供されます: 各要素を `Monoid` にマップして組み合わせます。

#### 法則


fold_map の一貫性

`fold_map(fa, f) == fold_right(fa, M::empty(), |a, acc| f(a).combine(acc))`


デフォルトの `fold_map` をオーバーライドする場合、上記の右畳みみ定式と一致しなければなりません。

#### 実装

| 型コンストラクタ | 備考                                                                 |
|------------------|-----------------------------------------------------------------------|
| `OptionF`        | 含まれる値があれば畳み込み; `None` なら `init` を返す。    |
| `ResultF<E>`     | `Ok` 値を畳み込み; `Err` なら `init` を返す。                  |
| `VecF`           | 反転して反復することで右畳み込み。`alloc` が必要。             |
| `IdentityF`      | 単一の含まれる値に自明に `f` を適用。                  |
| `NonEmptyVecF`   | 末尾を右畳み込みし、先頭に `f` を適用。`alloc` が必要。 |

#### 例: fold_map で合計

``` rust
use karpal_std::prelude::*;

// fold_map は各要素を Monoid にマップして組み合わせる。
// i32 の場合、Monoid 実装は加算を単位元 0 と共に使う。

let sum = VecF::fold_map(vec![1, 2, 3], |a: i32| a);
assert_eq!(sum, 6); // 1 + 2 + 3

let sum = OptionF::fold_map(Some(42), |a: i32| a);
assert_eq!(sum, 42);

let sum = OptionF::fold_map(None::<i32>, |a: i32| a);
assert_eq!(sum, 0); // i32 の Monoid::empty()
```

#### 例: fold_right

``` rust
use karpal_std::prelude::*;

// fold_right は要素を右から左へ処理する。
// 減算では結合性が重要:
// fold_right([1, 2, 3], 0, |a, b| a - b)
//   = 1 - (2 - (3 - 0))
//   = 1 - (2 - 3)
//   = 1 - (-1)
//   = 2
let result = VecF::fold_right(vec![1, 2, 3], 0, |a, b| a - b);
assert_eq!(result, 2);
```


## Traversable


### Traversable

エフェクト付き関数で走査できる Functor + Foldable。


#### シグネチャ

``` rust
pub trait Traversable: Functor + Foldable {
    fn traverse<G, A, B, F>(fa: Self::Of<A>, f: F) -> G::Of<Self::Of<B>>
    where
        G: Applicative,
        F: Fn(A) -> G::Of<B>,
        B: Clone;
}
```

`traverse` は構造内のすべての要素にエフェクト付き関数 `f` を適用し、結果をエフェクト `G` の内側に集めます。`f` の適用が一つでも「失敗」(`OptionF` の `None` など) を生成すると、走査全体が短絡します。

#### 法則


単位律

`traverse::<IdentityF, _, _, _>(fa, pure) == pure(fa)`


合成律

`traverse::<Compose<F, G>, _, _, _>(fa, |a| Compose(F::fmap(f(a), g)))`  
`== Compose(F::fmap(traverse::<F, _, _, _>(fa, f), |fb| traverse::<G, _, _, _>(fb, g)))`


自然律

`t(traverse::<F, _, _, _>(fa, f)) == traverse::<G, _, _, _>(fa, |a| t(f(a)))`  
任意のアプリカティブ自然変換 `t: F ~> G` について


Karpal はエフェクトとして `OptionF` を使ったプロパティベーステストで単位律を検証します。

#### 実装

| 型コンストラクタ | 備考                                                                                                      |
|------------------|------------------------------------------------------------------------------------------------------------|
| `OptionF`        | `Some` なら内側の値を走査; `None` なら `G::pure(None)` を返す。                                   |
| `ResultF<E>`     | `Ok` 値を走査; `Err` なら `G::pure(Err(e))` を返す。`E: Clone` が必要。                        |
| `VecF`           | 各要素を左から右へ走査し、`Applicative::ap` で蓄積。`alloc` と `B: Clone` が必要。 |

#### 例: Option での traverse

``` rust
use karpal_std::prelude::*;

// 文字列のリストを整数にパースし、いずれかが失敗すれば全体が失敗。
fn parse(s: &str) -> Option<i32> {
    s.parse().ok()
}

// すべての要素がパース成功:
let result = VecF::traverse::<OptionF, _, _, _>(
    vec!["1", "2", "3"],
    parse,
);
assert_eq!(result, Some(vec![1, 2, 3]));

// 一つの要素が失敗するので、走査全体が None を返す:
let result = VecF::traverse::<OptionF, _, _, _>(
    vec!["1", "oops", "3"],
    parse,
);
assert_eq!(result, None);
```

#### 例: Option 上の traverse

``` rust
use karpal_std::prelude::*;

// エフェクト付き関数で Option を走査:
let result = OptionF::traverse::<OptionF, _, _, _>(
    Some(3),
    |x| Some(x * 2),
);
assert_eq!(result, Some(Some(6)));

// 内側のエフェクトが失敗する場合:
let result = OptionF::traverse::<OptionF, _, _, _>(
    Some(3),
    |_x| None::<i32>,
);
assert_eq!(result, None);

// None の走査は常に成功:
let result = OptionF::traverse::<OptionF, i32, i32, _>(
    None,
    |x| Some(x * 2),
);
assert_eq!(result, Some(None));
```


Karpal は Apache-2.0 + CLA でライセンスされています。[GitHub で見る](https://github.com/Industrial-Algebra/Karpal)。
