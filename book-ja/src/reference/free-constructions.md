# 自由構成

自由構成は型コンストラクタから代数構造を「無料で」生成します。これらは `karpal-free` クレートにあり、`karpal-core` の HKT エンコーディングと型クラス階層の上に構築されます。

## 概要

| 型                 | 何を与えるか       | 主要なアイデア                                                                 |
|----------------------|-------------------------|--------------------------------------------------------------------------|
| `Coyoneda<F, A, B>`  | 自由関手            | `F: Functor` なしの `fmap`、`lower()` まで延期                    |
| `Yoneda<F, A>`       | マップ融合              | CPS による O(1) マップ合成; `lift` は `F: Functor` を必要               |
| `Free<F, A>`         | 自由モナド              | モナド的プログラムをデータとして構築、`fold_map` で解釈                |
| `Cofree<F, A>`       | 自由コモナド          | 注釈付き木/ストリーム; `F` が分岐の形を決定                  |
| `Freer<F, A>`        | 自由モナド (Functor なし) | `Free` と同様だが `fold_map` まで `F: Functor` を必要としない                         |
| `Lan<G, H, A, B>`    | 左 Kan 拡張      | Coyoneda を一般化; `fmap` は抽出関数を合成                  |
| `Ran` (トレイト)        | 右 Kan 拡張     | CPS 形 `∀R. (A → G R) → H R`; Codensity を一般化                    |
| `Codensity<F, A>`    | CPS モナド               | 境界なしの `pure`/`chain`; `to_monad` は `F: Applicative + Chain` を必要 |
| `Density<W, A>`      | CPS コモナド             | `W` に境界なしの `extract`/`fmap`                                   |
| `Day<F, G, A, B, C>` | Day 畳み込み         | 二つの関手と結合関数をペアリング; 二つの NT 経由で解釈     |
| `FreeAp<F, A>`       | 自由アプリカティブ        | 解釈前のエフェクトの静的解析; `F` への `retract`     |
| `FreeAlt<F, A>`      | 自由 Alternative        | アプリカティブ分岐間の選択; `zero`/`alt`/`retract`                |

## 型


### Coyoneda\<F, A, B\>

自由関手 — `fmap` を関数合成として延期することで、任意の型コンストラクタを Functor にします。


#### 定義

``` rust
pub struct Coyoneda<F: HKT, A, B> {
    f: Box<dyn Fn(B) -> A>,   // 蓄積された変換
    fb: F::Of<B>,              // 元の値
    _marker: PhantomData<F>,
}

pub struct CoyonedaF<F: HKT>(PhantomData<F>);
```

`Coyoneda<F, A, B>` は `F<B>` と関数 `B → A` を一緒に格納します。型パラメータ `B` は `lift` からの元の「基本」型です。`fmap` を呼ぶと蓄積された関数に合成されます (`A` は変わるが `B` は固定)。`lower()` だけが単一の `F::fmap` 経由で合成された関数を適用します。

#### 主要メソッド

``` rust
impl<F: HKT, A: 'static> Coyoneda<F, A, A> {
    /// F<A> を Coyoneda に持ち上げる。Functor 境界は不要。
    pub fn lift(fa: F::Of<A>) -> Self;
}

impl<F: HKT, A: 'static, B: 'static> Coyoneda<F, A, B> {
    /// Functor 境界なしでマップ — 蓄積された関数に合成。
    pub fn fmap<C: 'static>(self, g: impl Fn(A) -> C + 'static) -> Coyoneda<F, C, B>;

    /// F::fmap 経由で蓄積された関数を適用し、F<A> を生成。
    /// これが F: Functor を必要とする唯一の演算。
    pub fn lower(self) -> F::Of<A> where F: Functor;
}
```

#### 例

