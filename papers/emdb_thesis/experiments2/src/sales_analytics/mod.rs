//! ## A complex analytical workload
//! To test [`emdb`]'s OLAP performance, particularly against [`duckdb`].
//! - Embeds buisness logic in database (advantageous for [`emdb`])
//! - Complex aggregations

use crate::utils::{choose, choose_internal, total};
use emdb::macros::emql;
use rand::{rngs::ThreadRng, Rng};
use sales_analytics::{Database, Datastore};

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum ProductCategory {
    Electronics,
    Clothing,
    Food,
}

#[derive(Clone, Copy, Debug)]
pub enum Currency {
    GBP,
    USD,
    BTC,
}

/// Validate a proce by the rules:
/// - No more than $10k in dollars
/// - Fewer than 20 in BTC
fn validate_price(price: &u64, currency: &Currency) -> bool {
    const DECIMAL: u64 = 100;
    match currency {
        Currency::GBP => true,
        Currency::USD => *price <= 10_000 * DECIMAL,
        Currency::BTC => *price < 20,
    }
}

fn exchange(btc_rate: f64, usd_rate: f64, price: u64, currency: Currency) -> u64 {
    match currency {
        Currency::GBP => price,
        Currency::USD => (price as f64 * usd_rate) as u64,
        Currency::BTC => (price as f64 * btc_rate) as u64,
    }
}

#[derive(Clone, PartialEq, Eq, Debug)]
struct Aggregate {
    clothes: usize,
    electronics: usize,
    food: usize,
    money_spent: u64,
}

impl Default for Aggregate {
    fn default() -> Self {
        Aggregate {
            clothes: 0,
            electronics: 0,
            food: 0,
            money_spent: 0,
        }
    }
}

