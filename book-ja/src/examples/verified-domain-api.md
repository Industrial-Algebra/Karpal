# 検証済みドメイン API

この例は `karpal-proof` と `karpal-verify` が API 境界でどう組み合わさるかを示します: ドメイン型は内部的に `Proven<P, T>` を受け取りますが、外部で生成された証拠はまず `Certified<B, P, T>` と明示的な信頼の引き渡えを通過しなければなりません。

## ドメインの目標

ドメインが結合演算が結合的であることが知られている値だけを扱いたいとします。生の `T` を受け取る代わりに、`Proven<IsAssociative, T>` を要求できます。

``` rust
use karpal_proof::{IsAssociative, Proven};

#[derive(Debug, Clone)]
struct VerifiedAccumulator<T> {
    inner: Proven<IsAssociative, T>,
}

impl<T> VerifiedAccumulator<T> {
    fn new(inner: Proven<IsAssociative, T>) -> Self {
        Self { inner }
    }
}
```

これが `karpal-proof` スタイルです: ドメイン API は要求される法則を型レベルの事前条件として記述します。

## Rust ネイティブのエントリポイント

値がすでに信頼できる Rust 側の証拠コンストラクタから来ている場合、API は単純です:

``` rust
use karpal_proof::Proven;

let proven = Proven::from_semigroup(5i32);
let acc = VerifiedAccumulator::new(proven);
```

ここでは値は Rust ネイティブの証拠を経由して入ります。外部の信頼境界は関与しません。

## 外部証明書のエントリポイント

> **⚠️ 信頼境界の警告**
>
> 以下に示す `unsafe` 変換は暗号論的または形式的な保証では **ありません**。`Certificate` は署名・チェックサム・リプレイ保護のない任意の文字列を運びます。`into_proven()` の呼び出しは外部の来歴を完全に消去します。このコードにアクセスできる人は誰でも `Proven` 値を偽造できます。
>
> これは **監査された信頼境界** であり、セキュリティ機構ではありません。`unsafe` キーワードにより、コードレビューが外部の証拠が受理されるすべての箇所を確実に検出します。完全な設計理論は [フェーズ 12 信頼モデル](https://github.com/Industrial-Algebra/Karpal/blob/develop/docs/dev/phase-12-trust-model.md) を参照してください。

次に、結合性が外部の証明器によって確立された場合を考えます。`karpal-verify` はその証拠が暗黙に `Proven<...>` になることを意図的に防ぎます。

``` rust
use karpal_proof::{IsAssociative, Proven};
use karpal_verify::{Certificate, Certified, SmtCertificate};

let cert = Certificate::new("smtlib2", "sum_assoc", "z3:unsat");
let certified = unsafe {
    Certified::<SmtCertificate, IsAssociative, i32>::assume(5, cert)
};

// まだ Proven<...> 値ではない。
let proven: Proven<IsAssociative, i32> = unsafe { certified.into_proven() };
let acc = VerifiedAccumulator::new(proven);
```

二つの明示的な `unsafe` ステップが要点です: コードレビューがインポートされた信頼境界を見つけて監査できます。

## 境界の設計パターン

有用なパターンは、unsafe 変換を一つの狭い境界関数に閉じ込め、他では安全な API のみを公開することです:

``` rust
use karpal_proof::{IsAssociative, Proven};
use karpal_verify::{Certified, SmtCertificate};

fn import_associative_i32(
    certified: Certified<SmtCertificate, IsAssociative, i32>,
) -> VerifiedAccumulator<i32> {
    let proven = unsafe { certified.into_proven() };
    VerifiedAccumulator::new(proven)
}
```

これによりインポートされた証明の決定を明示的かつ局所化できます。

## なぜこれが重要か

- **`karpal-proof`** はドメインに豊かな法則認識 API を与えます。
- **`karpal-verify`** は外部の証明器が信頼の来歴を消去することなくそれらの API に供給できるようにします。
- **組み合わせ** により、公開 API を原則的に保ちながら、プロジェクト認識実行・構造化診断・アーカイブされた検証アーティファクトを含む SMT や完全な Lean ワークフローと統合できます。

## 推奨される使い方

1.  内部ドメイン API を `Proven<P, T>` と精密化ラッパーを中心に設計する。
2.  `karpal-verify` で外部のオブリゲーションをモデル化し discharge する。
3.  証明書を `Certified<B, P, T>` としてインポートする。
4.  小さく監査された境界レイヤーでのみ `Proven<P, T>` に変換する。

より広範なエクスポート/実行ワークフローについては [検証ワークフロー](verification-workflow.md) を参照してください。API の概要については [証明と検証](../reference/proof-verification.md) を参照してください。CI/レポート/アーカイブの詳細については [検証 CI ワークフロー](../reference/verification-ci.md)、シリアライズされた互換性の詳細については [検証スキーマ](../reference/verification-schemas.md) を参照してください。


Karpal は Apache-2.0 + CLA でライセンスされています。[GitHub で見る](https://github.com/Industrial-Algebra/Karpal)。
