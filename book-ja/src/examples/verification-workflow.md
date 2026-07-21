# 検証ワークフロー

この例は現実的な `karpal-verify` ワークフローを歩きます: 代数オブリゲーションのバンドルを定義し、SMT と Lean アーティファクトをエクスポートし、ドライランでコマンドをプレビューし、CI 指向のサマリーを生成し、最後に明示的な信頼境界を通じて外部証明書をインポートします。

## シナリオ

加法的モノイドのように振る舞う型があり、三つのことが欲しいとします:

- その法則の機械可読な記述、
- SMT と Lean のためのエクスポートアーティファクト、そして
- 外部検証から Rust に戻るレビュー可能な経路。

`karpal-verify` スタックはまさにこの流れのために設計されています。

## 1. オブリゲーションバンドルを構築

``` rust
use karpal_std::prelude::*;

let sig = AlgebraicSignature::monoid(Sort::Int, "combine", "e");
let bundle = ObligationBundle::monoid(
    "sum_monoid",
    Origin::new("karpal-core", "Monoid for Sum<i32>"),
    &sig,
);

assert_eq!(bundle.obligations().len(), 3);
```

結果のバンドルは結合律、左単位律、右単位律を含みます。バンドルがすべての下流ステップの共有ソースになります。

## 2. SMT と Lean アーティファクトをエクスポート

``` rust
let smt_scripts = export_smt_bundle(&bundle);
let lean_module = export_lean_bundle("KarpalVerify", &bundle);

assert_eq!(smt_scripts.len(), 3);
assert!(lean_module.contains("namespace KarpalVerify"));
```

この段階ではまだメモリ内のプレーンな文字列です。他のツールと統合したり、より高レベルなエクスポートパイプラインを構築したりするのに有用です。

## 3. アーティファクトを書き出し、起動計画を検査

ルートレイアウトを選ぶと、`karpal-verify` はファイルとそれらを実行するために必要なコマンド計画を具体化できます。

``` rust
let layout = ArtifactLayout::new("target/karpal-verify-example");
let batch = dry_run_bundle_artifacts(
    &bundle,
    &layout,
    "KarpalVerify",
    &SmtConfig::default(),
    &LeanConfig::default(),
);

for plan in &batch.plans {
    println!("{}", plan.render_shell());
}
```

ドライランバッチは CI を配線する際に特に有用です。ソルバーバイナリが利用可能でなくても、パスとエクスポータ出力を検証できるからです。バッチは構造化 Lean エクスポートメタデータ、生成された Lean プロジェクトデータ、後で生成モジュールの隣にシリアライズされる型付き Lean マニフェストモデルも運びます。

## 4. ビルド → 実行 → 報告をオーケストレーション

オーケストレーション層は低レベルの部品を一つのまとまった流れに包みます。例えば、ドライラン CI スタイルのセッションです:

``` rust
let output = verify_bundle_with_ci_outputs(
    &bundle,
    &ArtifactLayout::new("target/karpal-verify-example"),
    "KarpalVerify",
    &SmtConfig::default(),
    &LeanConfig::default(),
    &DryRunner,
).expect("verification session should succeed");

assert_eq!(output.report.obligation_count(), 3);
assert!(output.report_files.json_path.ends_with("verification-report.json"));
assert!(output.report_files.markdown_path.ends_with("verification-report.md"));
```

この一回の呼び出しで以下をすべて行います:

- SMT と Lean アーティファクトを書き出す、
- 起動計画を作成する、
- 指定されたランナーでそれらを実行する、
- `VerificationReport` を構築する、
- 生成されたアーティファクトの隣に JSON / Markdown サマリーを書き出す、
- スキーマバージョン管理された Lean 診断サイドカーを書き出す、
- スキーマバージョン管理された Lean マニフェストをそれらのレポートファイルに相互リンクする。

## 5. バックエンドの意味論を理解する

同じ「成功」という言葉がバックエンドによって異なる意味を持ちます:

``` rust
assert!(VerificationPolicy::for_kind(CommandKind::Smt)
    .accepts(ExecutionStatus::Unsat));
assert!(VerificationPolicy::for_kind(CommandKind::Lean)
    .accepts(ExecutionStatus::Success));
```

SMT バックエンドでは、Karpal は法則の否定をエクスポートするため、`unsat` が成功ケースです。Lean では、成功は受理されたモジュールとエラーを報告しない解析された診断です。Lean 診断はエクスポートされた定理の同一性にマップバックされ、診断メッセージが直接定理を名指さない場合はソース行スパンをフォールバックとして使います。

## 6. より制御のためにセッションビルダーを使う

ツール名、追加引数、カスタムレポート名を設定する必要がある場合は、`VerificationSession` を直接使います:

``` rust
let session = VerificationSession::new(
    bundle.clone(),
    ArtifactLayout::new("target/karpal-verify-example-2"),
    "KarpalVerify",
)
.with_smt_config(SmtConfig::new("z3").with_arg("-smt2"))
.with_lean_config(
    LeanConfig::new("lean")
        .with_driver(LeanDriver::LakeBuild)
)
.with_report_stem("nightly-summary");

let dry_report = session.dry_run_report();
assert!(dry_report.obligations.iter().all(|o| o.status().is_some()));
```

## 7. 外部の証拠を明示的にインポート

最後のステップは意図的に明示的です。外部の証拠はまず証明書と `Certified<...>` ラッパになり、`Proven<...>` 値にはなりません。

``` rust
use karpal_proof::{IsAssociative, Proven};
use karpal_verify::{Certificate, Certified, SmtCertificate};

let cert = Certificate::new("smtlib2", "sum_assoc", "z3:unsat");
let imported =
    unsafe { Certified::<SmtCertificate, IsAssociative, i32>::assume(1, cert) };
let _: Proven<IsAssociative, i32> = unsafe { imported.into_proven() };
```

これが `karpal-verify` の意図的な信頼の引き渡えです: 外部の証拠は有用ですが、Rust ネイティブの証拠と暗黙に混同されることはありません。

## 次にどこへ

- 完全な API 概要は [証明と検証](../reference/proof-verification.md)。
- CI に焦点を当てたレイアウトと報告の指針は [検証 CI ワークフロー](../reference/verification-ci.md)。
- レポート/マニフェスト/サイドカーの互換性の詳細は [検証スキーマ](../reference/verification-schemas.md)。
- インポートされた証明の境界の設計メモは [信頼モデル](https://github.com/Industrial-Algebra/Karpal/blob/develop/docs/dev/phase-12-trust-model.md)。


Karpal は Apache-2.0 + CLA でライセンスされています。[GitHub で見る](https://github.com/Industrial-Algebra/Karpal)。