``` rust
use karpal_free::{Coyoneda, CoyonedaF};
use karpal_core::hkt::OptionF;

// 複数の fmap を連鎖 — まだ Functor は不要
let co = Coyoneda::<OptionF, _, _>::lift(Some(1))
    .fmap(|x| x + 1)
    .fmap(|x| x * 10)
    .fmap(|x| x + 5);

// lower() だけが Functor を必要 — すべてのマップを一度に適用
let result = co.lower();
assert_eq!(result, Some(25)); // (1+1)*10+5
```


### Yoneda\<F, A\>

データ型としての米田の補題 — CPS による O(1) マップ合成。


#### 定義

``` rust
pub struct Yoneda<F: HKT + Functor + 'static, A: 'static> {
    inner: Box<dyn YonedaLower<F, A>>,
}

pub struct YonedaF<F: HKT + Functor + 'static>(PhantomData<F>);
```

`Yoneda<F, A>` は Coyoneda に似ていますが、`lift` は `F: Functor` を必要とします。利点は **マップ融合** です: `fmap` 呼び出しの連鎖は `F` に適用する前に合成されるため、`lower()` は構造を一度しか走査しません。

#### Coyoneda と Yoneda の比較

|                           | Coyoneda                        | Yoneda                              |
|---------------------------|---------------------------------|-------------------------------------|
| `lift` は Functor を必要?  | いいえ                              | はい                                 |
| `fmap` は Functor を必要?  | いいえ                              | いいえ                                  |
| `lower` は Functor を必要? | はい                             | いいえ (既に持ち上げ済み)                 |
| ユースケース                  | 非 Functor 型をマップ可能にする | パフォーマンスのためにマップの連鎖を融合 |


### Free\<F, A\>

自由モナド — モナド的計算をデータ構造として構築し、解釈します。`Pure(a)` (完了した計算) と `Roll(F<Free<F,A>>)` (一層のエフェクト) の二つのバリアントを持ちます。`pure`/`lift_f` で構築し、`chain` で合成し、`fold_map` で任意のターゲットモナドに [自然変換](bifunctor-natural.md) 経由で解釈します。


### Cofree\<F, A\>

自由コモナド — 注釈付き木やストリーム。`Of<A> = (A, F<Cofree<F, A>>)` で、各ノードが値 `A` と `F` で決まる分岐を持ちます。`extract` で注釈を読み、`extend`/`duplicate` で構造全体に拡張します。


### Freer\<F, A\>

Functor 制約なしの自由モナド。`Free` と同様ですが、`fold_map` まで `F: Functor` を必要としません。内部で Kan 拡張を使います。


### Lan と Ran — Kan 拡張

`Lan<G, H, A, B>` は左 Kan 拡張で Coyoneda を一般化します。`Ran` は右 Kan 拡張で、CPS 形 `∀R. (A → G R) → H R` を持ち、Codensity を一般化します。


### Codensity と Density

`Codensity<F, A>` は CPS モナドで、`F: Monad` 境界なしに `pure`/`chain` を提供します。自由モナドのパフォーマンス最適化によく使われます。`Density<W, A>` はその双対で、CPS コモナドです。


### Day 畳み込み

`Day<F, G, A, B, C>` は二つの関手を結合関数とペアリングします。アプリカティブを合成する構造を与え、二つの自然変換経由で解釈します。


### FreeAp\<F, A\>

自由アプリカティブ — 解釈前にエフェクトツリーの静的解析を可能にします。`Pure` と `Ap` バリアントを持ち、`retract` で `F` 自身に解釈します。

注: 一般の `fold_map` (任意の `G` への自然変換経由の解釈) は Rust では不可能です。詳細は [FreeAp fold_map の探求](../design-notes/freeap-fold-map-exploration.md) を参照してください。


### FreeAlt\<F, A\>

自由 Alternative — アプリカティブ分岐間の選択。`zero`、`alt`、`retract` を提供します。


Karpal は Apache-2.0 + CLA でライセンスされています。[GitHub で見る](https://github.com/Industrial-Algebra/Karpal)。
