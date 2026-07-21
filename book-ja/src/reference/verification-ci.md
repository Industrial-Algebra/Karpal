# 検証 CI ワークフロー

このガイドは継続的インテグレーションで `karpal-verify` スタックを使う方法を示します。目的は外部検証の実行を検査・アーカイブ可能にすることです: アーティファクトを生成し、明示的なバックエンドポリシーで計画を実行し、JSON / Markdown サマリーをそれらのアーティファクトの隣に永続化します。

## ワークフローの概要

1.  `AlgebraicSignature` から `ObligationBundle` を構築する。
2.  CI ワークスペースの下に `ArtifactLayout` を選ぶ。
3.  `VerificationSession` または `verify_bundle_with_ci_outputs(...)` を実行する。
4.  生成された SMT / Lean アーティファクトとレポートファイルを CI アーティファクトとして公開する。
5.  `VerificationReport` とインポートされた証明書を明示的な信頼境界でレビューする。

## ディレクトリレイアウト

`karpal-verify` は予測可能なディスク上レイアウトを使います。`target/karpal-verify` のようなルートを与えると:

``` text
target/karpal-verify/
├── smt/
│   ├── associativity.smt2
│   ├── left_identity.smt2
│   └── right_identity.smt2
├── lean/
│   ├── KarpalVerify.lean
│   └── KarpalVerify.manifest.json
├── lakefile.lean
├── lean-toolchain
├── verification-report.json
├── verification-report.md
└── verification-report.lean-diagnostics.json
```

このレイアウトは CI で有用です — 単一のディレクトリを後の検査のためのアーティファクトバンドルとして添付できるからです。

## ワンショットヘルパー

単純な CI ジョブでは、最も簡単なエントリポイントは `verify_bundle_with_ci_outputs(...)` です:

``` rust
use karpal_verify::{
    verify_bundle_with_ci_outputs, AlgebraicSignature, ArtifactLayout, DryRunner,
    LeanConfig, ObligationBundle, Origin, SmtConfig, Sort,
};

let sig = AlgebraicSignature::monoid(Sort::Int, "combine", "e");
let bundle = ObligationBundle::monoid(
    "sum_monoid",
    Origin::new("karpal-core", "Monoid for Sum<i32>"),
    &sig,
);

let output = verify_bundle_with_ci_outputs(
    &bundle,
    &ArtifactLayout::new("target/karpal-verify"),
    "KarpalVerify",
    &SmtConfig::default(),
    &LeanConfig::default(),
    &DryRunner,
).expect("verification run should succeed");

assert!(output.report_files.json_path.ends_with("verification-report.json"));
assert!(output.report_files.markdown_path.ends_with("verification-report.md"));
```

この関数はアーティファクトを構築し、指定されたランナーで計画を実行し、CI 指向のサマリーを生成ファイルの隣に直接書き出します。Lean アーティファクトが存在する場合、出力セットには型付き Lean マニフェストと Lean 診断サイドカーも含まれ、CI システムはソースレベルの証明コンテキストと解析された失敗面の両方をアーカイブできます。

## セッション API

より制御が必要な場合は `VerificationSession` を使います。ソルバーバイナリ、Lean 引数、Lean 実行ドライバ (直接 `lean`、`lake env lean`、`lake build` など)、レポートファイルのステムをカスタマイズできます。

``` rust
use karpal_verify::{
    AlgebraicSignature, ArtifactLayout, LeanConfig, ObligationBundle, Origin,
    SmtConfig, Sort, VerificationSession,
};

let sig = AlgebraicSignature::semiring(Sort::Int, "add", "zero", "mul", "one");
let bundle = ObligationBundle::semiring(
    "wrap_ring",
    Origin::new("karpal-algebra", "Semiring for WrapRing"),
    &sig,
);

let session = VerificationSession::new(
    bundle,
    ArtifactLayout::new("target/verify-semiring"),
    "KarpalVerify",
)
.with_smt_config(SmtConfig::new("z3").with_arg("-smt2"))
.with_lean_config(LeanConfig::new("lean"))
.with_report_stem("ci-summary");
```

### CI でのドライラン検証

ドライランは、すべての CI ジョブに外部ツールをインストールすることなくエクスポートとパス生成を検証したい場合に有用です:

