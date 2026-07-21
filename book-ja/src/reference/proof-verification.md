# 証明と検証

Karpal は法則について推論するための相補的な層を持つようになりました。`karpal-proof` は Rust 内の証拠、精密化型、derive ベースの法則チェックを提供します。`karpal-verify` はその物語を外側に拡張し、オブリゲーション IR、SMT-LIB2 と Lean 4 のエクスポータ、オプションの amari-flynn 統計検証フック、アーティファクト生成、実行計画、報告、三層バンドルサマリー、インポートされた証明書のための明示的な信頼境界を提供します。これらの API が一緒になって Karpal の完全な外部検証基盤を覆います。

## 概要

| クレート                 | 役割                                           | 典型的な使い方                                                                                                              |
|-----------------------|------------------------------------------------|--------------------------------------------------------------------------------------------------------------------------|
| `karpal-proof`        | 内部の法則証拠と精密化証拠 | Rust 内で値が性質を満たすことが知られていることをエンコード                                                           |
| `karpal-proof-derive` | derive 駆動の法則検証ヘルパ         | 型の代数法則をチェックするテストを生成                                                                  |
| `karpal-verify`       | 外部検証ブリッジ                   | オブリゲーションを外部の証明器にエクスポートし、amari-flynn で稀有事象チェックをブリッジし、証明書を明示的にインポート |

## クレートマップ

| クレート           | 焦点                                                                                                |
|-----------------|------------------------------------------------------------------------------------------------------|
| `karpal-proof`  | 法則証拠、書き換え証拠、精密化型、derive ベースの法則検証                 |
| `karpal-verify` | オブリゲーション IR、エクスポータ、実行/報告、オーケストレーション、明示的なインポート信頼境界 |

## `karpal-proof` 層

`karpal-proof` は法則の証拠を値とファントムマーカーとしてモデル化します。中核のアイデアは、結合律やモノイド構造のような性質を、コンパイラが第一原理から証明したと見せかけずに型システムに反映できるということです。


### Proven\<P, T\>

性質マーカー `P` の証拠と対になった値 `T`。


``` rust
use karpal_proof::{IsMonoid, Proven};

let checked: Proven<IsMonoid, i32> = Proven::from_monoid(5);
let value: i32 = checked.into_inner();
```

`IsAssociative`、`IsMonoid`、`IsGroup`、`IsSemiring` のような性質マーカーにより、下流の API は生のトレイト境界ではなく証拠を要求できます。


### 精密化型

より強いドメイン不変量のための小さな実行時チェック付きラッパ。


``` rust
use karpal_proof::{NonEmpty, Positive};

let xs = NonEmpty::try_new(vec![1, 2, 3]).expect("vector is non-empty");
let p = Positive::new(42).expect("value is positive");
```

外部検証をしていなくてもこれらのラッパは有用です: 構築後に不正な状態を表現不可能にします。


### 書き換え証拠

代数的書き換えステップの合成可能な証拠。


書き換えは結合律・可換律・単位除去・分配律・逆元相殺のような法則ガイド付き変換を捉えます。正規化、記号的簡略化、Rust 内の証明指向 API に適しています。


### derive ベースの法則チェック

`karpal-proof-derive` は `VerifySemigroup`、`VerifyMonoid`、`VerifyGroup`、`VerifySemiring`、`VerifyLattice` のようなマクロを提供します。これらの derive ヘルパは型の関連する代数法則を行使するテストを生成します。

``` rust
use karpal_proof::VerifyMonoid;

#[derive(Clone, Debug, PartialEq, Eq, VerifyMonoid)]
struct SumI32(i32);
```

これは Rust ネイティブのテスト指向ワークフローです: 継続的チェックと回帰防止に優れていますが、定理証明器の結果をインポートすることとは異なります。

## `karpal-verify` 層

`karpal-verify` は外部検証のための Karpal のブリッジです。*モデリング*、*エクスポート*、*実行*、*信頼* を意図的に分離し、各ステップを検査可能に保ちます。

### オブリゲーション IR

中核の中間表現はバックエンドに依存せず、特定の証明器構文に commit することなく代数法則を記述できます。

