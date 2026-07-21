# ドメインモデルのオプティクス

Lens 合成、Prism、プロ関手変換を使った e コマースドメインモデル。

## 概要

現実世界のドメインモデルは **直積型** (名前付きフィールドを持つ構造体) と **直和型** (異なるバリアントを持つ列挙型) の両方を含みます。Karpal はそれらを扱うための二つの相補的なオプティクスを提供します:

- **Lens** — 直積型内の単一フィールドに焦点を当てます。すべての直積はそのフィールドを持つため、Lens は常に成功します。Lens は `.then()` で合成し、深く入れ子になったフィールドに到達できます。
- **Prism** — 直和型の単一バリアントに焦点を当てます。バリアントは存在する也可能で、Prism は優雅に失敗できます。Prism により、他に触れずに個別のバリアントをプレビュー・構築・変更できます。

どちらのオプティクスも `transform` をサポートし、[Profunctor](../reference/profunctor-family.md) 抽象化 (`FnP`) を使って `A -> A` の内側の関数から再利用可能な `S -> S` 更新関数を生成します。これが合成可能で第一級のデータ変換の鍵です。

## ドメインモデル

この例は入れ子になった構造体と支払い方法の列挙型を持つ e コマース注文を定義します:

``` rust
#[derive(Debug, Clone, PartialEq)]
struct Order {
    id: u32,
    customer: Customer,
    items: Vec<Item>,
    payment: Payment,
}

#[derive(Debug, Clone, PartialEq)]
struct Customer {
    name: String,
    address: Address,
}

#[derive(Debug, Clone, PartialEq)]
struct Address {
    street: String,
    city: String,
    zip: String,
}

#[derive(Debug, Clone, PartialEq)]
struct Item {
    name: String,
    price_cents: i64,
    quantity: u32,
}

#[derive(Debug, Clone, PartialEq)]
enum Payment {
    CreditCard { last4: String, exp: String },
    BankTransfer { iban: String },
    Wallet { provider: String, balance_cents: i64 },
}
```

`Order`、`Customer`、`Address`、`Item` は直積型です — すべてのインスタンスがすべてのフィールドを持ちます。`Payment` は直和型です — 各注文はちょうど一つの支払い方法を使います。

## Lens の定義

`SimpleLens<S, A>` は `Lens::new` で作成し、getter (`&S -> A`) と setter (`(S, A) -> S`) を取ります:

``` rust
fn customer_lens() -> SimpleLens<Order, Customer> {
    Lens::new(
        |o: &Order| o.customer.clone(),
        |o, customer| Order { customer, ..o },
    )
}

fn address_lens() -> SimpleLens<Customer, Address> {
    Lens::new(
        |c: &Customer| c.address.clone(),
        |c, address| Customer { address, ..c },
    )
}

fn city_lens() -> SimpleLens<Address, String> {
    Lens::new(
        |a: &Address| a.city.clone(),
        |a, city| Address { city, ..a },
    )
}
```

各 Lens は単一フィールドの読み書き方法を知る小さく自己完結した単位です。setter は Rust の構造体更新構文 (`..o`) を使い、他のすべてのフィールドを変更なくコピーします。

## `.then()` による Lens 合成

個別の Lens は `.then()` で `ComposedLens` に合成します。これにより、getter と setter を手動で繋がずに深く入れ子になったフィールドに到達できます:

``` rust
let order_city = customer_lens().then(address_lens()).then(city_lens());
let order_zip  = customer_lens().then(address_lens()).then(zip_lens());

// 深い取得
println!("Order city: {}", order_city.get(&order));
println!("Order zip:  {}", order_zip.get(&order));

// 深い設定 (新しい Order を返す、元は変更なし)
let updated = order_city.set(order.clone(), "Shelbyville".into());

// 深い over (焦点の値に関数を適用)
let uppercased = order_city.over(order.clone(), |c| c.to_uppercase());
```

合成 Lens `order_city` は `ComposedLens<Order, String>` 型です。単純な Lens と同じ `get`、`set`、`over` 操作をサポートしますが、三レベル深く到達します: `Order -> Customer -> Address -> city`。

## `FnP` による Lens 変換

`transform` メソッドは Lens と内側の関数を再利用可能な更新関数に変換します。`FnP` プロ関手 (boxed 関数型) を使い、`A -> A` 関数を `S -> S` 関数に持ち上げます:

``` rust
let normalize_city: Box<dyn Fn(String) -> String> =
    Box::new(|c| c.trim().to_uppercase());
let normalize_order_city = city_lens().transform::<FnP>(normalize_city);

let addr = Address {
    street: "456 Oak Ave".into(),
    city: "  new york  ".into(),
    zip: "10001".into(),
};
let normalized = normalize_order_city(addr);
// normalized.city == "NEW YORK"
```

