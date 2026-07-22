# ワークスペース概要

Karpal はドメイン別に編成された 18 のクレートで構成されます:

| クレート | 目的 |
|---------|------|
| `karpal-core` | HKT エンコーディング、Functor→Monad、コモナド、随伴、end/coend |
| `karpal-profunctor` | Profunctor、Strong、Choice、FnP |
| `karpal-optics` | Iso、Lens、Prism、Traversal、Fold、Getter、Setter、Review |
| `karpal-arrow` | Category/Arrow 階層、FnA、KleisliF、CokleisliF |
| `karpal-free` | Coyoneda、Free、Cofree、Freer、Day、Kan 拡張 |
| `karpal-recursion` | Fix、cata、ana、hylo、para、apo、histo、futu、zygo、chrono |
| `karpal-algebra` | Group、Semiring、Ring、Field、Lattice、HeytingAlgebra、Module、VectorSpace |
| `karpal-effect` | ExceptT、WriterT、ReaderT、StateT、MonadTrans |
| `karpal-proof` | Proven<P,T>、Rewrite、精密化型 |
| `karpal-proof-derive` | #[derive(VerifySemigroup)] など |
| `karpal-verify` | Obligation IR、SMT/Lean 4/Kani、GPU オブリゲーション、信頼境界 |
| `karpal-verify-derive` | #[export_obligations] マクロ |
| `karpal-diagram` | モノイダル圏、ストリング図式、コヒーレンス証拠 |
| `karpal-schubert-types` | シューベルト交点型、SchubertProven、LR 豊饒化 |
| `karpal-higher` | 2-圏、豊饒圏、バイ圏、FFunctor、FMonad |
| `karpal-topos` | 前層、篩、部分対象分類子、グロタンディーク位相、層 |
| `karpal-index` | AI エージェントライブラリ発見 CLI |
| `karpal-std` | プレリュード再エクスポート |

詳細な API ドキュメントは [HTML リファレンスドキュメント](https://karpal.industrial-algebra.com) を参照してください。
