# 随伴と圏論

`karpal-core` の高度な圏論的構成。随伴 F ⊣ U はモナドとコモナドを生み出す根本的な関係です。このモジュールは関手合成、end、coend、双自然変換、継続モナド、プロ関手レベルの随伴も含みます。

## 概要

| 概念                   | モジュール       | 主要なアイデア                                                                                      |
|---------------------------|--------------|-----------------------------------------------------------------------------------------------|
| `Adjunction<F, U>`        | `adjunction` | F ⊣ U: `unit: A → U(F(A))`、`counit: F(U(B)) → B`                                             |
| `ComposeF<F, G>`          | `compose`    | 関手合成: `(F . G)(A) = F(G(A))`                                                   |
| `DinaturalTransformation` | `dinatural`  | プロ関手の対角間の変換: `P(A,A) → Q(A,A)`                                     |
| `End<P>`                  | `end`        | 全称量化: `∀A. P(A,A)`                                                        |
| `Coend<P, A>`             | `coend`      | 存在量化: `∃A. P(A,A)`                                                      |
| `ContravariantAdjunction` | `adjunction` | 反変関手間の随伴; `ContF<R> ⊣ ContF<R>` が継続モナドを与える |
| `ProfunctorAdjunction`    | `adjunction` | プロ関手の圏における随伴                                                     |

## 随伴


### Adjunction\<F, U\>

左随伴 F と右随伴 U の間の根本的な関係。


#### シグネチャ

``` rust
pub trait Adjunction<F: HKT, U: HKT> {
    fn unit<A: Clone + 'static>(a: A) -> U::Of<F::Of<A>>;
    fn counit<B: 'static>(fub: F::Of<U::Of<B>>) -> B;
}
```

一部の右随傍 (`ReaderF<E>` など) は `Box<dyn Fn>` の `'static` 制限のため汎用 `Functor` トレイトを実装できないため、このトレイトは `Functor` ではなく `HKT` で境界付けされます。

#### 法則 (三角形の恒等式)


左の三角形

``` rust
// counit(F::fmap(fa, unit)) == fa
// 「上がってから戻るのは F 上で恒等」
```


右の三角形

``` rust
// U::fmap(unit(a), counit) == a
// 「上がってから戻るのは U 上で恒等」
```


#### 派生演算

``` rust
// left_adjunct: (F(A) -> B) -> (A -> U(B))
fn left_adjunct(f, a) = U::fmap(unit(a), f)

// right_adjunct: (A -> U(B)) -> (F(A) -> B)
fn right_adjunct(f, fa) = counit(F::fmap(fa, f))
```

#### 実装

| 証拠       | F (左)    | U (右)    | フィーチャー  |
|---------------|-------------|--------------|----------|
| `IdentityAdj` | `IdentityF` | `IdentityF`  | `no_std` |
| `CurryAdj<E>` | `EnvF<E>`   | `ReaderF<E>` | `alloc`  |


## 随伴からのモナドとコモナド

すべての随伴 F ⊣ U はモナドとコモナドの両方を生み出します:

- **U . F 上のモナド** — `pure = unit`、`join = U(counit)`
- **F . U 上のコモナド** — `extract = counit`、`duplicate = F(unit)`


### CurryAdj\<E\> — 直積/指数 随伴

`EnvF<E> ⊣ ReaderF<E>`: State と Store を与える標準的な随伴。


#### どう動くか

``` text
EnvF<E>::Of<A>   = (E, A)            -- 直積 (「環境とのペアリング」)
ReaderF<E>::Of<A> = Box<dyn Fn(E) -> A>  -- 指数 (「環境からの関数」)

unit(a) = |e| (e, a)                   -- 値をペアの reader に埋め込む
counit((e, f)) = f(e)                  -- 関数を環境に適用
```

#### State モナド (U . F = ReaderF . EnvF)

合成関手 `ReaderF<E> . EnvF<E>` は `Of<A> = Box<dyn Fn(E) → (E, A)>` を与えます — まさに State モナドで、環境 `E` が受け渡され、潜在的に変更されます。

``` rust
use karpal_core::adjunction::*;

// State モナド: E -> (E, A) ここで E は可変状態
let get = state_get::<i32>();               // |e| (e, e)
let put = |s| state_put(s);                  // |_| (s, ())
let modify = state_modify(|e: i32| e + 1);  // |e| (e+1, ())

// Pure は状態に触れずに値を包む
let pure_42 = state_pure::<i32, _>(42);
assert_eq!(pure_42(0), (0, 42));

// Chain は状態渡し計算を逐次化する
let program = state_chain(
    state_get::<i32>(),
    |x| state_chain(
        state_modify(move |e: i32| e + x),
        |_| state_get::<i32>(),
    ),
);
assert_eq!(program(10), (20, 20));  // 10 を取得、10 を加算、20 を取得
```

#### Store コモナド (F . U = EnvF . ReaderF)

合成関手 `EnvF<E> . ReaderF<E>` は `Of<A> = (E, Box<dyn Fn(E) → A>)` を与えます — Store コモナドで、位置と値を参照する関数を保持します。


## 関手合成

`ComposeF<F, G>` は二つの関手を合成します: `(F . G)(A) = F(G(A))`。これにより、複数の関手変換を一つの型に連鎖できます。


## End と Coend

`End<P>` は全称量化 `∀A. P(A, A)` をエンコードし、`Coend<P, A>` は存在量化 `∃A. P(A, A)` をエンコードします。これらはプロ関手の対角上の普遍的構成で、自然変換と双自然変換の理論で中心的な役割を果たします。


## 双自然変換

`DinaturalTransformation` はプロ関手の対角間の変換 `P(A, A) → Q(A, A)` です。通常の自然変換の一般化で、反変・共変の両方の位置を持ちます。


## 継続モナドとプロ関手随伴

`ContravariantAdjunction` は反変関手間の随伴です。`ContF<R> ⊣ ContF<R>` は継続モナドを与えます。`ProfunctorAdjunction` はプロ関手の圏における随伴で、より高次の構成を可能にします。


Karpal は Apache-2.0 + CLA でライセンスされています。[GitHub で見る](https://github.com/Industrial-Algebra/Karpal)。
