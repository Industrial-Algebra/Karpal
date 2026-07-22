# トポス理論

`karpal-topos` クレートは [構造化された空](../concepts/structured-emptiness.md) の基盤となる圏論的インフラを実現します: 小圏、前層、篩、部分対象分類子 Ω、有限極限、グロタンディークトポロジー、層、そして米田の補題です。

これはフェーズ 16 のスタック — Karpal の最も抽象的なレイヤーで、「ゼロには幾何学がある」が形式化されます: 空である理由が空それ自と同じくらい重要であり、トポス理論がその言語を提供します (Ω は篩の束で、層化は局所から大域への貼り合わせです)。

## 概要

| モジュール           | 内容                                                         | フィーチャーゲート         |
|------------------|------------------------------------------------------------------|----------------------|
| `small_category` | `SmallCategory`、`ChainCat<N>` (有限半順序)、`DiscreteCat`     | `no_std`             |
| `presheaf`       | `Presheaf<C>`、`ConstantPresheaf`、`InitialSegmentPresheaf`      | コア; 前層の値は `alloc` |
| `representable`  | `Representable<c>` — ホム前層 `Hom(-, c)`                | `no_std`             |
| `sieve`          | `Sieve`、`FiniteSieve` (前合成で閉じた族)          | `alloc`              |
| `classifier`     | `Omega` (部分対象分類子)、`Terminal`、`TruthValue` 束 | `no_std`             |
| `limits`         | `pullback_fiber`、`equalizer_fiber`、`characteristic_at`        | `alloc`              |
| `topology`       | `GrothendieckTopology`、`LawvereTierneyTopology`                 | `no_std`             |
| `sheaf`          | `is_separated_at`、`is_sheaf_at`、層化インターフェース       | `alloc`              |
| `yoneda`         | `yoneda_apply`、`yoneda_extract` — 米田の全単射          | `no_std`             |

このクレートは三つの構成でビルドできます: `std`、`no_std + alloc`、純粋 `no_std` (`sieve`、`limits`、`sheaf` モジュールは `alloc` ゲートされています)。

## 小圏

### SmallCategory

**対象がファントムマーカー型**で **射がランタイムデータを運ぶ値**である小圏。

``` rust
pub trait SmallCategory {
    /// A から B への射の型。
    type Mor<A, B>;

    /// f: A → B のあとに g: B → C を合成し、g ∘ f: A → C を得る。
    fn compose<A, B, C>(g: Self::Mor<B, C>, f: Self::Mor<A, B>) -> Self::Mor<A, C>;
}
```

**法則:** 結合律 — `compose(h, compose(g, f)) == compose(compose(h, g), f)`。

#### なぜ `karpal_arrow::Category` でないのか?

`karpal_arrow::Category` は *計算可能な* 射に偏っています (`compose`/`id` が関数のような値を返す)。前層は、射がしばしば有限データ (単体圏 Δ、半順序圏) であるような任意の小圏上で定義されます。この `SmallCategory` は意図的に別物です: 射はインデックスデータです。

#### 恒等は具象圏ごと

Rust はファントム型パラメータから対象の恒等を抽出できないため、`SmallCategory` は `compose` だけを提供します。各具象圏は対象インデックストレイトに束ねられた固有メソッドとして `identity` を提供します。これは隠さない正直な制限であり、欠落ではありません。

### ChainCat\<N\>

有限鎖 `0 ≤ 1 ≤ … ≤ N` の半順序圏。射 `i → j` は `i ≤ j` のときに限り存在します (一意な証人)。これは最も単純な非自明な小圏です。

``` rust
use karpal_topos::{ChainCat, ChainMor, ChainObj, SmallCategory};

// 対象マーカー、それぞれが位置をコンパイル時に公開。
struct C0; struct C1; struct C2;
impl ChainObj for C0 { const IDX: usize = 0; }
impl ChainObj for C1 { const IDX: usize = 1; }
impl ChainObj for C2 { const IDX: usize = 2; }

// 恒等は固有メソッド:
let id: ChainMor<C1, C1> = ChainCat::<2>::identity::<C1>();

// 射は始域 ≤ 終域のときに限り存在:
let f: ChainMor<C0, C2> = ChainCat::<2>::morphism::<C0, C2>().unwrap();
assert!(ChainCat::<2>::morphism::<C2, C0>().is_none()); // 2 > 0、射なし

// 合成:
let g: ChainMor<C1, C2> = ChainCat::<2>::morphism::<C1, C2>().unwrap();
let gf: ChainMor<C0, C2> = ChainCat::<2>::compose(g, f);
assert_eq!((gf.from(), gf.to()), (0, 2));
```

`DiscreteCat` は退化したケースです: 恒等射しか存在しません。

## 前層

### Presheaf\<C\>

反変関手 `C^op → Set`。各対象に集合を割り当て、各射 `f: Dom → Cod` に制限写像 `restrict(f): P(Cod) → P(Dom)` を割り当てます。

