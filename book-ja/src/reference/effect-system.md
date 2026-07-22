# エフェクトシステムとモナド変換子

`karpal-effect` クレートはモナド変換子 — 任意の内側のモナドの上にエフェクト (エラー、状態、環境、ログ) を積み重ねるための合成可能な構成要素 — を提供します。また、Rust の `Box<dyn Fn>` が必要とする `'static` 境界を持つ関手階層のバリアントである `FunctorSt`、`ApplicativeSt`、`ChainSt` を導入します。

## 概要

| 変換子            | 表現                    | エフェクト                                                    |
|------------------|-----------------------------------|-----------------------------------------------------------|
| `ExceptTF<E, M>` | `M::Of<Result<A, E>>`             | エラー処理 — `Err` で短 circuits                  |
| `WriterTF<W, M>` | `M::Of<(A, W)>`                   | ログ蓄積 — `W` は `Monoid` でなければならない                 |
| `ReaderTF<E, M>` | `Box<dyn Fn(E) -> M::Of<A>>`      | 共有環境 — すべての計算が同じ `E` を読む |
| `StateTF<S, M>`  | `Box<dyn Fn(S) -> M::Of<(S, A)>>` | 可変状態 — 状態は計算を通じて受け渡される    |

四つの変換子すべてが `HKT`、`FunctorSt`、`ChainSt`、`MonadTrans` を実装します。`ExceptTF` と `WriterTF` は加えて `ApplicativeSt` を実装します。

## 静的型クラス

`karpal-core` の標準 `Functor` / `Applicative` / `Chain` トレイトは型パラメータに `'static` 境界を持ちません。内部で `Box<dyn Fn>` を使うモナド変換子はこれらの境界を必要とするため、`karpal-effect` は `St` 接尾辞を持つ並行トレイトを導入します。


### FunctorSt / ApplicativeSt / ChainSt

変換子互換性のための `'static` 境界を持つミラートレイト。


#### シグネチャ

``` rust
pub trait FunctorSt: HKT {
    fn fmap_st<A: 'static, B: 'static>(
        fa: Self::Of<A>,
        f: impl Fn(A) -> B + 'static,
    ) -> Self::Of<B>;
}

pub trait ApplicativeSt: FunctorSt {
    fn pure_st<A: 'static>(a: A) -> Self::Of<A>;
}

pub trait ChainSt: FunctorSt {
    fn chain_st<A: 'static, B: 'static>(
        fa: Self::Of<A>,
        f: impl Fn(A) -> Self::Of<B> + 'static,
    ) -> Self::Of<B>;
}
```

#### 基本実装

| 型         | `FunctorSt` | `ApplicativeSt` | `ChainSt` |
|--------------|-------------|-----------------|-----------|
| `OptionF`    | あり         | あり             | あり       |
| `ResultF<E>` | あり         | あり             | あり       |
| `IdentityF`  | あり         | あり             | あり       |
| `VecF`       | あり         | あり             | あり       |

これらの実装は自明です — `OptionF` の場合、`fmap_st` は単なる `fa.map(f)` です。`'static` 境界は `Box<dyn Fn>` が必要とするものに一致するため、boxed クロージャで動作する基本型は自動的にそれを満たします。


### MonadTrans

内側のモナド計算を変換子スタックに持ち上げます。


#### シグネチャ

``` rust
pub trait MonadTrans<M: HKT>: HKT {
    fn lift<A: 'static>(ma: M::Of<A>) -> Self::Of<A>
    where
        M::Of<A>: Clone;
}
```

`lift` はエフェクトを追加せずに `M` 計算を変換子に埋め込みます。クロージャベースの変換子 (ReaderT、StateT) の内側の関数が複数回呼び出される可能性があるため、`M::Of<A>` の `Clone` 境界が必要です。

#### 法則


lift は pure を保存する

``` rust
lift(M::pure_st(a)) == pure(a)
```


#### 例

``` rust
use karpal_effect::{MonadTrans, ExceptTF, WriterTF, ReaderTF, StateTF};
use karpal_core::hkt::OptionF;

// Some(42) を ExceptT に持ち上げ — Some(Ok(42)) を生成
let lifted = ExceptTF::<&str, OptionF>::lift(Some(42));
assert_eq!(lifted, Some(Ok(42)));

// Some(42) を WriterT に持ち上げ — Some((42, "")) を生成
let lifted = WriterTF::<String, OptionF>::lift(Some(42));
assert_eq!(lifted, Some((42, String::new())));

// Some(42) を ReaderT に持ち上げ — 環境を無視
let lifted = ReaderTF::<i32, OptionF>::lift(Some(42));
assert_eq!(lifted(999), Some(42));

// Some(42) を StateT に持ち上げ — 状態を変更なく通過
let lifted = StateTF::<i32, OptionF>::lift(Some(42));
assert_eq!(lifted(99), Some((99, 42)));
```


## モナド変換子


### ExceptTF\<E, M\>

内側のモナドにエラー処理を追加します。Haskell の `EitherT` / `ExceptT` と等価です。表現は `M::Of<Result<A, E>>` で、内側のモナドの `Ok`/`Err` 値をラップします。`Err` が発生すると計算は短絡します。`pure` は `Ok` で値を包み、`chain` は `Err` を伝播させます。

### WriterTF\<W, M\>

ログ蓄積を追加します。表現は `M::Of<(A, W)>` で、値とモノイダルなログ `W` を対にします。`tell` でログに追記し、`listen` でログを読み取ります。`W` は `Monoid` でなければなりません (例: `String`、`Vec<Event>`)。

### ReaderTF\<E, M\>

共有環境を追加します。表現は `Box<dyn Fn(E) -> M::Of<A>>` で、環境 `E` を受け取り計算を生成する関数です。`ask` で環境を取得し、`local` で環境を局所的に変更します。すべての計算が同じ `E` を読みます。

### StateTF\<S, M\>

可変状態を追加します。表現は `Box<dyn Fn(S) -> M::Of<(S, A)>>` で、状態 `S` を受け取り新しい状態と値を生成します。`get`/`put`/`modify` で状態を操作します。状態は計算の連鎖を通じて明示的に受け渡されます。


Karpal は Apache-2.0 + CLA でライセンスされています。[GitHub で見る](https://github.com/Industrial-Algebra/Karpal)。
