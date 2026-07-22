# FreeAp `fold_map`: なぜ Rust では不可能か (そして代わりに何ができるか)

**状態:** 設計理論 — 完結した調査
**日付:** 2026-07-01
**イシュー:** [#95](https://github.com/Industrial-Algebra/Karpal/issues/95)
**発見:** Proserpina ドキュメント批評 (バッチ 4)

## 背景

自由アプリカティブ (`FreeAp`) は圏論と関数型プログラミングにおける標準的な構成です。Haskell ではそのシグネチャは:

```haskell
data FreeAp f a
  = Pure a
  | forall b. Ap (f b) (FreeAp f (b -> a))

foldMap :: Applicative g => (forall x. f x -> g x) -> FreeAp f a -> g a
```

`Ap` コンストラクタの `forall b` は **存在型** です — 各ノードは独自の中間型 `b` を隠します。`foldMap` の `forall x` は **ランク 2 全称** です — 自然変換は *すべての* 型で機能しなければなりません。

この二つの量指定子は一緒になって、Haskell (完全な System F 多相を持つ) では表現しやすいが、この調査が示すように **現在の Rust では根本的に表現不可能** な緊張を作り出します。

## 問題

Karpal の `FreeAp` は存在 `dyn` トレイトを持つ GAT ベース HKT エンコーディングを使います:

```rust
trait FreeApNode<F: HKT + 'static, A: 'static> {
    fn retract_node(self: Box<Self>) -> F::Of<A>
    where
        F: Applicative;
    fn count_effects(&self) -> usize;
}

pub enum FreeAp<F: HKT + 'static, A: 'static> {
    Pure(A),
    Ap(Box<dyn FreeApNode<F, A>>),
}
```

各 `Ap` ノードは中間型 `B` を `dyn FreeApNode<F, A>` の背後で消去します。これは `retract` (F 自身に解釈) には機能します。`F::ap` と `F::pure` がトレイトレベルで利用可能だからです。

しかし `fold_map` は `B` が消去された状態で `nt.transform::<B>()` を呼ぶ必要があります。Rust は `dyn` トレイトオブジェクトを通じて汎用関数を単相化できません。**汎用メソッドと `dyn` ディスパッチは Rust の型システムで相互排他です。**

## 探ったアプローチ

### アプローチ 1: ノードトレイトの汎用メソッド

**アイデア:** `fold_map` を `FreeApNode` の汎用メソッドとして追加。

```rust
trait FreeApNode<F: HKT + 'static, A: 'static> {
    fn fold_map<G: Applicative, NT: NatTrans<F, G>>(&self, nt: &NT) -> G::Of<A>;
}

trait NatTrans<F: HKT, G: HKT> {
    fn transform<B>(fb: F::Of<B>) -> G::Of<B>;
}
```

**結果: ❌ コンパイル不可。**

`fold_map<G, NT>` は汎用メソッドです。汎用メソッドはトレイトを **非 dyn 互換** にします (`dyn FreeApNode` がコンパイル不可)。`dyn` がなければ中間型を消去できず、異種混合ツリーを構築できません。

これが循環の罠です:
- `B` を消去するには `dyn` が必要。
- `nt<B>` をディスパッチするには単相化が必要。
- `dyn` と汎用メソッドは相互排他。

### アプローチ 2: 消去されたインタプリタによるチャーチエンコーディング

**アイデア:** `FreeAp` をそれ自身の fold として表現 — インタプリタを取り結果を生成するクロージャ。

```rust
trait ErasedInterp<F> {
    fn pure_erased(&self, val: Box<dyn Any>) -> Box<dyn Any>;
    fn ap_erased(&self, ff: Box<dyn Any>, fa: Box<dyn Any>) -> Box<dyn Any>;
    fn lift_erased(&self, fb: Box<dyn Any>) -> Box<dyn Any>;
}

pub struct FreeApC<F: 'static, A: 'static> {
    run: Box<dyn FnOnce(&dyn ErasedInterp<F>) -> Box<dyn Any>>,
}
```

**結果: ❌ コンパイル不可。**

インタプリタの `pure_erased` は `Box<dyn Any>` を受け取り `G::Of<A>` を構築しなければなりません。しかし `A` は実行時に消去されており — インタプリタには `G::pure` を呼ぶために `Box<dyn Any>` から具体的な型を復元する方法がありません。

同様に、`ap_erased` は二つの消去されたボックスを受け取るが、アプリカティブ適用を行うための関数/引数の型を決定できません。

**根本原因:** `Box<dyn Any>` はインタプリタが正しく型付けされた結果を構築するのに必要な型を消去します。型消去は型構築と互換性がありません。

### アプローチ 3: 再帰的単相エンコーディング

**アイデア:** すべてのエフェクトを同じ型 `X` に制約し、存在型の必要性を回避。

```rust
pub enum FreeApMono<F: HKT + 'static, X: 'static, A: 'static> {
    Pure(A),
    Ap {
        effect: F::Of<X>,
        kont: Box<FreeApMono<F, X, Box<dyn Fn(X) -> A>>>,
    },
}
```

**結果: ❌ コンパイル不可 — 無限型再帰。**

`Ap` バリアントは `FreeApMono<F, X, Box<dyn Fn(X) -> A>>` を格納し、型チェッカレベルで無限型連鎖を作ります:

```
FreeApMono<F, X, A>
  contains FreeApMono<F, X, Box<dyn Fn(X) -> A>>
    contains FreeApMono<F, X, Box<dyn Fn(X) -> Box<dyn Fn(X) -> A>>>
      contains FreeApMono<F, X, Box<dyn Fn(X) -> Box<dyn Fn(X) -> Box<dyn Fn(X) -> A>>>>
        contains ... (無限)
```

コンパイラエラー:
```
error[E0320]: overflow while adding drop-check rules for `FreeApMono<F, X, A>`
  = note: overflowed on `FreeApMono<F, X, Box<dyn Fn(X) -> Box<dyn Fn(X) -> Box<...>>>>`
```

再帰的アプリカティブ構造 (`Ap(F<B>, FreeAp<F, B->A>)`) は存在型で **必ず** 砕かなければなりません。存在型 (`fold_map` をブロックする) なし、あるいは無限型なしで、再帰的アプリカティブツリーを表現する方法はありません。

### アプローチ 4: 非再帰的リストベースエンコーディング

**アイデア:** エフェクトを `Vec<F::Of<X>>` に平坦化し、純粋関数で結合。アプリカティブは (モナドと異なり) 独立したエフェクトを持つため、単相の場合これは意味的に妥当です。

```rust
pub struct FreeApSeq<F: HKT + 'static, X: 'static, A: 'static> {
    effects: Vec<F::Of<X>>,
    combine: Box<dyn Fn(&[X]) -> A>,
}
```

`fold_map` は: (1) 各エフェクトに NT を適用、(2) `Applicative` 経由で `G` エフェクトを逐次化、(3) 結果に `combine` をマップ。

**結果: ⚠️ 理論的には妥当だが、所有権摩擦にヒット。**

`sequence` 演算 (`Vec<G::Of<X>>` を `G::Of<Vec<X>>` に変える) は `G::ap` を通じてアキュムレータを受け渡す必要があります。アキュムレータクロージャは `Vec<X>` を move で捕獲し `FnOnce` になります — しかし `G::ap` は `Box<dyn Fn>` を必要とし、複数回呼び出し可能でなければなりません。

これは `Rc<RefCell<Vec<X>>>` で回避できますが、ランタイムオーバーヘッドと複雑さを導入し、Karpal のゼロコスト原則に違反します。

**追加のトレードオフ:**
- 単相エフェクトのみサポート (すべて `F::Of<X>`)
- アプリカティブ合成構造を失う (ツリーではなくフラットなリスト)
- `combine: Fn(&[X]) -> A` インターフェースが不格好 (カリー化ではなく位置指定)

## 根本的障壁

四つのアプローチすべてが同じ根本理由で失敗します:

> **Rust の型システムは、存在型 (`forall b. ...`) を通じてディスパッチされるランク N 多相 (`forall x. f x -> g x`) を表現できない。**

型理論の用語で:
- `fold_map` は `forall x` の **否定出現** を必要とする (自然変換が `x` について多相でなければならない)
- `Ap` コンストラクタは `exists b` の **肯定出現** を必要とする (中間型が隠されている)
- Rust のトレイトシステムはランク N 型も第一級存在型もサポートしない — 両方とも `dyn` で近似されるが、`dyn` は単相的 (非汎用) メソッドを必要とする

これはバグでも見落しでもありません。2026 年現在の Rust の型システムの根本的な性質です。この制限は Rust のすべての GAT ベース HKT エンコーディングで共有されます。

## 現在のエンコーディングが提供するもの

| 能力 | 状態 |
|-----------|--------|
| `retract()` — F 自身に解釈 | ✅ 動作 |
| `count_effects()` — エフェクトツリーの静的解析 | ✅ 動作 |
| `fmap()` — 結果上の関手マップ | ✅ 動作 |
| `ap()` — アプリカティブ合成 | ✅ 動作 |
| `fold_map()` — NT 経由で任意の G に解釈 | ❌ 不可能 (根本的) |
| 異種エフェクト型 | ✅ サポート |
| アプリカティブ法則検証 | ✅ 4 つの proptest 法則 |

## 推奨事項

1. **現在のエンコーディングは正しい。** Rust の型システムで可能な最大の能力セットを提供します。

2. **ドキュメントは既に制限を説明し** 回避策を提供しています: `fold_map nt ≡ retract . hoist nt`。

3. **`fold_map` が本当に必要な単相ユースケース** では、ユーザーは `Vec<F::Of<X>>` を構築し対象の `Applicative` で直接逐次化できます。これはアプリケーションコードで単純です:

   ```rust
   let effects: Vec<F::Of<X>> = vec![...];
   let g_effects: Vec<G::Of<X>> = effects.into_iter().map(nt).collect();
   let sequenced = G::sequence(g_effects);  // G: Traversable の場合
   let result = G::fmap(sequenced, combine);
   ```

4. **`FreeApSeq` 型** (アプローチ 4) は、需要が生じれば別クレートやモジュールとして追加できます。それは一般性とゼロコスト保証を犠牲にして単相ケースの `fold_map` を提供するでしょう。

## より広い意義

この調査は Karpal を超えて関連します。自由構成で圏論的抽象化をエンコードしようとする任意の Rust ライブラリは同じ壁にヒットするでしょう。ここの知見は以下に適用されます:

- 汎用インタプリタを持つ自由モナド
- 多相 fold を持つチャーチエンコーディングされたデータ型
- 存在型を通じたランク N 多相を必要とする任意の構成

この制限の文書化は、より広い Rust 圏論エコシステムの参考として機能します。

## 参考文献

- [イシュー #95](https://github.com/Industrial-Algebra/Karpal/issues/95) — 元の報告
- [Proserpina 批評レポート](https://github.com/Industrial-Algebra/Karpal/blob/develop/docs/reviews/batch4-algebra-review.md) — バッチ 4、所見 B6
- `karpal-free/src/free_ap.rs` — 実装
- Haskell `Control.Applicative.Free` — `foldMap` を表現できる参照実装
- [圏論的障壁としての Rust クロージャトレイト](./rust-closure-categorical-barrier.md) — `Fn`/`FnOnce` 摩擦の横断的分析