``` rust
pub trait Presheaf<C: SmallCategory> {
    /// 集合 P(Obj): 対象 Obj での前層の値。
    type At<Obj>;

    /// f: Dom → Cod に沿った制限。P(Cod) → P(Dom) に写す。
    fn restrict<Dom, Cod>(f: C::Mor<Dom, Cod>, x: Self::At<Cod>) -> Self::At<Dom>;
}
```

**法則:**
- 単位律: `restrict(id, x) == x`
- 合成律: `restrict(g ∘ f, x) == restrict(f, restrict(g, x))`

**反変性**に注意してください: `f: Dom → Cod` に沿った制限は `Cod` での値を `Dom` での値に写し、合成の順序が逆転します。

#### 実装

| 前層                 | `P(i)`                       | 制限                                   |
|--------------------------|------------------------------|-----------------------------------------------|
| `ConstantPresheaf<T>`    | `T` (どこでも同じ)        | 恒等 (`x` を変更なく返す)              |
| `InitialSegmentPresheaf` | `{0, 1, …, i}` (`SegmentSet`)| 最初の `Dom::IDX + 1` 要素に切り詰め|
| `Representable<c>`       | `Hom(i, c)` (射)      | 前合成: `m ↦ m ∘ f`                   |
| `Omega`                  | `TruthValue` (篩の階数)    | `min(rank, Dom::IDX + 1)`                     |
| `Terminal`               | `()`                         | 恒等                                      |

### Representable\<c\>

ホム前層 `Hom_C(-, c)`。各対象 `d` について `At<d> = Hom_C(d, c)` です。`f: Dom → Cod` に沿った制限は前合成です: `Hom(Cod, c) → Hom(Dom, c)` が `m` を `m ∘ f` に送ります。これは米田の補題の錨です。

## 篩

対象 `c` 上の **篩** は `c` への射の前合成で閉じた族です: `f: d → c` が篩にあり `g: e → d` が任意の射のとき、合成 `f ∘ g` も篩にあります。篩はグロタンディークトポロジーの基盤となる「被覆」の概念です。

``` rust
use karpal_topos::{FiniteSieve, Sieve, ChainCat, ChainObj};
# struct C0; struct C2; struct C3;
# impl ChainObj for C0 { const IDX: usize = 0; }
# impl ChainObj for C2 { const IDX: usize = 2; }
# impl ChainObj for C3 { const IDX: usize = 3; }

// {2} 単独は閉じていない: 0→2、1→2 との前合成が 0 と 1 を必要とする。
let unclosed: FiniteSieve<C3> = FiniteSieve::new([2]);
assert!(!Sieve::<ChainCat<3>, C3>::is_closed(&unclosed));

// close() は下方閉包を強制: {2} が {0, 1, 2} になる。
let closed = unclosed.close();
assert!(Sieve::<ChainCat<3>, C3>::is_closed(&closed));

// 最大篩はすべての始域 [0, Cod::IDX] を含む。
let max: FiniteSieve<C3> = FiniteSieve::maximal();
```

## 部分対象分類子 Ω

前層トポス `[C^op, Set]` において、部分対象分類子 Ω は各対象 `c` に **`c` 上の篩の集合** を割り当てる前層です。`ChainCat<N>` 上では、篩は **階数** で表現可能な下方閉部分集合です — 鎖ヘイティング代数です。

### TruthValue

``` rust
pub struct TruthValue { pub rank: usize }
```

対象 `i` について、`Ω(i)` は階数 `0..=i+1` を含みます:
- 階数 `0` = 空の篩 (**bottom** — 「何も被覆されない」)
- 階数 `k` = 篩 `{0, …, k-1}`
- 階数 `i+1` = 最大篩 (**top** — 「すべてが被覆される」)

これは **ヘイティング代数** (直観主義論理) を形成し、構造化された空の基礎です:

``` rust
use karpal_topos::TruthValue;

let a = TruthValue { rank: 2 };
let b = TruthValue { rank: 4 };

a.meet(b);                          // 束の meet (篩の共通部分)
a.join(b);                          // 束の join (篩の和)
a.implies_at(b, 4);                 // 対象 4 でのヘイティング含意
a.neg_at(3);                        // ヘイティング否定: ¬a = a → bottom
```

注意: `¬¬a ≠ a` が一般に成り立ちます — これは古典論理ではなく直観主義論理です。欠落した中間項自体が一種の構造化された空です。

### Terminal と真値写像

`Terminal` は終前層 (すべての対象を `()` に送る) です。真値写像 `true: 1 → Ω` は最大篩を選びます:

``` rust
use karpal_topos::truth_at;
let max_sieve_on_2 = truth_at(2); // TruthValue { rank: 3 }
```

部分対象 `S ↪ A` は、`true` に沿った引き戻しが `S` を復元する一意の特性射 `χ: A → Ω` に対応します。