``` rust
use karpal_verify::{Obligation, Origin, Sort};

let assoc = Obligation::associativity(
    "sum_assoc",
    Origin::new("karpal-algebra", "Semigroup for Sum<i32>"),
    Sort::Int,
    "combine",
);
```

IR には以下が含まれます:

- 名前付き証明目標のための `Obligation`
- 来歴のための `Origin`
- シグネチャと式のための `Declaration`、`Sort`、`Term`
- 分類メタデータのための `VerificationTier` と `ProofDialect`

### 代数シグネチャとバンドル

`AlgebraicSignature` は `combine`、`identity`、`inverse`、`add`、`mul`、`meet`、`join` のような意味的役割を登録します。`ObligationBundle` は半群・モノイド・群・半環・束のような構造の関連法則をまとめます。

``` rust
use karpal_verify::{AlgebraicSignature, ObligationBundle, Origin, Sort};

let sig = AlgebraicSignature::group(Sort::Int, "combine", "e", "inv");
let bundle = ObligationBundle::group(
    "sum_group",
    Origin::new("karpal-algebra", "Group for i32"),
    &sig,
);
assert_eq!(bundle.obligations().len(), 5);
```

### エクスポータ

同じオブリゲーションバンドルを異なるバックエンドにエクスポートできます:

- **SMT-LIB2** は `SmtLib2` と `export_smt_bundle(...)` 経由
- **Lean 4** は `Lean4`、`export_lean_bundle(...)`、構造化 Lean エクスポート API 経由

``` rust
use karpal_verify::{export_smt_bundle, export_lean_bundle};

let smt_scripts = export_smt_bundle(&bundle);
let lean_module = export_lean_bundle("KarpalVerify", &bundle);
```

### Lean 統合

Lean ブリッジは単なるテキストエクスポート以上です。構造化 Lean メタデータは定理の同一性、宣言スパン、モジュールインポート、シンボルエイリアス、プロジェクト/パッケージ情報、レポート相互リンクを追跡します。これにより Karpal はエクスポートされたオブリゲーション、生成された Lean ソース、CI アーティファクト、解析された Lean 診断の間に安定した接続を保てます。

- `LeanPrelude`、`LeanImport`、`LeanAlias` 経由の **プレリュード/インポートブリッジング**
- `LeanTheorem` と `LeanExport` による **構造化定理メタデータ**
- `LeanProject` と生成された `lakefile.lean` および `lean-toolchain` による **プロジェクトスキャフォールディング**
- `LeanDriver::LakeEnv` と `LeanDriver::LakeBuild` による **プロジェクト認識実行**
- `parse_lean_output(...)` 経由の **解析された診断** (定理名ヒットと行認識フォールバックマッピングを含む)
- スキーマバージョン管理されたレポート JSON、Lean マニフェスト JSON、Lean 診断サイドカー JSON による **CI サイドカーとマニフェスト**

### アーティファクト、計画、実行

`std` フィーチャーで、`karpal-verify` はアーティファクトレイアウトの準備、ファイル書き込み、起動計画の構築、ドライランまたはローカルプロセスとしての実行ができます。

これらのシリアライズされたアーティファクトもスキーマバージョン管理されます: レポート JSON、Lean マニフェスト JSON、Lean 診断サイドカーはすべて `schema_version` マーカーを持ち、CI ツールが互換性のある形式変更と破壊的変更を明示的に検出できます。

| 型                 | 責任                                                     |
|----------------------|--------------------------------------------------------------------|
| `ArtifactLayout`     | 生成された SMT と Lean アーティファクトのディレクトリレイアウト              |
| `ArtifactBatch`      | 検証バッチのレコードと起動計画             |
| `InvocationPlan`     | 実行可能ファイル、引数、作業ディレクトリ、追跡入力ファイル       |
| `DryRunner`          | プロセスを起動せずにシェル描画されたドライラン結果を返す  |
| `LocalProcessRunner` | `std::process::Command` 経由でローカルソルバーまたは Lean コマンドを実行 |

### バックエンド固有の検証ポリシー

`karpal-verify` は明示的なバックエンドポリシーを定義し、成功がすべてのツールで一様に解釈されないようにします:

- **SMT**: 検証成功は否定されたオブリゲーションが `unsat` であることを意味
- **Lean**: 検証成功はプロセスが成功で終了し、解析された Lean 診断がエラーを報告しないことを意味

