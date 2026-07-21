# Karpal

> Rust のための高階型と代数構造

[はじめる](./getting-started.md) | [リファレンスを見る](./reference/functor-family.md) | [GitHub](https://github.com/Industrial-Algebra/Karpal)

---

## 機能

### 型安全な抽象化

GAT による HKT エンコーディングにより、`Option`、`Result`、`Vec`、およびマッピング・逐次化・畳み込みをサポートする任意のコンテナについて汎用的な関数を書けます。

### 完全な階層

Functor から Monad、Alt から Alternative、Foldable、Traversable、Comonad、および反変双対 — すべてプロパティベースの法則検証付き。

### プロ関手オプティクス

プロ関手階層で駆動される Lens、Prism、合成。再利用可能で第一級のフィールドアクセサとパターンマッチャを構築。

### 快適なマクロ

`do_!` は入れ子になった `.and_then()` 連鎖を平坦化します。`ado_!` は独立した計算を組み合わせます。どちらも任意の Monad や Applicative で動作します。

### 証明と検証

`karpal-proof` は法則証拠と精密化型を提供し、`karpal-verify` は明示的な信頼境界付きでオブリゲーションを SMT と Lean にエクスポートします。

## クイック例

`do_!` で入れ子になったエラー処理を平坦化:

```rust
use karpal_std::prelude::*;

// do_! なし — 各ステップで右にドリフト
fn process(input: &str) -> Option<String> {
    parse_id(input).and_then(|id| {
        lookup_user(id).and_then(|user| {
            check_permissions(&user).and_then(|role| {
                Some(format!("{} logged in as {:?}", user.name, role))
            })
        })
    })
}

// do_! あり — 上から下へ読める
fn process(input: &str) -> Option<String> {
    do_! { OptionF;
        id = parse_id(input);
        user = lookup_user(id);
        role = check_permissions(&user);
        Some(format!("{} logged in as {:?}", user.name, role))
    }
}
```

## ワークスペース

| クレート               | 説明                                                                               |
|---------------------|-------------------------------------------------------------------------------------------|
| `karpal-core`       | HKT エンコーディング、関手階層、Semigroup、Monoid、マクロ                                |
| `karpal-profunctor` | Profunctor、Strong、Choice、FnP                                                           |
| `karpal-optics`     | プロ関手オプティクス: Lens、Prism、合成                                               |
| `karpal-arrow`      | Arrow 階層: Category、Arrow、ArrowChoice、Kleisli、Cokleisli                         |
| `karpal-free`       | 自由構成: Coyoneda、Yoneda、Free Monad、Cofree Comonad                          |
| `karpal-recursion`  | 帰納スキーム: Fix、cata、ana、hylo、para、histo、chrono                              |
| `karpal-algebra`    | 抽象代数: Group、Ring、Field、Lattice、Module、VectorSpace                        |
| `karpal-effect`     | モナド変換子と static-bound 関手階層                                     |
| `karpal-proof`      | 法則証拠、精密化型、書き換え証拠、derive ベースの法則チェック            |
| `karpal-verify`     | 外部検証ブリッジ: オブリゲーション、エクスポータ、ランナー、報告、信頼モデル |
| `karpal-diagram`    | モノイダル圏とストリング図式                                                   |
| `karpal-schubert-types` | シューベルト交点型                                                           |
| `karpal-higher`     | 2-圏、豊饒圏、バイ圏                                           |
| `karpal-topos`      | 前層、篩、部分対象分類子、グロタンディーク位相、層 |
| `karpal-std`        | 標準プレリュード再エクスポート                                                               |

`karpal-core`、`karpal-profunctor`、`karpal-arrow`、`karpal-free`、`karpal-recursion`、`karpal-algebra`、`karpal-effect`、`karpal-proof`、および `karpal-verify` のモデリング/エクスポート部分は、オプションの `std`/`alloc` フィーチャーゲート付きで `no_std` 互換です。

## ドキュメントマップ

| 目的                                                | どこから始める                                                                |
|-----------------------------------------------------|-------------------------------------------------------------------------------|
| コア HKT とトレイト階層                        | [はじめての利用](./getting-started.md) と [アーキテクチャ](./architecture-full.md) |
| 詳細な型クラス API                             | [リファレンスページ](./reference/functor-family.md)                              |
| `karpal-proof` 法則証拠と精密化型   | [証明と検証](./reference/proof-verification.md)                     |
| `karpal-verify` エクスポータ、ランナー、信頼モデル | [証明と検証](./reference/proof-verification.md)                     |
| CI アーティファクト/レポートワークフロー                         | [検証 CI ワークフロー](./reference/verification-ci.md)                    |
| シリアライズされたアーティファクトスキーマと互換性       | [検証スキーマ](./reference/verification-schemas.md)                   |
| エンドツーエンド検証のチュートリアル                 | [検証ワークフロー](./examples/verification-workflow.md)                  |
| 検証済み証拠のドメイン API へのインポート        | [検証済みドメイン API](./examples/verified-domain-api.md)                      |

---

Karpal は Apache-2.0 + CLA でライセンスされています。[GitHub で見る](https://github.com/Industrial-Algebra/Karpal)。