## 有限極限

前層トポスの極限は **各点で** 計算されます。自然変換は Rust では第一級の値になれないため (ランク N の壁)、これらは一つの対象での前層の値と射の作用を取る **ファイバー関数** として公開されます:

``` rust
use karpal_topos::{pullback_fiber, equalizer_fiber, characteristic_at};

// 一つの対象での引き戻しファイバー: f(p) == g(q) を満たす対 (p, q)。
let pb = pullback_fiber(&[1,2,3], &[10,20,30], |p| p % 2, |q| (q/10) % 2);

// 等化子ファイバー: f(p) == g(p) を満たす要素 p。
let eq = equalizer_fiber(&[1,2,3,4], |p| *p, |p| p + (p % 2));

// 対象 i での特性射 χ: p を部分対象に制限したものが S に留まるような最大の篩の階数。
let chi = characteristic_at(2, &42, |_p, j| j < 2); // 階数 2
```

定義定理: **`p ∈ S(i)` iff `χ(p)` が `i` 上の最大篩** — 部分対象は `true` に沿った χ の引き戻しです。

## グロタンディークトポロジー

**グロタンディークトポロジー** `J` は各対象に被覆篩の集まりを割り当てます。

``` rust
pub trait GrothendieckTopology {
    fn is_covering(i: usize, rank: usize) -> bool;
}
```

**法則** (公理チェッカーで検証):
1. **極大性** — 最大篩 (階数 `i+1`) は常に被覆。
2. **安定性** — 階数 `r` が `i` を被覆するなら、`min(r, j+1)` が `j` を被覆。
3. **推移性** — 「局所的に被覆する」篩は被覆。

| トポロジー           | 何が被覆されるか                                          |
|--------------------|------------------------------------------------------|
| `TrivialTopology`  | 最大篩のみ (階数 `i+1`)                 |
| `DenseTopology`    | 任意の空でない篩 (階数 ≥ 1)                       |

### ローヴェル・ティアニートポロジー

真値上の閉包作用素 `j: Ω → Ω` としての同値な概念:

``` rust
pub trait LawvereTierneyTopology {
    fn j(i: usize, rank: usize) -> usize;
}
```

**法則:** `j(top) = top`、`j(j(r)) = j(r)` (冪等性)、`j(min(r,s)) = min(j(r), j(s))` (meet を保存)。グロタンディークとローヴェル・ティアニートポロジーの間に全単射があります; `TrivialTopology` と `DenseTopology` は両方を実装します。

## 層

前層 `P` はトポロジー `J` に対して **層** であるとは、すべての被覆篩について、局所断面のすべての両立する族が大域断面に一意に貼り合わせられることです。

``` rust
use karpal_topos::{is_separated_at, is_sheaf_at};

// 分離 (一意な貼り合わせ): 異なる要素は異なる制限プロファイルを持つ。
let separated = is_separated_at(2, 3, &[1, 2, 3], |x, _k| *x);

// 完全な層条件: すべての両立する族が一意に貼り合わせられる。
let is_sheaf = is_sheaf_at(
    2, 1, &[7, 8],
    |_k| vec![7, 8],
    |x, _k| *x,
);
```

### 層化

層化 `a: PSh(C) → Sh(C, J)` は包含の左随伴です — 前層を「最良の層近似」に送ります。完全なプラス構成は genuinely 複雑で実装されて **いません**; インターフェースは随伴の形状 (単位/余単位/三角恒等式) と `karpal_core::Adjunction` との接続を文書化します。これは境界について正直であり、完全であると装ったスタブではありません。

## 米田の補題

任意の前層 `P` と対象 `c` について:

``` text
Nat(Hom(-, c), P)  ≅  P(c)
```

Rust は自然変換を第一級の値として表現できません (対象インデックスについてランク N 多相です — `FreeAp::fold_map` と同じ壁)。そのため全単射はその **計算可能な作用** で公開されます:

``` rust
use karpal_topos::{yoneda_apply, yoneda_extract};

// 前進: x ∈ P(c) が自然変換を誘導する。
// f: Dom → Cod を与えられ、yoneda_apply は restrict(f, x) ∈ P(Dom) を計算する。
let applied = yoneda_apply::<P, C, Dom, Cod>(f, x);

// 逆: c で恒等射上で変換を評価する。
let x = yoneda_extract::<P, C, Cod, _>(id_c, |f| action(f));
```

- `yoneda_apply(f, x)` = `restrict(f, x)` — 誘導された変換の成分。
- `yoneda_extract(id_c, action)` = `action(id_c)` — 生成元を復元。

ラウンドトリップの恒等式は直接テスト可能です: 適用のあとの抽出は `x` を復元します。なぜなら前層の単位律により `restrict(id, x) == x` だからです。


Karpal は Apache-2.0 + CLA でライセンスされています。[GitHub で見る](https://github.com/Industrial-Algebra/Karpal)。
