# プロ関手ファミリー

プロ関手: 第一引数について反変、第二引数について共変。

プロ関手ファミリーは `karpal-profunctor` クレートにあり、Karpal の [プロ関手オプティクス](optics.md) の背後にある抽象機構を提供します。`Functor` が単一の型パラメータ内の値を変換するのに対し、`Profunctor` は二パラメータ型を流れる値を変換します — 入力端と出力端を持つパイプと考えてください。パイプを開くことなく、入力を (反変に) 前処理し、出力を (共変に) 後処理できます。

## 階層

プロ関手階層はサブクラスに分岐し、それぞれが異なるオプティクスファミリーを可能にします:

``` rust
HKT2
  |
Profunctor          -- dimap、lmap、rmap            (Iso)
  |         \
Strong     Choice                                   (Lens / Prism)
  |           |
  +-----------+
        |
    Traversing      -- wander                       (Traversal)
```

- **Profunctor** -- 基本トレイト。同時の前処理と後処理のための `dimap` を提供。[Iso](optics.md#iso) を駆動。
- **Strong** -- 直積型 (タプル) を通じてプロ関手を持ち上げ。[Lens](optics.md#lens) を駆動。
- **Choice** -- 直和型 (`Result`) を通じてプロ関手を持ち上げ。[Prism](optics.md#prism) を駆動。
- **Traversing** -- 複数の焦点を扱うため Strong + Choice を拡張。[Traversal](optics.md#traversal) を駆動。

三つのトレイトすべてが `karpal-core` の `HKT2` エンコーディングを必要とします:

``` rust
pub trait HKT2 {
    type P<A, B>;
}
```

`HKT2` を実装する型は二パラメータ型コンストラクタです。型 `A` と `B` を与えられると、具体的な型 `P<A, B>` を生成します。


### Profunctor

第一引数について反変、第二引数について共変な型。


#### シグネチャ

``` rust
/// プロ関手は第一引数について反変、第二引数について共変である。
pub trait Profunctor: HKT2 {
    fn dimap<A: 'static, B: 'static, C, D>(
        f: impl Fn(C) -> A + 'static,
        g: impl Fn(B) -> D + 'static,
        pab: Self::P<A, B>,
    ) -> Self::P<C, D>;

    fn lmap<A: 'static, B: 'static, C>(
        f: impl Fn(C) -> A + 'static,
        pab: Self::P<A, B>,
    ) -> Self::P<C, B> { ... }

    fn rmap<A: 'static, B: 'static, D>(
        g: impl Fn(B) -> D + 'static,
        pab: Self::P<A, B>,
    ) -> Self::P<A, D> { ... }
}
```

`dimap` は基本演算です。入力を前処理する関数 `f: C -> A` (反変 — 逆方向に注意) と出力を後処理する関数 `g: B -> D` (共変) を取り、プロ関手 `P<A, B>` を `P<C, D>` に適応させます。

便利メソッド `lmap` と `rmap` は `dimap` に基づくデフォルト実装を持ちます:

- `lmap(f, pab)` -- 入力だけ前処理。`dimap(f, |b| b, pab)` と等価。
- `rmap(g, pab)` -- 出力だけ後処理。`dimap(|a| a, g, pab)` と等価。

#### 法則


単位律

両側で恒等関数で dimap しても何も変わりません:

``` rust
P::dimap(|a| a, |b| b, pab) == pab
```


合成律

合成された関数で dimap するのは二回 dimap するのと同じです:

``` rust
P::dimap(|a| f(g(a)), |b| h(i(b)), pab)
    == P::dimap(g, h, P::dimap(f, i, pab))
```

反変 (左) 側での順序の逆転に注意: 反変性が合成を逆転させるため、`f` のあと `g` は `|a| f(g(a))` になります。


#### 実装

| マーカー型  | `P<A, B>` の解決結果                | フィーチャーゲート    |
|--------------|--------------------------------------|-----------------|
| `FnP`        | `Box<dyn Fn(A) -> B>`                | `alloc`         |
| `ForgetF<R>` | `Box<dyn Fn(A) -> R>` (B はファントム) | `alloc`         |
| `TaggedF`    | `B` (A はファントム)                   | なし (`no_std`) |

#### 例

``` rust
use karpal_profunctor::{Profunctor, FnP};

// プロ関手値としての単純な二倍関数
let double: Box<dyn Fn(i32) -> i32> = Box::new(|x| x * 2);

// dimap: 入力側で文字列を i32 にパースし、
//        出力側で i32 結果を文字列にフォーマット
let f = FnP::dimap(
    |s: &str| s.len() as i32,  // 反変: &str -> i32
    |n: i32| n.to_string(),     // 共変: i32 -> String
    double,
);
assert_eq!(f("hello"), "10"); // len("hello") = 5、二倍 = 10

// lmap: 入力だけ前処理
let negate: Box<dyn Fn(i32) -> i32> = Box::new(|x| -x);
let neg_len = FnP::lmap(|s: &str| s.len() as i32, negate);
assert_eq!(neg_len("hi"), -2);

// rmap: 出力だけ後処理
let add_one: Box<dyn Fn(i32) -> i32> = Box::new(|x| x + 1);
let as_string = FnP::rmap(|n: i32| format!("result: {}", n), add_one);
assert_eq!(as_string(9), "result: 10");
```


### Strong

直積型 (タプル) を通じて持ち上げられるプロ関手。`first` と `second` 演算を提供し、プロ関手をペアの一方の成分に適用してもう一方をそのまま通過させます。これが [Lens](optics.md#lens) を駆動します — レンズは「積の一部に焦点を当てる」Strong プロ関手です。


### Choice

直和型 (`Result`) を通じて持ち上げられるプロ関手。`left` と `right` 演算を提供し、プロ関手を `Result` の一方のバリアントに適用します。これが [Prism](optics.md#prism) を駆動します — プリズムは「和の一つのバリアントに焦点を当てる」Choice プロ関手です。


### Traversing

複数の焦点を扱うため Strong + Choice を拡張します。`wander` 演算は `Traversable` を通じてプロ関手を持ち上げ、0 個以上の焦点を持つ [Traversal](optics.md#traversal) オプティクスを可能にします。


### FnP — 関数プロ関手

`FnP` は最も重要な具象実装です: `P<A, B> = Box<dyn Fn(A) -> B>`。すべてのオプティクス変換 (`Lens::transform`、`Prism::transform`) は `FnP` を使って再利用可能な `S -> S` 更新関数を生成します。


Karpal は Apache-2.0 + CLA でライセンスされています。[GitHub で見る](https://github.com/Industrial-Algebra/Karpal)。
