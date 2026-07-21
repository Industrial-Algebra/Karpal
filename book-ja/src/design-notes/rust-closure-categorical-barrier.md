# 圏論的障壁としての Rust クロージャトレイト

**状態:** 設計理論 — 未解決問題  
**日付:** 2026-07-01  
**関連:** イシュー [#95](https://github.com/Industrial-Algebra/Karpal/issues/95)、[#98](https://github.com/Industrial-Algebra/Karpal/issues/98)  
**調査:** [FreeAp fold_map](./freeap-fold-map-exploration.md)、[ReaderTF/StateTF ApplicativeSt](#98)

## 要旨

Rust のクロージャトレイト階層 (`FnOnce` → `FnMut` → `Fn`) は、圏論に対応物のない意味的区別をエンコードします: *計算が何回呼び出されうるか*。ほとんどの圏論的構成要素は遅延設定 (Haskell や数学的 Set) で定義され、そこでは値 `A → B` はコストなく繰り返し消費できます。Rust の正格 Call-by-value 意味論と所有権の組み合わせが、この区別をいくつかの自然な圏論的エンコーディングをブロックする **第一級の型システム障壁** にします。

この文書は正確なメカニズムを特定し、ブロックされた構成をカタログ化し、将来の言語機能がどのような脱出口を提供しうるかを概説します。

## メカニズム

### FnOnce 対 Fn: 所有権の障壁

圏論において、関数 `f: A → B` は値です。ゼロ回、一回、あるいは何度も適用できます。「何回」という概念は存在しません。

Rust において、クロージャリテラル `|x| body` は捕獲した環境がどう使われるかによって分類されます:

| トレイト | 捕獲方法 | 呼び出し可能 | シグネチャ |
|-------|------------|----------|-----------|
| `FnOnce` | 値 (消費) | 一回 | `call_once(self, args)` |
| `FnMut` | 可変参照 | 複数回 | `call_mut(&mut self, args)` |
| `Fn` | 不変参照 | 複数回 | `call(&self, args)` |

重要な摩擦: クロージャが所有する値 `a: A` を move で捕獲する場合、それは `FnOnce` です — 値は呼び出し時に消費され、再現できません。クロージャが `Fn` になるには、`a` は `Clone` でなければならず、各呼び出しで `a.clone()` が新しい値を生成できる必要があります。

つまり:

```
move |x| consume(a, x)  ⟹  FnOnce  (a を move で捕獲、消費する)
move |x| a.clone()      ⟹  Fn      (a を move で捕獲、複製する)
```

### なぜこれが圏論的に重要か

多くの圏論的構成は、値が複数回生成される必要がある文脈で *`G::Of<A>` 型の値を生成する* ことを伴います。Haskell (遅延) では、値の生成にコストはありません — サンクです。Rust (正格) では、値の生成はその材料を消費し、再現には複製か `Rc`/`Arc` 経由の共有が必要です。

## ブロックされた構成

### 1. FreeAp `fold_map`: 存在型を通じた自然変換

標準的なシグネチャ:

```haskell
foldMap :: Applicative g => (forall x. f x -> g x) -> FreeAp f a -> g a
```

Rust ではこれを表現できません。なぜなら:
1. `(forall x. f x -> g x)` は **ランク 2 多相的** な自然変換
2. `Ap (f b) (...)` の中間型 `B` は **存在量化** されている
3. ランク 2 関数を存在型を通じてディスパッチするには単相化が必要 — `dyn Trait` はそれを提供できない
4. 単相化が機能しても、自然変換は *ツリーのすべてのノードで* 呼び出し可能 (複数回の呼び出し) でなければならないが、各呼び出しはエフェクト値を消費する — 再帰的ウォークが `Fn` を必要とするときに `FnOnce` になってしまう

**圏論的ギャップ:** Haskell で `foldMap` が機能するのは:
- `forall x` が第一級 (System F)
- 存在型が第一級
- 遅延評価により呼び出し回数が無関係

Rust はこの三つすべてを欠いています。我々が探った四つの代替エンコーディング (汎用ノードトレイト、チャーチエンコーディング、再帰的単相、リストベース) はそれぞれ、同じ根本的障壁の異なる副理由で失敗します。

### 2. ReaderTF/StateTF `ApplicativeSt`: クロージャ文脈での pure

問題:

```rust
// ReaderTF::Of<A> = Box<dyn Fn(E) -> M::Of<A>>

impl ApplicativeSt for ReaderTF<E, M> {
    fn pure_st<A: 'static>(a: A) -> Box<dyn Fn(E) -> M::Of<A>> {
        // クロージャの毎回の呼び出しで M::Of<A> を生成しなければならない。
        // しかし M::pure_st(a) は a を消費する。最初の呼び出し後、a はなくなっている。
        //
        // A: Clone がなければ、クロージャは FnOnce にしかなれない。
    }
}
```

**圏論的ギャップ:** 圏論では、`pure: a → Reader e a` は自然変換です。結果の `Reader e a` は値です。異なる環境に適用するのは自由です — 単なる関数適用です。Rust では、「結果」自体が関数 (`Box<dyn Fn>`) であり、毎回の呼び出しで *新しい* `M::Of<A>` を生成しなければなりません。`pure` は `a` を消費するため、最初の呼び出ししか成功しません。

同じことが `StateTF` にも当てはまります: `pure(a)(s) = M::pure_st((s, a))` は `a` を消費し、クロージャを `FnOnce` にします。

### 3. その他の潜在的な問題

同じパターンが以下にも現れるでしょう:

- **ContT (継続モナド変換子):** `pure_st(a) = |k| k(a)` — `k` を同じ継続で複数回呼び出すには `a` が複製可能でなければならない
- **汎用インタプリタを持つ任意の自由構成:** インタプリタはすべてのノードで適用され、毎回文脈データを消費する
- **関数値の末尾を持つ Cofree コモナド:** 繰り返し抽出時の同様のクロージャ捕獲の問題

## 共通の糸

これらすべての失敗は構造的性質を共有します:

```
圏論的文脈:     "G<A> を一度生成し、それを返す"
Rust の表現:     "各呼び出しで G<A> を生成する Fn クロージャを返す"
衝突:                Fn は再現性を要求するが、生成は消費する。
```

これは圏論的余代数/代数の区別にマップします:
- **余代数的:** `A → G<A>` (一度生成、観察/消費)
- **代数的:** `(E → G<A>)` (要求に応じて生成、潜在的に複数回)

Rust は余代数的バージョン (`FnOnce`) を表現できますが、`Clone` なしでは代数的バージョン (`Fn`) を表現できません。ほとんどの圏論的構成要素は代数的アクセス (値は繰り返し観察できる) を暗黙に仮定します。

## なぜ現在の Rust で修正できないか

Rust プロジェクトは役立ついくつかの道を探りましたが、安定化に近いものはありません:

| 機能 | 状態 | 役立つか? |
|---------|--------|---------------|
| `impl for<X> Fn(X) -> Y` (ランク N クロージャ) | 提案されていない | FreeAp に役立つが、Fn/FnOnce には无关 |
| 存在型 (`exists X. ...`) | ロードマップにない | FreeAp に役立つ |
| `FnOnce` → `Fn` アップキャスト (Clone 付き) | トレイトシステムにない | 一部のケースを橋渡しできる |
| 遅延評価 / サンク | 却下 | 根本原因を解決する |
| トレイトオブジェクトの `for<'a>` (ライフタイム境界付き dyn) | GAT 経由で部分的に利用可能 | 既に ContravariantLt (#93) で使用 |

## 探り使用した脱出口

1. **ライフタイムパラメータ化 GAT** (`type Of<'a, T>`): ContravariantLt (#93) で使用。`'static` 制約を解決するが、`Fn`/`FnOnce` の問題は解決しない。

2. **ブランケット実装** (`impl<F: Functor> FunctorSt for F`): St 階層 (#97) で使用。並行する型クラスファミリーを橋渡しするが、根本的な表現問題は解決しない。

3. **より強い境界を持つスタンドアロン関数:** `reader_t_pure`/`state_t_pure` (#98) で使用。関数は毎回の呼び出しで複製するため `Clone` を要求する。波及効果 (#98 の調査) のためトレイトはそれを要求しない。これは実用的な妥協。

4. **結び目作りの代わりの反復収束:** `loop_fixpoint` (#94) で使用。Haskell の遅延 `loop` を、フィードバックが安定したときに終了する反復フィックスポイントで置き換える。

5. **正直な削除:** Comonad としての `OptionF` (#92) と `CokleisliF<OptionF>` (#92)。完全性が侵害されるため数学的に存在できない実装を削除。

## 代替エンコーディングが現れた場合

この文書は、Rust の型システムが — HKT、ランク N クロージャ、第一級存在型、`for<X>` 量指定子のいずれによって — 進化したときに、これらの構成を再検討できるように存在します。注目すべき具体的なもの:

### 将来の機能: `for<X>` 量化クロージャ

```rust
// 仮想的: トレイトオブジェクトを通じたランク N クロージャ
trait NatTrans<F: HKT, G: HKT> {
    fn transform<X>(fx: F::Of<X>) -> G::Of<X>
    where for<X>  // 注意: 仮想的な構文
}

// すると fold_map は:
impl<F: HKT, A> FreeAp<F, A> {
    fn fold_map<G: Applicative>(
        self,
        nt: &dyn for<X> Fn(F::Of<X>) -> G::Of<X>,  // 仮想的
    ) -> G::Of<A>
}
```

これには `dyn` を通じたランク N クロージャのサポートが必要です — トレイトシステムへの重要な拡張。

### 将来の機能: 第一級存在型

```rust
// 仮想的: 存在型
enum FreeAp<F: HKT, A> {
    Pure(A),
    Ap(exists B. F::Of<B>, FreeAp<F, B -> A>),  // 仮想的
}
```

これによりパターンマッチ時に存在型 `B` を復元でき、`fold_map` の単相化が可能になります。

### 将来の機能: 遅延/サンク評価

```rust
// 仮想的: GAT ベースの遅延値
trait Lazy {
    type Of<'a, T> = ???;  // サンク的な表現
}
```

Rust が遅延評価プリミティブを得れば、多くの圏論的構成において `Fn`/`FnOnce` の区別は無関係になります — 値の生成がそれを消費せず、`pure` はデフォルトで `Fn` になるでしょう。

## 他の Karpal 設計文書との関係

- [FreeAp fold_map の探求](./freeap-fold-map-exploration.md) — 四つの代替エンコーディングの詳細な調査
- [反変ライフタイム境界](https://github.com/Industrial-Algebra/Karpal/blob/develop/docs/dev/contravariant-lifetime-bounds.md) — `'static`/`Box<dyn Fn>` の制限とライフタイム認識 GAT の回避策

## 参考文献

- Rust クロージャトレイト階層: [`FnOnce`](https://doc.rust-lang.org/std/ops/trait.FnOnce.html)、[`FnMut`](https://doc.rust-lang.org/std/ops/trait.FnMut.html)、[`Fn`](https://doc.rust-lang.org/std/ops/trait.Fn.html)
- GAT 安定化: [RFC 1598](https://rust-lang.github.io/rfcs/1598-generic_associated_types.html)
- `for<>` ライフタイム構文: [リファレンス](https://doc.rust-lang.org/reference/trait-bounds.html#higher-ranked-trait-bounds)
