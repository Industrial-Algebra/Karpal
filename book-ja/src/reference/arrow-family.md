# アローファミリー

アローは関数を、構造化された入出力を持つ計算へと一般化します。[プロ関手](profunctor-family.md) のアイデア — 合成を持つ二パラメータ型コンストラクタ — を、積の経路付け、和の経路付け、適用、ループ、失敗を持つ合成可能パイプラインの完全な代数へと拡張します。

アローファミリーは `karpal-arrow` クレートにあり、`karpal-core` の `HKT2` エンコーディングの上に構築されます。

## トレイト階層

``` rust
HKT2
 +-> Semigroupoid          compose(f, g)
     +-> Category           id()
         +-> Arrow           arr(f)、first、second、split、fanout
              |-> ArrowChoice    left、right、splat、fanin
              |-> ArrowApply     app  (~ Monad)
              |-> ArrowLoop      loop_arrow  (D: Default)
              +-> ArrowZero      zero_arrow
                   +-> ArrowPlus  plus(f, g)
```

- **Semigroupoid** -- 合成可能な射 (結合的な合成)。
- **Category** -- 恒等射を追加。
- **Arrow** -- 純粋関数を持ち上げ、積 (タプル) を通じて経路付け。
- **ArrowChoice** -- 直和型 (`Result`) を通じて経路付け。
- **ArrowApply** -- 第一級のアロー適用、Monad と同等の表現力。
- **ArrowLoop** -- `D: Default` を使った正格評価でのフィードバック/フィックスポイントコンビネータ。
- **ArrowZero** -- 失敗/空のアロー。
- **ArrowPlus** -- アロー間の結合的な選択。

## トレイト


### Semigroupoid

結合的に合成できる射。


#### シグネチャ

``` rust
/// Semigroupoid: 合成可能な射。
///
/// 法則:
/// - 結合律: compose(f, compose(g, h)) == compose(compose(f, g), h)
pub trait Semigroupoid: HKT2 {
    fn compose<A: Clone + 'static, B: Clone + 'static, C: Clone + 'static>(
        f: Self::P<B, C>,
        g: Self::P<A, B>,
    ) -> Self::P<A, C>;
}
```

`compose` は二つの射を連鎖します: `g: A -> B` と `f: B -> C` を与えられ、`f . g: A -> C` を生成します。数学的慣習 (`g` のあと `f`) に合わせ、`f` が先に来ることに注意してください。

#### 法則


結合律

合成は結合的です:

``` rust
P::compose(f, P::compose(g, h)) == P::compose(P::compose(f, g), h)
```


### Category

恒等射を持つ Semigroupoid。


#### シグネチャ

``` rust
/// Category: 恒等射を持つ Semigroupoid。
///
/// 法則:
/// - 左単位律:  compose(id(), f) == f
/// - 右単位律: compose(f, id()) == f
pub trait Category: Semigroupoid {
    fn id<A: Clone + 'static>() -> Self::P<A, A>;
}
```

`id` は他の任意の射と合成するとその射を変更なく返す恒等射を生成します。

#### 法則


左単位律

``` rust
P::compose(P::id(), f) == f
```


右単位律

``` rust
P::compose(f, P::id()) == f
```


### Arrow

純粋関数を持ち上げ、積 (タプル) 上で操作できる Category。


#### シグネチャ

``` rust
/// Arrow: 純粋関数を持ち上げ、積上で操作できる Category。
///
/// 法則:
/// - arr(id) == id()
/// - arr(|a| g(f(a))) == compose(arr(g), arr(f))
/// - first(arr(f)) == arr(|(a, c)| (f(a), c))
/// - first(compose(f, g)) == compose(first(f), first(g))
pub trait Arrow: Category {
    /// 純粋関数をアローに持ち上げる。
    fn arr<A: Clone + 'static, B: Clone + 'static>(
        f: impl Fn(A) -> B + 'static,
    ) -> Self::P<A, B>;

    /// アローをペアの第一成分に適用し、第二成分を通過させる。
    fn first<A: Clone + 'static, B: Clone + 'static, C: Clone + 'static>(
        pab: Self::P<A, B>,
    ) -> Self::P<(A, C), (B, C)>;

    /// アローをペアの第二成分に適用する。
    fn second<A: Clone + 'static, B: Clone + 'static, C: Clone + 'static>(
        pab: Self::P<A, B>,
    ) -> Self::P<(C, A), (C, B)> { ... }

    /// `***`: 二つのアローを積上で並列に適用。
    fn split<A: Clone + 'static, B: Clone + 'static, C: Clone + 'static, D: Clone + 'static>(
        f: Self::P<A, B>,
        g: Self::P<C, D>,
    ) -> Self::P<(A, C), (B, D)> { ... }

    /// `&&&`: 入力を二つのアローに供給し、結果をペアとして収集。
    fn fanout<A: Clone + 'static, B: Clone + 'static, C: Clone + 'static>(
        f: Self::P<A, B>,
        g: Self::P<A, C>,
    ) -> Self::P<A, (B, C)> { ... }
}
```

`arr` は任意の純粋関数をアローに持ち上げます。`first` はアローをタプルの第一成分に適用し、第二成分を変更なく通過させます。`second`、`split`、`fanout` は `first`、`compose`、`arr` から構築されたデフォルト実装を持ちます。


### ArrowChoice

直和型 (`Result`) を通じてアローを経路付けします。`left` は `Result` の `Ok` バリアントにアローを適用し、`right` は `Err` バリアントに適用します。これにより条件付きの分岐パイプラインを構築できます。

### ArrowApply

第一級のアロー適用 (`app`) を提供します。アローを値として扱い、動的に適用できます。これは Monad と同等の表現力を持ちますが、ほとんどの実用的な用途では Arrow を使う方が構造化されています。

### ArrowLoop

フィードバック/フィックスポイントコンビネータ `loop_arrow` を提供します。Haskell の遅延 `loop` と異なり、Rust の正格評価ではフィードバック型 `D: Default` が必要で、`loop_arrow` は単一パス評価を使います (これは設計上の制限であり、完全な反復収束のためには `loop_fixpoint` を使います)。

### ArrowZero と ArrowPlus

`ArrowZero` は失敗/空のアロー `zero_arrow` を提供し、`ArrowPlus` はアロー間の結合的な選択 `plus(f, g)` を提供します。これらは Arrow 版の `Plus`/`Alt` です。


## 具象アロー


### FnA — 関数アロー

`FnA` は標準的な具象アローで、`P<A, B> = Box<dyn Fn(A) -> B + 'static>` です。すべてのアロートレイト (Category、Arrow、ArrowChoice、ArrowLoop、ArrowZero、ArrowPlus) を実装します。ほとんどのアローパイプラインは `FnA` で構築されます。


### KleisliF と CokleisliF

`KleisliF<M>` はモナド `M` からアローを構築します: `P<A, B> = A -> M::Of<B>`。モナド計算をアローパイプラインとして表現できます。`CokleisliF<W>` はその双対で、コモナド `W` から構築します: `P<A, B> = W::Of<A> -> B`。


Karpal は Apache-2.0 + CLA でライセンスされています。[GitHub で見る](https://github.com/Industrial-Algebra/Karpal)。
