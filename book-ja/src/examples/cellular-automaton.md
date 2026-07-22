# セルラーオートマトン

NonEmptyVec 上の Extend (Comonad) を使った一次元セルラーオートマトン。

## 概要

**セルラーオートマトン** は離散ステップで進化するセルのグリッドです。各ステップで、すべてのセルがセルとその近傍を検査する *ルール* に基づいて値を更新します。関数型プログラミングにおける古典的アプローチは、これを **コモナド** でモデル化することです。

重要な洞察は、コモナドがセルラーオートマトンパターンに直接対応する二つの演算を提供することです:

- **`extract`** は「焦点の」セル — グリッド内の現在位置 — を読み取ります。
- **`extend`** は焦点の文脈から新しい値を計算する関数を取り、グリッドの *すべての* 位置に適用します。これはまさにセルラーオートマトンルールの動作です: ルールは位置の周りの近傍を見て、`extend` がそれを至る所で実行します。

Karpal では、`NonEmptyVec` が `Extend` と `Comonad` を実装します。`extend` メソッドは (`tails` 経由で) グリッドのすべての可能な焦点付きビューを生成し、ルール関数をそれぞれに適用して、一回の呼び出しで次世代を生成します。

## ルール関数

各ルールはグリッド全体を `&NonEmptyVec<u8>` として受け取り、ベクタの *先頭* が現在のセルとして機能します。ルールは隣接位置を見ることで近傍を検査し、そのセルの新しい値を返します。

### ルール 90 (近傍の XOR)

二つの近傍のちょうど一方が生存していればセルは生存 (`1`) になり、さもなくば死亡 (`0`) します。単一のシードセルから始めると、古典的なシェルピンスキーの三角形パターンを生成します。

``` rust
fn rule_90(grid: &NonEmptyVec<u8>) -> u8 {
    let tails = grid.tails();
    let current = <NonEmptyVecF as Comonad>::extract(grid);
    let len = grid.len();

    // 左の近傍を取得 (ラップ around)
    let left = if len > 1 {
        *tails.tail.last().map(|t| &t.head).unwrap_or(&grid.head)
    } else {
        current
    };

    // 右の近傍を取得
    let right = if grid.tail.is_empty() {
        grid.head // ラップ around
    } else {
        grid.tail[0]
    };

    // 近傍の XOR
    left ^ right
}
```

### 多数決ルール

より単純なルール: (左、現在、右) のうち二つ以上が生存していればセルは生存します。これはノイズを平滑化し、一様な領域に収束する傾向があります。

``` rust
fn rule_majority(grid: &NonEmptyVec<u8>) -> u8 {
    let current = <NonEmptyVecF as Comonad>::extract(grid);
    let len = grid.len();

    let left = if len > 1 {
        let tails = grid.tails();
        *tails.tail.last().map(|t| &t.head).unwrap_or(&grid.head)
    } else {
        current
    };

    let right = if grid.tail.is_empty() {
        grid.head
    } else {
        grid.tail[0]
    };

    let sum = left as u16 + current as u16 + right as u16;
    if sum >= 2 { 1 } else { 0 }
}
```

## Extend による進化

`step` 関数がオートマトンの中核です。グリッドとルールで `NonEmptyVecF::extend` を呼び出し、次世代を生成します。Extend はグリッドのすべての焦点付きビューを生成し、それぞれにルールをマップすることで、すべての位置でルールを適用します。

``` rust
fn step(grid: NonEmptyVec<u8>, rule: fn(&NonEmptyVec<u8>) -> u8) -> NonEmptyVec<u8> {
    NonEmptyVecF::extend(grid, rule)
}
```

`evolve` 関数は指定された世代数だけ `step` を反復し、空間-時間図として表示できるよう完全な履歴を収集します。

``` rust
fn evolve(
    initial: NonEmptyVec<u8>,
    rule: fn(&NonEmptyVec<u8>) -> u8,
    steps: usize,
) -> Vec<NonEmptyVec<u8>> {
    let mut history = vec![initial.clone()];
    let mut current = initial;
    for _ in 0..steps {
        current = step(current, rule);
        history.push(current.clone());
    }
    history
}
```

## 表示

表示ヘルパは各世代を `#` (生存) と `.` (死亡) 文字の文字列として描画し、ターミナルでパターンを見えるようにします。

``` rust
fn display_grid(grid: &NonEmptyVec<u8>) -> String {
    let mut s = String::new();
    for cell in grid.iter() {
        s.push(if *cell == 1 { '#' } else { '.' });
    }
    s
}
```

## すべてを組み合わせる

`main` 関数は中央に単一のシードセルを持つ初期グリッドを設定し、ルール 90 を 10 世代実行し、次により複雑なパターンで多数決ルールを実演します。また `Comonad::extract` と `Extend::duplicate` を直接示します。

``` rust
fn main() {
    // 初期状態: 21 セルのグリッドの中央に単一セル
    let width = 21;
    let mid = width / 2;
    let mut cells: Vec<u8> = vec![0; width];
    cells[mid] = 1;
    let initial = NonEmptyVec::new(cells[0], cells[1..].to_vec());

    // ルール 90 (近傍の XOR)
    let history = evolve(initial.clone(), rule_90, 10);
    for (i, grid) in history.iter().enumerate() {
        println!("  {:>2}: {}", i, display_grid(grid));
    }

    // Comonad::extract は焦点のセルを読み取る
    let head = <NonEmptyVecF as Comonad>::extract(&initial);

    // 異なるパターンでの多数決ルール
    let pattern = NonEmptyVec::new(1, vec![0, 1, 1, 0, 0, 1, 0, 1, 1, 0, 1, 0, 0, 1, 0]);
    let history = evolve(pattern, rule_majority, 8);

    // Extend::duplicate はすべての焦点付きビューを示す
    let small = NonEmptyVec::new(1, vec![2, 3]);
    let duplicated: NonEmptyVec<NonEmptyVec<u8>> = NonEmptyVecF::duplicate(small);
}
```

## 実行

ワークスペースルートから実行:

``` rust
cargo run -p karpal-std --example cellular_automaton
```

単一のシードから成長するルール 90 のシェルピンスキーの三角形パターンに続き、多数決ルールがランダムに見えるパターンを安定した領域に平滑化するのが見えます。

## 使用するトレイト

| トレイト     | この例での役割                                                                                                                         | リファレンス                                          |
|-----------|----------------------------------------------------------------------------------------------------------------------------------------------|----------------------------------------------------|
| `Comonad` | `NonEmptyVec` から焦点のセル値を読み取る `extract` を提供。                                                                      | [コモナドファミリー](../reference/comonad-family.md) |
| `Extend`  | すべての位置でルールを適用し次世代を生成する `extend` を提供。すべての焦点付き位置を見る `duplicate` も提供。 | [コモナドファミリー](../reference/comonad-family.md) |
| `HKT`     | `NonEmptyVecF` は `Extend` と `Comonad` を実装する型コンストラクタマーカー。                                                        | [関手ファミリー](../reference/functor-family.md) |


Karpal は Apache-2.0 + CLA でライセンスされています。[GitHub で見る](https://github.com/Industrial-Algebra/Karpal)。