結果は city フィールドだけを正規化するプレーンな `Address -> Address` 関数です。保存し、受け渡し、任意の `Address` 値に適用できます。

## Prism の定義

`SimplePrism<S, A>` は `Prism::new` で作成し、マッチ関数 (`S -> Result<A, S>`) とビルド関数 (`A -> S`) を取ります。マッチはバリアントが一致すれば `Ok(a)` を返し、さもなくば元の値で `Err(s)` を返します:

``` rust
fn credit_card_prism() -> SimplePrism<Payment, (String, String)> {
    Prism::new(
        |p| match p {
            Payment::CreditCard { last4, exp } => Ok((last4, exp)),
            other => Err(other),
        },
        |(last4, exp)| Payment::CreditCard { last4, exp },
    )
}

fn wallet_prism() -> SimplePrism<Payment, (String, i64)> {
    Prism::new(
        |p| match p {
            Payment::Wallet { provider, balance_cents } => Ok((provider, balance_cents)),
            other => Err(other),
        },
        |(provider, balance_cents)| Payment::Wallet { provider, balance_cents },
    )
}

fn bank_transfer_prism() -> SimplePrism<Payment, String> {
    Prism::new(
        |p| match p {
            Payment::BankTransfer { iban } => Ok(iban),
            other => Err(other),
        },
        |iban| Payment::BankTransfer { iban },
    )
}
```

## Prism 操作

Prism は三つのコア操作を提供します:

- **`preview(&S) -> Option<A>`** — 焦点のバリアントの抽出を試みます。マッチすれば `Some(a)`、さもなくば `None`。
- **`review(A) -> S`** — バリアントの内側のデータから直和型の値を構築します。
- **`over(S, Fn(A) -> A) -> S`** — マッチすれば焦点のバリアントを変更; さもなくば変更なく通過させます。

``` rust
let cc = credit_card_prism();
let wallet = wallet_prism();

// preview: マッチすれば抽出
cc.preview(&order.payment);      // Some(("4242", "12/25"))
wallet.preview(&order.payment);  // None (注文はクレジットカード支払い)

// review: バリアントを構築
let new_payment = wallet.review(("PayPal".into(), 5000));
// Payment::Wallet { provider: "PayPal", balance_cents: 5000 }

// over: マッチ時のみ変更
let updated_payment = cc.over(order.payment.clone(), |(last4, _exp)| {
    (last4, "01/28".into())
});
// 有効期限を更新; 他のフィールドはそのまま

// 非マッチバリアントでの over: 変更なく通過
let unchanged = wallet.over(order.payment.clone(), |(prov, bal)| {
    (prov, bal + 1000)
});
// まだ CreditCard — wallet.over はここでは何もしない
```

## Prism 変換

Lens と同様に、Prism は `transform` をサポートし再利用可能な `S -> S` 関数を生成します。変換された関数はバリアントがマッチすれば内側の変更を適用し、さもなくば値を変更なく返します:

``` rust
let add_balance: Box<dyn Fn((String, i64)) -> (String, i64)> =
    Box::new(|(prov, bal)| (prov, bal + 2500));
let add_wallet_balance = wallet_prism().transform::<FnP>(add_balance);

let wallet_payment = Payment::Wallet {
    provider: "PayPal".into(),
    balance_cents: 10000,
};
let topped_up = add_wallet_balance(wallet_payment);
// Payment::Wallet { provider: "PayPal", balance_cents: 12500 }

// 非ウォレット支払いに適用 — 変更なく通過
let still_cc = add_wallet_balance(order.payment.clone());
// まだ CreditCard { last4: "4242", exp: "12/25" }
```

## Lens と Prism の組み合わせ

実際には Lens と Prism を一緒に使います。Lens は直積型フィールドにドリルし; Prism は直和型バリアントで分岐します。この例は注文のコレクションを反復し両方のオプティクスを使うことを実演します:

``` rust
let order_city_lens = customer_lens().then(address_lens()).then(city_lens());

for o in &orders {
    let city = order_city_lens.get(o);
    let name = name_lens().get(&o.customer);
    let payment_type = match &o.payment {
        Payment::CreditCard { .. } => "CC",
        Payment::BankTransfer { .. } => "Bank",
        Payment::Wallet { .. } => "Wallet",
    };
    println!("Order #{}: {} ({}, pays via {})", o.id, city, name, payment_type);
}

// Prism を使ってすべての銀行 IBAN を抽出
let bank = bank_transfer_prism();
for o in &orders {
    if let Some(iban) = bank.preview(&o.payment) {
        println!("Order #{}: {}", o.id, iban);
    }
}
```

## 実行

ワークスペースルートからこの例を実行:

``` rust
cargo run -p karpal-std --example domain_model_optics
```