``` rust
use karpal_verify::{CommandKind, ExecutionStatus, VerificationPolicy};

assert!(VerificationPolicy::for_kind(CommandKind::Smt)
    .accepts(ExecutionStatus::Unsat));
assert!(VerificationPolicy::for_kind(CommandKind::Lean)
    .accepts(ExecutionStatus::Success));
```

SMT 出力解析は `SmtOutput` を通じてより豊かな詳細も記録し、解析された状態、`sat` 後の単純なモデルテキスト、`:reason-unknown` メタデータを含みます。Lean 解析は構造化診断、定理ヒット、位置認識フォールバックマッチングを記録し、Lean がソース位置だけを出力する場合でもレポートが失敗を正しいエクスポートされた定理に添付できます。

### 報告とオーケストレーション

報告層は結果、アーティファクトパス、オプションの証明書を各オブリゲーションに添付します。新しいセッション/オーケストレーション層はビルド → 実行 → 報告のためのより高レベルなワークフローを提供します。

``` rust
use karpal_verify::{
    verify_bundle, AlgebraicSignature, ArtifactLayout, DryRunner, LeanConfig,
    ObligationBundle, Origin, SmtConfig, Sort,
};

let sig = AlgebraicSignature::semigroup(Sort::Int, "combine");
let bundle = ObligationBundle::semigroup(
    "sum_semigroup",
    Origin::new("karpal-core", "Semigroup for Sum<i32>"),
    &sig,
);
let report = verify_bundle(
    &bundle,
    &ArtifactLayout::new("target/karpal-verify"),
    "KarpalVerify",
    &SmtConfig::default(),
    &LeanConfig::default(),
    &DryRunner,
).expect("verification session should succeed");
assert_eq!(report.obligation_count(), 1);
```

`VerificationSession::verify_with_ci_outputs(...)` は加えて、JSON と Markdown サマリーを生成されたアーティファクトの隣に直接書き出し、スキーマバージョン管理された Lean 診断サイドカーと CI レポートファイルへの相互リンクを持つ型付き Lean マニフェストも書き出します。CI 固有の指針、アーティファクトレイアウトの推奨については [検証 CI ワークフロー](verification-ci.md) を、シリアライズされた互換性契約については [検証スキーマ](verification-schemas.md) を参照してください。

## 明示的な信頼境界

外部証明書は暗黙に Rust の証拠証拠にはなりません。インポートされた証拠はまず `Certified<B, P, T>` になります。ここで `B` はバックエンド、`P` は主張された性質、`T` は包まれた値です。`Proven<P, T>` への交差は明示的な `unsafe` アクションのままです。

``` rust
use karpal_proof::{IsAssociative, Proven};
use karpal_verify::{Certificate, Certified, LeanCertificate};

let cert = Certificate::new("lean4", "sum_assoc", "Sum.assoc");
let externally_checked =
    unsafe { Certified::<LeanCertificate, IsAssociative, i32>::assume(1, cert) };
let _: Proven<IsAssociative, i32> = unsafe { externally_checked.into_proven() };
```

この方針により、インポートされた信頼を検索可能・レビュー可能に保ち、Rust トレイトや実行時チェックから直接導出された証拠と区別します。信頼モデルに特化した設計メモは [信頼モデル](https://github.com/Industrial-Algebra/Karpal/blob/develop/docs/dev/phase-12-trust-model.md) を参照してください。

## 推奨ワークフロー

1.  法則を `Obligation` または `ObligationBundle` としてモデル化する。
2.  SMT-LIB2 スクリプトまたは Lean モジュールをエクスポートする。
3.  アーティファクトを書き、起動計画を生成する。
4.  明示的なバックエンドポリシーで実行する。
5.  `VerificationReport` とオプションの CI サマリーを収集する。
6.  外部の証拠は `Certified<...>` 経由でのみインポートする。
7.  注意深く監査された境界でのみ `Proven<...>` に交差する。

チュートリアル形式の例は [検証ワークフロー](../examples/verification-workflow.md) を参照してください。


Karpal は Apache-2.0 + CLA でライセンスされています。[GitHub で見る](https://github.com/Industrial-Algebra/Karpal)。
