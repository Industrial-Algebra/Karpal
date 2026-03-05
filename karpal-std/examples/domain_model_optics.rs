//! Domain Model with Optics Example
//!
//! Demonstrates Lens (product type field access), ComposedLens (deep access via .then()),
//! Prism (sum type variant focus), and transform (reusable update functions).
//!
//! Run with: cargo run -p karpal-std --example domain_model_optics

use karpal_std::prelude::*;

// --- Domain model ---

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
    CreditCard {
        last4: String,
        exp: String,
    },
    BankTransfer {
        iban: String,
    },
    Wallet {
        provider: String,
        balance_cents: i64,
    },
}

// --- Lenses ---

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

fn zip_lens() -> SimpleLens<Address, String> {
    Lens::new(|a: &Address| a.zip.clone(), |a, zip| Address { zip, ..a })
}

fn name_lens() -> SimpleLens<Customer, String> {
    Lens::new(
        |c: &Customer| c.name.clone(),
        |c, name| Customer { name, ..c },
    )
}

// --- Prisms ---

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
            Payment::Wallet {
                provider,
                balance_cents,
            } => Ok((provider, balance_cents)),
            other => Err(other),
        },
        |(provider, balance_cents)| Payment::Wallet {
            provider,
            balance_cents,
        },
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

// --- Sample data ---

fn sample_order() -> Order {
    Order {
        id: 1001,
        customer: Customer {
            name: "Alice Smith".into(),
            address: Address {
                street: "123 Main St".into(),
                city: "Springfield".into(),
                zip: "62701".into(),
            },
        },
        items: vec![
            Item {
                name: "Keyboard".into(),
                price_cents: 7999,
                quantity: 1,
            },
            Item {
                name: "USB Cable".into(),
                price_cents: 899,
                quantity: 3,
            },
        ],
        payment: Payment::CreditCard {
            last4: "4242".into(),
            exp: "12/25".into(),
        },
    }
}

fn main() {
    println!("=== Domain Model with Optics Example ===\n");

    let order = sample_order();

    // 1. Lens: get/set/over
    println!("--- Lens: basic get/set/over ---");
    let customer = customer_lens();
    let address = address_lens();
    let city = city_lens();

    println!("  Customer name: {}", name_lens().get(&order.customer));
    println!("  City: {}", city.get(&address.get(&customer.get(&order))));

    // 2. Composed Lens: deep access with .then()
    println!("\n--- ComposedLens: deep access with .then() ---");
    let order_city = customer_lens().then(address_lens()).then(city_lens());
    let order_zip = customer_lens().then(address_lens()).then(zip_lens());

    println!("  Order city: {}", order_city.get(&order));
    println!("  Order zip:  {}", order_zip.get(&order));

    // Deep set
    let updated = order_city.set(order.clone(), "Shelbyville".into());
    println!("  After city update: {}", order_city.get(&updated));
    println!("  Original unchanged: {}", order_city.get(&order));

    // Deep over (modify)
    let uppercased = order_city.over(order.clone(), |c| c.to_uppercase());
    println!("  Uppercased city: {}", order_city.get(&uppercased));

    // 3. Lens::transform — reusable update function
    println!("\n--- Lens::transform — reusable update function ---");
    let normalize_city: Box<dyn Fn(String) -> String> = Box::new(|c| c.trim().to_uppercase());
    let normalize_order_city = city_lens().transform::<FnP>(normalize_city);
    let addr = Address {
        street: "456 Oak Ave".into(),
        city: "  new york  ".into(),
        zip: "10001".into(),
    };
    let normalized = normalize_order_city(addr);
    println!("  Normalized city: {:?}", normalized.city);

    // 4. Prism: sum type focus
    println!("\n--- Prism: sum type focus ---");
    let cc = credit_card_prism();
    let wallet = wallet_prism();
    let bank = bank_transfer_prism();

    println!("  Credit card last4: {:?}", cc.preview(&order.payment));
    println!("  Wallet preview:    {:?}", wallet.preview(&order.payment));

    // Prism::review — construct a variant
    let new_payment = wallet.review(("PayPal".into(), 5000));
    println!("  New wallet payment: {:?}", new_payment);

    // Prism::over — modify only if matched
    let updated_payment = cc.over(order.payment.clone(), |(last4, _exp)| {
        (last4, "01/28".into())
    });
    println!("  Updated CC expiry: {:?}", updated_payment);

    // Prism on non-matching variant: passes through unchanged
    let unchanged = wallet.over(order.payment.clone(), |(prov, bal)| (prov, bal + 1000));
    println!("  Wallet over on CC (unchanged): {:?}", unchanged);

    // 5. Prism::transform — reusable pattern-matching function
    println!("\n--- Prism::transform — reusable variant modifier ---");
    let add_balance: Box<dyn Fn((String, i64)) -> (String, i64)> =
        Box::new(|(prov, bal)| (prov, bal + 2500));
    let add_wallet_balance = wallet.transform::<FnP>(add_balance);

    let wallet_payment = Payment::Wallet {
        provider: "PayPal".into(),
        balance_cents: 10000,
    };
    let topped_up = add_wallet_balance(wallet_payment);
    println!("  Topped up: {:?}", topped_up);

    // Apply same transform to non-wallet — passes through
    let still_cc = add_wallet_balance(order.payment.clone());
    println!("  CC unchanged: {:?}", still_cc);

    // 6. Summary: combine multiple optics
    println!("\n--- Combining lenses and prisms ---");
    let orders = vec![
        sample_order(),
        Order {
            id: 1002,
            customer: Customer {
                name: "Bob Jones".into(),
                address: Address {
                    street: "789 Elm St".into(),
                    city: "Capital City".into(),
                    zip: "12345".into(),
                },
            },
            items: vec![Item {
                name: "Mouse".into(),
                price_cents: 2999,
                quantity: 1,
            }],
            payment: Payment::Wallet {
                provider: "Stripe".into(),
                balance_cents: 50000,
            },
        },
        Order {
            id: 1003,
            customer: Customer {
                name: "Carol White".into(),
                address: Address {
                    street: "321 Pine Rd".into(),
                    city: "Springfield".into(),
                    zip: "62702".into(),
                },
            },
            items: vec![],
            payment: Payment::BankTransfer {
                iban: "DE89370400440532013000".into(),
            },
        },
    ];

    let order_city_lens = customer_lens().then(address_lens()).then(city_lens());
    println!("  All cities:");
    for o in &orders {
        let payment_type = match &o.payment {
            Payment::CreditCard { .. } => "CC",
            Payment::BankTransfer { .. } => "Bank",
            Payment::Wallet { .. } => "Wallet",
        };
        println!(
            "    Order #{}: {} ({}, pays via {})",
            o.id,
            order_city_lens.get(o),
            name_lens().get(&o.customer),
            payment_type
        );
    }

    // Find all bank IBANs using Prism
    println!("\n  Bank IBANs:");
    for o in &orders {
        if let Some(iban) = bank.preview(&o.payment) {
            println!("    Order #{}: {}", o.id, iban);
        }
    }
}
