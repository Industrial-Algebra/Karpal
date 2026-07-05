# Domain Model with Optics

E-commerce domain model using Lens composition, Prism, and profunctor transform.

## Overview

Real-world domain models contain both **product types** (structs with named fields) and **sum types** (enums with distinct variants). Karpal provides two complementary optics for working with them:

- **Lens** — focuses on a single field inside a product type. Every product has the field, so a Lens always succeeds. Lenses compose with `.then()` to reach deeply nested fields.
- **Prism** — focuses on a single variant of a sum type. The variant may or may not be present, so a Prism can fail gracefully. Prisms let you preview, construct, and modify individual variants without touching the others.

Both optics support `transform`, which uses the [Profunctor](../reference/profunctor-family.md) abstraction (`FnP`) to produce a reusable `S -> S` update function from an `A -> A` inner function. This is the key to composable, first-class data transformations.

## The Domain Model

The example defines an e-commerce order with nested structs and an enum for payment methods:

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

`Order`, `Customer`, `Address`, and `Item` are product types — every instance has every field. `Payment` is a sum type — each order uses exactly one payment method.

## Defining Lenses

A `SimpleLens<S, A>` is created with `Lens::new`, which takes a getter (`&S -> A`) and a setter (`(S, A) -> S`):

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

Each lens is a small, self-contained unit that knows how to read and write a single field. The setter uses Rust's struct update syntax (`..o`) to copy all other fields unchanged.

## Lens Composition with `.then()`

Individual lenses compose into a `ComposedLens` via `.then()`. This lets you reach deeply nested fields without manually threading getters and setters:

``` rust
let order_city = customer_lens().then(address_lens()).then(city_lens());
let order_zip  = customer_lens().then(address_lens()).then(zip_lens());

// Deep get
println!("Order city: {}", order_city.get(&order));
println!("Order zip:  {}", order_zip.get(&order));

// Deep set (returns a new Order, original unchanged)
let updated = order_city.set(order.clone(), "Shelbyville".into());

// Deep over (apply a function to the focused value)
let uppercased = order_city.over(order.clone(), |c| c.to_uppercase());
```

The composed lens `order_city` has type `ComposedLens<Order, String>`. It supports the same `get`, `set`, and `over` operations as a simple lens, but it reaches three levels deep: `Order -> Customer -> Address -> city`.

## Lens Transform with `FnP`

The `transform` method converts a lens and an inner function into a reusable update function. It uses the `FnP` profunctor (a boxed function type) to lift an `A -> A` function into an `S -> S` function:

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

The result is a plain `Address -> Address` function that normalizes only the city field. You can store it, pass it around, and apply it to any `Address` value.

## Defining Prisms

A `SimplePrism<S, A>` is created with `Prism::new`, which takes a match function (`S -> Result<A, S>`) and a build function (`A -> S`). The match returns `Ok(a)` if the variant matches, or `Err(s)` with the original value if it does not:

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

## Prism Operations

Prisms provide three core operations:

- **`preview(&S) -> Option<A>`** — attempts to extract the focused variant. Returns `Some(a)` on match, `None` otherwise.
- **`review(A) -> S`** — constructs a sum type value from the variant's inner data.
- **`over(S, Fn(A) -> A) -> S`** — modifies the focused variant if it matches; passes through unchanged if it does not.

``` rust
let cc = credit_card_prism();
let wallet = wallet_prism();

// preview: extract if matched
cc.preview(&order.payment);      // Some(("4242", "12/25"))
wallet.preview(&order.payment);  // None (order pays by credit card)

// review: construct a variant
let new_payment = wallet.review(("PayPal".into(), 5000));
// Payment::Wallet { provider: "PayPal", balance_cents: 5000 }

// over: modify only if matched
let updated_payment = cc.over(order.payment.clone(), |(last4, _exp)| {
    (last4, "01/28".into())
});
// Updates the expiry; leaves other fields intact

// over on non-matching variant: passes through unchanged
let unchanged = wallet.over(order.payment.clone(), |(prov, bal)| {
    (prov, bal + 1000)
});
// Still CreditCard — wallet.over is a no-op here
```

## Prism Transform

Like lenses, prisms support `transform` to produce a reusable `S -> S` function. The transformed function applies the inner modification when the variant matches and returns the value unchanged otherwise:

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

// Apply to a non-wallet payment — passes through unchanged
let still_cc = add_wallet_balance(order.payment.clone());
// Still CreditCard { last4: "4242", exp: "12/25" }
```

## Combining Lenses and Prisms

In practice you use lenses and prisms together. Lenses drill into product type fields; prisms branch on sum type variants. The example demonstrates iterating over a collection of orders and using both optics:

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

// Extract all bank IBANs using a prism
let bank = bank_transfer_prism();
for o in &orders {
    if let Some(iban) = bank.preview(&o.payment) {
        println!("Order #{}: {}", o.id, iban);
    }
}
```

## Run It

To run this example from the workspace root:

``` rust
cargo run -p karpal-std --example domain_model_optics
```

## Traits Used

| Trait / Type            | Role in this example                                               | Reference                                                |
|-------------------------|--------------------------------------------------------------------|----------------------------------------------------------|
| `Lens` / `SimpleLens`   | Focus on a single field in a product type; get, set, over          | [Optics](../reference/optics.md)                       |
| `ComposedLens`          | Chain lenses with `.then()` for deep nested access                 | [Optics](../reference/optics.md)                       |
| `Prism` / `SimplePrism` | Focus on a single variant of a sum type; preview, review, over     | [Optics](../reference/optics.md)                       |
| `FnP`                   | Profunctor marker type for `transform`; lifts `A -> A` to `S -> S` | [Profunctor Family](../reference/profunctor-family.md) |
| `Strong`                | Profunctor subclass used internally by Lens transform              | [Profunctor Family](../reference/profunctor-family.md) |
| `Choice`                | Profunctor subclass used internally by Prism transform             | [Profunctor Family](../reference/profunctor-family.md) |


Karpal is licensed under Apache-2.0 + CLA. [View on GitHub](https://github.com/Industrial-Algebra/Karpal).


