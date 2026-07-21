# 帰納スキーム

帰納スキームは再帰的データを fold と unfold するための構造化された合成可能なパターンを提供します。これらは `karpal-recursion` クレートにあり、`karpal-core` (HKT、Functor) と `karpal-free` (Cofree、Free) に依存します。

## 概要

| 型 / 関数 | カテゴリ             | 主要なアイデア                                                     |
|-----------------|----------------------|--------------------------------------------------------------|
| `Fix<F>`        | 不動点          | 再帰の結び目を結ぶ: `Fix<F> ≅ F<Fix<F>>`                |
| `Mu<F>`         | 最小不動点    | `Fix<F>` の型エイリアス (Rust は有限性を強制できない)      |
| `Nu<F, Seed>`   | 最大不動点 | シード + 余代数; 余再帰的構造の遅延観察 |
| `cata`          | Fold                 | カタモルフィズム — `F<A> → A` でボトムアップに fold                |
| `ana`           | Unfold               | アナモルフィズム — `A → F<A>` でトップダウンに unfold                |
| `hylo`          | Refold               | ハイロモルフィズム — unfold してから fold、中間の `Fix` なし       |
| `para`          | Fold+                | パラモルフィズム — 元の部分項へのアクセス付き fold         |
| `apo`           | Unfold+              | アポモルフィズム — `Either` による早期終了付き unfold     |
| `histo`         | Fold++               | ヒストモルフィズム — `Cofree` による完全な履歴付き fold          |
| `futu`          | Unfold++             | フュートモルフィズム — `Free` による複数ステップ unfold                  |
| `zygo`          | Composite            | ジゴモルフィズム — 補助 fold と並行に fold          |
| `chrono`        | Composite            | クロノモルフィズム — `futu` ; `histo` を一回で           |

## 不動点


### Fix\<F\>

関手の不動点。`Fix<F> ≅ F<Fix<F>>` となるよう再帰の結び目を結びます。


#### 定義

``` rust
pub struct Fix<F: HKT>(Rc<F::Of<Fix<F>>>);

// Rc 参照カウント経由の無条件 Clone
impl<F: HKT> Clone for Fix<F> { ... }

pub type Mu<F> = Fix<F>;
```

#### 主要メソッド

``` rust
// 一層包む
Fix::new(f: F::Of<Fix<F>>) -> Fix<F>

// 一層剥がす (消費)
fix.unfix() -> F::Of<Fix<F>>  // ただし F::Of<Fix<F>>: Clone

// 一層借用
fix.unfix_ref() -> &F::Of<Fix<F>>
```

#### 設計: Rc 対 Box

`Fix` は間接化に `Box` ではなく `Rc` を使います。これにより `Fix<F>: Clone` が無条件 (単なる参照カウントのインクリメント) になり、パラモルフィズムに不可欠です — パラモルフィズムは各部分項を保存と消費の両方をする必要があります。Rust のトレイトソルバは `Fix<OptionF>: Clone ↔ Option<Fix<OptionF>>: Clone` のような余帰納的 Clone 境界を証明できないため、`Box` は `Clone` を不可能にします。

#### 例: 自然数

``` rust
use karpal_recursion::{Fix, cata, ana};
use karpal_core::hkt::OptionF;

// None = Zero、Some(n) = Succ(n)
let three: Fix<OptionF> = Fix::new(Some(Fix::new(Some(Fix::new(None)))));

// または ana で構築:
let five: Fix<OptionF> = ana(
    |n: u32| if n == 0 { None } else { Some(n - 1) },
    5,
);

// cata で fold:
let count = cata::<OptionF, u32>(
    |layer| match layer {
        None => 0,
        Some(n) => n + 1,
    },
    five,
);
assert_eq!(count, 5);
```


### Nu\<F, Seed\>

最大不動点 — 遅延観察のための余代数と対になったシード。


#### 定義

``` rust
pub struct Nu<F: HKT, Seed> {
    pub seed: Seed,
    pub coalgebra: Box<dyn Fn(&Seed) -> F::Of<Seed>>,
}
```

