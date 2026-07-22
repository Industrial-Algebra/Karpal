# 数学的基礎

Karpal は圏論 — 構造と合成の数学 — の上に構築されています。

## HKT エンコーディング

高階型 (Higher-Kinded Types, HKT) とは、他の型をパラメータとして取る型です: つまり単なる `A` ではなく `F<A>` です。Rust はネイティブには HKT をサポートしませんが、GAT (Generic Associated Types、Rust 1.65 から安定) が依存関係ゼロのエンコーディングを提供します:

```rust
pub trait HKT {
    type Of<T>;
}
```

`OptionF` のようなマーカー型は `type Of<T> = Option<T>` として `HKT` を実装します。これにより、コンテナの「形」について汎用的なトレイトを書けるようになります。

## 関手階層

中核となる抽象化は関手階層です:

```
Functor → Apply → Applicative
                 ↓
          Chain → Monad
```

各レベルは機能を追加します:
- **Functor**: コンテナ上でマップする (`fmap`)
- **Apply**: 二つのコンテナを組み合わせる (`ap`)
- **Applicative**: 純粋な値を生成する (`pure`)
- **Chain**: 演算を逐次化する (`chain` / `bind`)
- **Monad**: 完全な逐次計算

## 代数的構造

関手階層に加えて、Karpal は代数的型クラスを提供します:

- **Semigroup / Monoid**: 結合的な組み合わせ + 単位元
- **Group / AbelianGroup**: モノイド + 逆元
- **Semiring / Ring / Field**: 分配律を持つ二つの演算
- **Lattice / BoundedLattice**: join + meet (すべての上限・下限を持つ半順序集合)
- **HeytingAlgebra**: 含意を持つ有界束 (直観主義論理)

ハイティング代数は「構造化された空」の基礎です — 「なぜ空なのか」という理由が情報を持つという考え方です。
