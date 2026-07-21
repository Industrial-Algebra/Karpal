# karpal-index による型の発見

`karpal-index` は、AI エージェント (および人間) が段階的なドリルダウンを通じて Karpal の型と操作を発見できる CLI バイナリです。

## コマンド

### 検索 (search)

```bash
$ karpal-index search Functor
Functor                        trait           共変関手: 関数 A->B を F<A>->F<B> に持ち上げる
FunctorFilter                  trait           FunctorFilter: 要素をフィルタできる Functor
```

### 詳細 (detail)

```bash
$ karpal-index detail Functor
Functor [trait]
  crate: karpal-core
  supertraits: HKT
  methods:
    - fn fmap<A, B>(fa: Self::Of<A>, f: impl Fn(A) -> B) -> Self::Of<B>;
  implementors:
    - OptionF
    - VecF
    - IdentityF
```

### 階層 (hierarchy)

```bash
$ karpal-index hierarchy Semigroup
Semigroup [trait]
  subtraits:
    - Monoid
  implementors:
    - String
```

### JSON 出力

すべてのコマンドはプログラム利用のための `--json` をサポートします:

```bash
$ karpal-index search Functor --json
[{"name":"Functor","kind":"trait","crate_name":"karpal-core",...}]
```

## 使い方

```sh
cargo run --bin karpal-index -- search Functor
# またはインストール:
cargo install --path . --bin karpal-index
karpal-index search Functor
```

このバイナリは実行時にワークスペースのソースツリーを読み取ります — 事前構築済みのインデックスは不要です。