#### 主要メソッド

``` rust
Nu::new(seed, coalgebra) -> Nu<F, Seed>
nu.observe() -> F::Of<Seed>   // 余代数を一回適用
nu.to_fix() -> Fix<F>         // ana 経由で完全に unfold
```

#### 例

``` rust
use karpal_recursion::Nu;
use karpal_core::hkt::OptionF;

let countdown = Nu::<OptionF, u32>::new(3, |&s| {
    if s == 0 { None } else { Some(s - 1) }
});
assert_eq!(countdown.observe(), Some(2));
```


## 帰納スキーム


### cata — カタモルフィズム

再帰的構造をボトムアップに fold します。基本的な「解体」演算です。


#### シグネチャ

``` rust
pub fn cata<F: HKT + Functor, A>(
    alg: impl Fn(F::Of<A>) -> A,
    fix: Fix<F>,
) -> A
```

#### 例: 自然数の和

``` rust
let n = ana(|s: u32| if s == 0 { None } else { Some(s - 1) }, 5);
let sum = cata::<OptionF, u32>(
    |layer| match layer {
        None => 0,
        Some(acc) => acc + 1,
    },
    n,
);
assert_eq!(sum, 5);
```

#### 法則

- `cata(Fix::new, x) == x` — コンストラクタで fold すれば恒等


### ana — アナモルフィズム

シードから再帰的構造をトップダウンに unfold します。


#### シグネチャ

``` rust
pub fn ana<F: HKT + Functor, A>(
    coalg: impl Fn(A) -> F::Of<A>,
    seed: A,
) -> Fix<F>
```

#### 例: 自然数の構築

``` rust
let three: Fix<OptionF> = ana(
    |n: u32| if n == 0 { None } else { Some(n - 1) },
    3,
);
```

#### 法則

- `cata(alg, ana(coalg, seed)) == hylo(alg, coalg, seed)`


### hylo — ハイロモルフィズム

一回で unfold してから fold します — 中間の `Fix` は割り当てられません。


#### シグネチャ

``` rust
pub fn hylo<F: HKT + Functor, A, B>(
    alg: impl Fn(F::Of<B>) -> B,
    coalg: impl Fn(A) -> F::Of<A>,
    seed: A,
) -> B
```

#### 主要な性質

`hylo` は `cata` と `ana` を融合します: 中間の `Fix<F>` 構造を構築せずに、unfold と fold を一回の走査で行います。これは効率上重要です — ハイロモルフィズムは中間データ構造をメモリに保持しません。

### para — パラモルフィズム

元の部分項へのアクセス付き fold。`cata` と異なり、`para` は各ノードで折り畳んだ結果 *と* 元の部分木の両方にアクセスできます。これにより、結果が入力の構造に依存する fold を表現できます。

### apo — アポモルフィズム

`Either` による早期終了付き unfold。`ana` と異なり、`apo` は再帰を早期に停止でき、残りを「既に構築済み」として直接挿入できます。

### histo — ヒストモルフィズム

`Cofree` による完全な履歴付き fold。`cata` は各ノードで子の結果しか見ませんが、`histo` は結果の *完全な履歴* (すべての祖先ノードの結果) にアクセスできます。これにより動的計画法のパターンを表現できます。

### futu — フュートモルフィズム

`Free` による複数ステップ unfold。`ana` は一度に一層しか生成しませんが、`futu` は複数層を一度に生成でき、より柔軟なコア再帰を可能にします。

### zygo — ジゴモルフィズム

補助 fold と並行に fold。二つの代数を同時に走査し、一方の結果を他方の計算で利用できます。

### chrono — クロノモルフィズム

`futu ; histo` を単一パスで。最も一般的な帰納スキームで、複数ステップの展開と完全な履歴へのアクセスを組み合わせます。


Karpal は Apache-2.0 + CLA でライセンスされています。[GitHub で見る](https://github.com/Industrial-Algebra/Karpal)。
