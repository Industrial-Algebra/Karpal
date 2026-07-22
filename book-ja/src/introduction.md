# はじめに

**Karpal** は Industrial Algebra エコシステムのための高階型 (Higher-Kinded Type, HKT) ライブラリです。以下を提供します:

- GAT による HKT エンコーディング (`trait HKT { type Of<T>; }`)
- 完全な関手階層 (Functor → Applicative → Monad、および Alt、Plus、Foldable、Traversable など)
- 代数的型クラス (Semigroup、Monoid、Group、Ring、Field、Lattice、Module、VectorSpace、HeytingAlgebra)
- プロ関手オプティクス (Lens、Prism、Traversal、Fold、Iso など)
- Category/Arrow 階層 (FnA、KleisliF、CokleisliF)
- 自由構成 (Free Monad、Cofree Comonad、Coyoneda、Day 畳み込み、Kan 拡張)
- 帰納スキーム (cata、ana、hylo、para、apo、histo、futu、zygo、chrono)
- 随伴と高度な圏論 (end、coend、dinatural transformation)
- モナド変換子 (ExceptT、WriterT、ReaderT、StateT)
- 代数法則の証拠と証明運搬コード
- 外部検証 (SMT-LIB2、Lean 4、Kani、GPU オブリゲーション)
- ストリング図式とモノイダル圏論
- シューベルト交点型システム
- 2-圏、豊饒圏、バイ圏

すべて `no_std` サポートとプロパティベースの法則検証を備えています。

## なぜ Karpal なのか?

Rust には `Option::map`、`Result::and_then`、`Iterator::collect` があります。これらはうまく機能しますが、場当たり的 (ad-hoc) です。すべてのコンテナがわずかに異なる名前で同じパターンを再発明しており、「マッピングをサポートする任意のコンテナ」や「エフェクトの逐次化をサポートする任意のコンテナ」に対して汎用的な関数を書く方法がありません。

Karpal はこれらのパターンに名前と法則を与え、それらをまたいで抽象化できるようにします。

### 標準の Rust と比べて何が得られるか?

**汎用的トラバーサル。** `traverse` は任意の `Traversable` + `Applicative` のペアに対して機能します。`validate_batch` を一度書けば、`Vec`→`Result`、`HashMap`→`Option`、その他の組み合わせでも動作します:

```rust
// 任意の Traversable コンテナと任意の Applicative エフェクトに対して動作
fn validate_all<C: Traversable, F: Applicative>(items: C::Of<Raw>) -> F::Of<C::Item>
```

**合成可能なレンズ。** 入れ子になった構造体アクセサを手書きする代わりに、レンズを関数のように合成し、第一級の値として受け渡せます:

```rust
let street_lens = address_lens.compose(street_name_lens);
let updated = street_lens.over(company, |s| format!("{} (HQ)", s));
```

**法則保証された抽象化。** すべての `Monad` 実装は、左単位律・右単位律・結合律についてプロパティテストされます。自分の型に `Monad` を実装して間違えた場合、ユーザーの前にテストスイートがそれを検出します。

## 正直な制限事項

GAT ベースの HKT エンコーディングには現実的な制約があります:
- 高階型推論がない — 型コンストラクタマーカー (`OptionF`、`VecF`) を明示する必要がある
- 異なるカインド署名を持つ型コンストラクタをまたいで抽象化できない
- nightly Rust が必要 (edition 2024 の機能)
- メソッドチェーン (`some.fmap(...)`) ではなく Static Land スタイル (`OptionF::fmap(...)`) となる

これらは、ネイティブの HKT サポートを持たない言語で HKT をエンコードすることに固有の制約です。Karpal は GAT エンコーディングを採用しています。依存関係ゼロであり、Rust 1.65 から安定しており、proc-macro の魔法を必要としないからです。

## ライセンス

Apache-2.0 + CLA。詳細は [CONTRIBUTING.md](https://github.com/Industrial-Algebra/Karpal/blob/develop/CONTRIBUTING.md) を参照してください。
