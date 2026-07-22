# 検証スキーマ

このページは `karpal-verify` が生成するシリアライズされたアーティファクト形式と、その `schema_version` マーカーに関する互換性契約を文書化します。

## バージョン付きアーティファクト

- 検証レポート JSON
- Lean マニフェスト JSON
- Lean 診断サイドカー JSON
- レポートおよびマニフェスト JSON に埋め込まれた入れ子の `report_files` メタデータブロック

## 現在のバージョン

現在の公開スキーマバージョンは `1` です。Rust コードでは `VERIFICATION_REPORT_SCHEMA_VERSION`、`LEAN_MANIFEST_SCHEMA_VERSION`、`VERIFICATION_SIDECAR_SCHEMA_VERSION` などの定数として公開されています。

## バージョン 1 の保証

- 各トップレベル JSON オブジェクトは文字列 `schema_version` を含む
- 既存のフィールド名は `1.x` 系列内で安定
- 新規フィールドはオプションの前方互換拡張としてのみ追加
- 入れ子の `report_files` オブジェクトも独自の `schema_version` を含む
- 文字列パスフィールドは現在の実行が書き込んだアーティファクト/セッションパスを保持

## コンシューマの指針

外部 CI ツール、ボット、アーカイブリーダは以下のようにすべきです:

1.  `schema_version == "1"` を受け入れる
2.  前方互換性のために未知のオプションフィールドを無視する
3.  将来のスキーマバンプを破壊的なパーサ境界として扱う

## 追加的変更と破壊的変更

新しいオプションカウンタ、新しいオプションメタデータブロック、より豊かな相互リンクフィールドなどの追加的変更では、スキーマバージョンは `1` のままです。必須フィールドが削除・非互換にリネーム・型変更・意味変更される場合に、将来のバージョンバンプが必要です。

## 関連ガイド

- [証明と検証](proof-verification.md) — 全体的なモデル/エクスポート/実行/信頼のストーリー
- [検証 CI ワークフロー](verification-ci.md) — 自動化でのレポート/レイアウト利用
- [検証ワークフロー](../examples/verification-workflow.md) — エンドツーエンドの例


Karpal は Apache-2.0 + CLA でライセンスされています。[GitHub で見る](https://github.com/Industrial-Algebra/Karpal)。
