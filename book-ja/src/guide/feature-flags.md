# フィーチャーフラグ

各クレートは `no_std` 互換性のための `std` および `alloc` フィーチャーゲートをサポートします。

## デフォルトフィーチャー

デフォルトでは、すべてのクレートが `std` を有効化します:

```toml
karpal-core = "0.7"  # デフォルトで std を有効化
```

## alloc 付き no_std

```toml
karpal-core = { version = "0.7", default-features = false, features = ["alloc"] }
```

## alloc なし no_std

コアトレイトはアロケータなしで動作します:

```toml
karpal-core = { version = "0.7", default-features = false }
```

## 特殊フィーチャー

| クレート | フィーチャー | 効果 |
|---------|------------|------|
| `karpal-verify` | `amari` | amari-flynn による統計的検証 |
| `karpal-proof` | `derive` | `#[derive(VerifySemigroup)]` など |
| `karpal-verify` | `derive` | `#[export_obligations]` マクロ |

## 例外

- `karpal-schubert-types` は std 専用 (`amari-enumerative` に依存)
- `karpal-index` はバイナリクレート (crates.io には公開されていません)

## クレート別 `no_std` 状態

| クレート | `no_std` (core のみ) | `alloc` | `std` | 備考 |
|---------|----------------------|---------|-------|-------|
| `karpal-core` | ✅ | ✅ | ✅ | HKT エンコーディング、関手階層、Semigroup/Monoid は alloc なしで動作 |
| `karpal-profunctor` | ✅ | ✅ | ✅ | Profunctor、Strong、Choice、FnP |
| `karpal-optics` | ✅ | ✅ | ✅ | Lens、Prism、合成 |
| `karpal-arrow` | ✅ | ✅ | ✅ | Arrow 階層 |
| `karpal-free` | ✅ | ✅ | ✅ | 自由構成 (ほとんどは alloc が必要) |
| `karpal-recursion` | ✅ | ✅ | ✅ | 帰納スキーム |
| `karpal-algebra` | ✅ | ✅ | ✅ | 抽象代数 |
| `karpal-effect` | ✅ | ✅ | ✅ | モナド変換子 |
| `karpal-proof` | ✅ | ✅ | ✅ | 法則証拠、精密化型 |
| `karpal-verify` | ❌ | ❌ | ✅ | 検証ブリッジ (プロセス起動、ファイルシステム) |
| `karpal-verify-derive` | ❌ | ❌ | ✅ | proc-macro クレート (std が必要) |
| `karpal-proof-derive` | ❌ | ❌ | ✅ | proc-macro クレート (std が必要) |
| `karpal-diagram` | ✅ | ✅ | ✅ | ストリング図式 |
| `karpal-schubert-types` | ❌ | ❌ | ✅ | `amari-enumerative` に依存 |
| `karpal-higher` | ✅ | ✅ | ✅ | 2-圏、豊饒圏 |
| `karpal-std` | ❌ | ❌ | ✅ | プレリュード再エクスポート (全クレートを取り込む) |

CI はプッシュのたびに `cargo build --no-default-features -p karpal-core -p karpal-profunctor` でこれを検証します。
