# インストール

## 必要要件

- nightly Rust (GAT ベースの HKT エンコーディングのため)
- Rust 2024 edition

```sh
rustup default nightly
```

## プロジェクトへの Karpal の追加

### フルプレリュード

```toml
[dependencies]
karpal-std = "0.7"
```

```rust
use karpal_std::prelude::*;
```

### 個別クレート

```toml
[dependencies]
karpal-core = "0.7"      # HKT、関手階層、Semigroup、Monoid
karpal-optics = "0.7"     # Lens、Prism、Traversal、Fold
karpal-proof = "0.7"      # Proven<P,T>、Rewrite 証拠
karpal-verify = "0.7"     # SMT-LIB2、Lean 4、Kani 検証
karpal-diagram = "0.7"    # モノイダル圏、ストリング図式
karpal-higher = "0.7"     # 2-圏、豊饒圏
karpal-schubert-types = "0.7"  # シューベルト交点型
```

## no_std サポート

ほとんどのクレートは `no_std` 互換で、オプションの `std`/`alloc` フィーチャーゲートを持ちます:

```toml
[dependencies]
karpal-core = { version = "0.7", default-features = false, features = ["alloc"] }
```