``` rust
let report = session.dry_run_report();
assert_eq!(report.obligation_count(), 6);
assert!(report.obligations.iter().all(|o| o.status().is_some()));
```

`DryRunner` はシェル描画されたコマンドを返すため、プレビュージョブやアーティファクトのスモークテストに適しています。

### CI での実際の実行

CI イメージにソルバーと Lean が含まれる場合:

``` rust
let output = session
    .verify_local_with_ci_outputs()
    .expect("local verification should run");

if !output.report.is_success() {
    panic!("verification failed; inspect generated report artifacts");
}
```

このパスは `LocalProcessRunner`、バックエンド固有の `VerificationPolicy` 解釈、レポートシリアライズを使います。

## バックエンドポリシーの振る舞い

CI は「成功」の意味を推測すべきではありません。`karpal-verify` では、成功は明示的に解釈されます:

| バックエンド | 成功条件          | 理由                                                                            |
|---------|----------------------------|--------------------------------------------------------------------------------|
| SMT     | `ExecutionStatus::Unsat`   | Karpal はオブリゲーションの否定をエクスポートするため、`unsat` は法則が成り立つことを意味する。 |
| Lean    | `ExecutionStatus::Success` | モジュールがプロセス失敗なしに Lean に受理された。                        |

この区別は CI ダッシュボードや失敗トリアージで重要です: ソルバーが `sat` や `unknown` を返す場合を、成功した Lean プロセスと同じように扱うべきではありません。

## レポートファイル

JSON と Markdown 出力は意図的に軽量です:

- **JSON** は CI ボット、アーティファクト収集、後処理に有用です。
- **Markdown** はアップロードされたアーティファクトやジョブサマリーでの人間による検査に有用です。

レポートにはバンドル名、ルートディレクトリ、成功/失敗カウント、オブリゲーションごとの状態、アーティファクトパス、利用可能な場合は証明書サマリーが含まれます。

### スキーマのバージョニング

検証レポート JSON、生成された Lean マニフェスト JSON、Lean 診断サイドカーはそれぞれトップレベルの `schema_version` マーカーを含みます。入れ子の `report_files` メタデータブロックもバージョン管理されます。

現在のスキーマバージョンは `1` です。`1.x` 系列内では既存フィールドは安定していることが期待され、新しいデータはオプションフィールド経由でのみ追加されるべきです。したがってコンシューマは以下のようにすべきです:

- `schema_version == "1"` を受け入れる
- 前方互換性のために未知のオプションフィールドを無視する
- 将来のスキーマバンプを破壊的なパーサ境界として扱う

より完全な互換性ポリシーと移行の期待については [検証スキーマ](verification-schemas.md) を参照してください。

## CI のサンプル構成

典型的な CI パイプラインは外部検証を二つのジョブに分割するかもしれません:

1.  **エクスポート / ドライランジョブ**
    - すべての PR で実行
    - アーティファクトとドライランレポートを生成
    - レビューのためエクスポートファイルを公開
2.  **ソルバー支援検証ジョブ**
    - Z3 / Lean が利用可能な場所で実行
    - ローカル検証を実行
    - 同じアーティファクトディレクトリと最終レポートファイルをアップロード

## 推奨プラクティス

- CI アーティファクトを見つけやすくするため決定論的なアーティファクトルートを使う。
- JSON レポートだけでなくレイアウトルート全体をアーカイブする。
- 複数出力が必要ない限り `DEFAULT_REPORT_STEM` でレポートファイル名を安定させる。
- エクスポータの正確性とは別にインポートされた証明書をレビューする。
- `Certified<...>` を自動的な定理インポートではなく明示的な信頼の引き渡えとして扱う。

SMT と Lean 指向のスニペットを含むエンドツーエンドの例は [検証ワークフローの例](../examples/verification-workflow.md) を参照してください。より高レベルの API 概要については [証明と検証](proof-verification.md) に戻ってください。シリアライズされたスキーマ契約については [検証スキーマ](verification-schemas.md) を参照してください。


Karpal は Apache-2.0 + CLA でライセンスされています。[GitHub で見る](https://github.com/Industrial-Algebra/Karpal)。