emql! {
    impl sales_analytics as Interface{
        pub = on,
    };
    impl emdb_impl as Serialized{
        interface = sales_analytics,
        pub = on,
        ds_name = EmDB,
    };

    table products {
        serial: usize,
        name: String,
        category: crate::sales_analytics::ProductCategory,
    } @ [unique(serial) as unique_serial_number]

    table purchases {
        customer_reference: usize,
        product_serial: usize,
        quantity: u8,
        price: u64,
        currency: crate::sales_analytics::Currency,
    } @ [pred(crate::sales_analytics::validate_price(price, currency)) as sensible_prices]

    // We delete old customers, but keep their references
    table current_customers {
        reference: usize,
        name: String,
        address: String,
    } @ [
        unique(reference) as unique_customer_reference,
        unique(address) as unique_customer_address,
        pred(name.len() > 2) as sensible_name,
        pred(address.len() > 0) as non_empty_address,
    ]

    // Old customers, deleted but references kept for purchases
    table old_customers {
        reference: usize,
    }

    // Basic queries for data population =======================================
    query new_customer(
        reference: usize,
        name: String,
        address: String,
    ) {
        row(
            reference: usize = reference,
            name: String = name,
            address: String = address,
        ) ~> insert(current_customers as ref customer_ref);
    }
    query new_sale(
        customer_reference: usize,
        product_serial: usize,
        quantity: u8,
        price: u64,
        currency: crate::sales_analytics::Currency,
    ) {
        row(
            customer_reference: usize = customer_reference,
            product_serial: usize = product_serial,
            quantity: u8 = quantity,
            price: u64 = price,
            currency: crate::sales_analytics::Currency = currency,
        ) ~> insert(purchases as ref sale_ref);
    }
    query customer_leaving(
        reference: usize,
    ) {
        row(
            reference: usize = reference,
        )
            ~> unique(reference for current_customers.reference as ref customer_ref)
            ~> delete(customer_ref)
            ~> map(reference: usize = reference)
            ~> insert(old_customers as ref customer_ref);
    }

    query new_product(
        serial: usize,
        name: String,
        category: crate::sales_analytics::ProductCategory,
    ) {
        row(
            serial: usize = serial,
            name: String = name,
            category: crate::sales_analytics::ProductCategory = category,
        ) ~> insert(products as ref product_ref);
    }

    // Anaysis queries =========================================================

    // Description:
    //   Get the total value of a customer's purchases, using the current
    //   exchange rates, but only if they are a current customer.
    //
    //   Additionally get the sum of all products they have purchased in each product
    //   category.
    // Reasoning:
    //   Allows us to demonstrate embedding of business logic into the database.
    query customer_value(btc_rate: f64, usd_rate: f64, cust_ref_outer: usize) {
        row(cust_ref: usize = cust_ref_outer)
            ~> unique(cust_ref for current_customers.reference as ref customer_ref)
            ~> deref(customer_ref as customer)
            ~> lift(
                use purchases
                    |> filter(**customer_reference == cust_ref_outer)
                    |> let customer_purchases;

                use products |> let all_prods;

                join(use all_prods [inner equi(serial = product_serial)] use customer_purchases)
                    |> map(result: crate::sales_analytics::Aggregate = {
                        use crate::sales_analytics::ProductCategory::*;
                        let q = *customer_purchases.quantity as usize;
                        let (electronics, clothes, food) = match all_prods.category {
                            Electronics => (q, 0, 0),
                            Clothing => (0, q, 0),
                            Food => (0, 0, q),
                        };
                        crate::sales_analytics::Aggregate {
                            clothes,
                            electronics,
                            food,
                            money_spent: (*customer_purchases.quantity as u64) * crate::sales_analytics::exchange(btc_rate, usd_rate, *customer_purchases.price, *customer_purchases.currency),
                        }
                    })
                    |> combine(use left + right in result[crate::sales_analytics::Aggregate::default()] = [crate::sales_analytics::Aggregate {
                        clothes: left.result.clothes + right.result.clothes,
                        electronics: left.result.electronics + right.result.electronics,
                        food: left.result.food + right.result.food,
                        money_spent: left.result.money_spent + right.result.money_spent,
                    }])
                    ~> return;
            ) ~> return;
    }

    // Description:
    //   For a given product get for each purchasing customer:
    //     - customer reference
    //     - total spent by the customer on the product
    // Reasoning:
    //   To demonstrate complex aggregations, and returning data structures
    query product_customers(serial: usize, btc_rate: f64, usd_rate: f64) {
        row(serial: usize = serial)
            ~> unique(serial for products.serial as ref product_ref)
            ~> deref(product_ref as product)
            ~> lift(
                use purchases
                    |> filter(**product_serial == serial)
                    |> groupby(customer_reference for let filtered_purchases in {
                        use filtered_purchases
                            |> map(sum: u64 = (*quantity as u64) * crate::sales_analytics::exchange(btc_rate, usd_rate, *price, *currency))
                            |> combine(use left + right in sum[0] = [left.sum + right.sum])
                            ~> map(customer: &'db usize = customer_reference, total: u64 = sum)
                            ~> return;
                    })
                    |> collect(customers as type customers_for_prod)
                    ~> map(product_serial: usize = serial, customers: type customers_for_prod = customers)
                    ~> return ;
            )
            ~> return;
    }

    // Description:
    //   Get the total sales per category, in the different currencies
    // Reasoning:
    //   Demonstrating aggregation over a large table
    query category_sales(btc_rate: f64, usd_rate: f64) {
        use purchases |> let purchase_data;
        use products |> let product_data;

        join(use purchase_data [inner equi(product_serial = serial)] use product_data)
            |> map(
                category: crate::sales_analytics::ProductCategory = *product_data.category,
                money: u64 = (*purchase_data.quantity as u64) * crate::sales_analytics::exchange(
                    btc_rate, usd_rate, *purchase_data.price, *purchase_data.currency
                )
            )
            |> groupby(category for let category_purchase_data in {
                use category_purchase_data
                    |> combine(use left + right in money[0] = [left.money + right.money])
                    ~> map(category: crate::sales_analytics::ProductCategory = category, total: u64 = money)
                    ~> return;
            })
            |> collect(category_totals)
            ~> return;
    }
}

pub mod duckdb_impl;
pub mod sqlite_impl;

pub struct TableConfig {
    pub customers: usize,
    pub sales: usize,
    pub products: usize,
}

impl TableConfig {
    pub fn from_size(size: usize) -> Self {
        TableConfig {
            customers: size / 2,
            sales: size,
            products: size / 4,
        }
    }

    pub fn populate_database<DS: Datastore>(
        Self {
            customers,
            sales,
            products,
        }: &Self,
        rng: &mut ThreadRng,
    ) -> DS {
        let mut ds = DS::new();

        {
            let mut db = ds.db();

            for i in 0..*customers {
                db.new_customer(
                    i,
                    format!("Test Subject {i}"),
                    format!("Address for person {i}"),
                );
            }

            for i in 0..*products {
                db.new_product(
                    i,
                    format!("Product {i}"),
                    choose! { rng
                        1 => ProductCategory::Electronics,
                        1 => ProductCategory::Clothing,
                        1 => ProductCategory::Food,
                    },
                );
            }
            for _ in 0..*sales {
                let currency = choose! { rng
                    1 => Currency::GBP,
                    1 => Currency::USD,
                    1 => Currency::BTC,
                };

                let price = match currency {
                    Currency::GBP => rng.gen_range(0..100000),
                    Currency::USD => rng.gen_range(0..=10000),
                    Currency::BTC => rng.gen_range(0..20),
                };

                db.new_sale(
                    rng.gen_range(0..*customers),
                    rng.gen_range(0..*products),
                    rng.gen_range(0..10),
                    price,
                    currency,
                );
            }
        }
        ds
    }
}
