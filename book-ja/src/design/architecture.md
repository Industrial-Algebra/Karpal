# アーキテクチャ

## 設計原則

- **GAT ベースの HKT エンコーディング**: `trait HKT { type Of<T>; }` — クリーン、依存関係ゼロ
- **Fantasy Land より Static Land**: 値のメソッドではなく、関連関数を持つトレイト
- **法則検証を内蔵**: すべてのトレイトに proptest ベースの法則テストが付属
- **`no_std` ファースト**: core と profunctor クレートはアロケータなしで動作
- **完全性より合成**: 各フェーズは次が始まる前に利用可能な層を提供
- **構造化された空**: ゼロは来歴を持つ — 「なぜ空なのか」が重要

## フェーズ完了状況

| フェーズ | クレート | 状態 |
|---------|---------|------|
| 1–11 | core から proof まで | ✅ 完了 |
| 12 | karpal-verify | ✅ 完了 |
| 13 | karpal-diagram | ✅ 完了 |
| 14 | karpal-schubert-types (A–D) | ✅ 完了 |
| 15 | karpal-higher | ✅ 完了 |
| 16A | HeytingAlgebra | ✅ 完了 |
| 16B–D | トポス理論 | ✅ 完了 |
| 17 | E2E 検証 | 🔲 計画中 |
| 18 | エコシステム検証 | 🔲 計画中 |

## ライセンス

Apache-2.0 + CLA。[CONTRIBUTING.md](https://github.com/Industrial-Algebra/Karpal/blob/develop/CONTRIBUTING.md) を参照してください。
