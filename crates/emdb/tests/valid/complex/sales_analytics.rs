//! ## A complex analytical workload
//! To test [`emdb`]'s OLAP performance.

use std::{collections::HashMap, fmt::Display};

use emdb::macros::emql;

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum ProductCategory {
    Electronics,
    Clothing,
    Food,
}

impl Display for ProductCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProductCategory::Electronics => write!(f, "📱"),
            ProductCategory::Clothing => write!(f, "👕"),
            ProductCategory::Food => write!(f, "🍌"),
        }
    }
}

#[allow(clippy::upper_case_acronyms)]
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

fn exchange(btc_rate: f64, usd_rate: f64,price: u64, currency: Currency) -> u64 {
    match currency {
        Currency::GBP => price,
        Currency::USD => (price as f64 * usd_rate) as u64,
        Currency::BTC => (price as f64 * btc_rate) as u64,
    }
} 

#[derive(Clone, PartialEq, Eq, Debug)]
#[derive(Default)]
struct Aggregate {
    clothes: usize,
    electronics: usize,
    food: usize,
    money_spent: u64,
}


emql!{
    impl my_db as Serialized {
        op_impl = Parallel,
    };

    table products {
        serial: usize,
        name: String,
        category: crate::valid::complex::sales_analytics::ProductCategory,
    } @ [unique(serial) as unique_serial_number]

    table purchases {
        customer_reference: [u8; 4],
        product_serial: usize,
        quantity: u8,
        price: u64,
        currency: crate::valid::complex::sales_analytics::Currency,
    } @ [pred(crate::valid::complex::sales_analytics::validate_price(price, currency)) as sensible_prices]

    // We delete old customers, but keep their references
    table current_customers {
        reference: [u8; 4],
        name: String,
        address: String,
    } @ [
        unique(reference) as unique_customer_reference, 
        unique(address) as unique_customer_address,
        pred(name.len() > 2) as sensible_name, 
        pred(!address.is_empty()) as non_empty_address,
    ]

    // Old customers, deleted but references kept for purchases
    table old_customers {
        reference: [u8; 4],
    }
    
    // Basic queries for data population ======================================= 
    query new_customer(
        reference: [u8; 4],
        name: String,
        address: String,
    ) {
        row(
            reference: [u8; 4] = reference,
            name: String = name,
            address: String = address,
        ) ~> insert(current_customers as ref customer_ref) ~> return;
    }
    query new_sale(
        customer_reference: [u8; 4],
        product_serial: usize,
        quantity: u8,
        price: u64,
        currency: crate::valid::complex::sales_analytics::Currency,
    ) {
        row(
            customer_reference: [u8; 4] = customer_reference,
            product_serial: usize = product_serial,
            quantity: u8 = quantity,
            price: u64 = price,
            currency: crate::valid::complex::sales_analytics::Currency = currency,
        ) ~> insert(purchases as ref sale_ref) ~> return;
    }
    query customer_leaving(
        reference: [u8; 4],
    ) {
        row(
            reference: [u8; 4] = reference,
        ) 
            ~> unique(reference for current_customers.reference as ref customer_ref)
            ~> delete(customer_ref)
            ~> map(reference: [u8; 4] = reference)
            ~> insert(old_customers as ref customer_ref);
    }

    query new_product(
        serial: usize,
        name: String,
        category: crate::valid::complex::sales_analytics::ProductCategory,
    ) {
        row(
            serial: usize = serial,
            name: String = name,
            category: crate::valid::complex::sales_analytics::ProductCategory = category,
        ) ~> insert(products as ref product_ref) ~> return;
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
    query customer_value(btc_rate: f64, usd_rate: f64, cust_ref_outer: [u8; 4]) {
        row(cust_ref: [u8;4] = cust_ref_outer)
            ~> unique(cust_ref for current_customers.reference as ref customer_ref)
            ~> deref(customer_ref as customer)
            ~> lift(
                use purchases
                    |> filter(**customer_reference == cust_ref_outer)
                    |> let customer_purchases;
                
                use products |> let all_prods;

                join(use all_prods [inner equi(serial = product_serial)] use customer_purchases)
                    |> map(result: crate::valid::complex::sales_analytics::Aggregate = {
                        use crate::valid::complex::sales_analytics::ProductCategory::*;
                        let q = *customer_purchases.quantity as usize;
                        let (electronics, clothes, food) = match all_prods.category {
                            Electronics => (q, 0, 0),
                            Clothing => (0, q, 0),
                            Food => (0, 0, q),
                        };
                        crate::valid::complex::sales_analytics::Aggregate {
                            clothes,
                            electronics,
                            food,
                            money_spent: (*customer_purchases.quantity as u64) * crate::valid::complex::sales_analytics::exchange(btc_rate, usd_rate, *customer_purchases.price, *customer_purchases.currency),
                        }
                    })
                    |> combine(use left + right in result[crate::valid::complex::sales_analytics::Aggregate::default()] = [crate::valid::complex::sales_analytics::Aggregate {
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
                            |> map(sum: u64 = (*quantity as u64) * crate::valid::complex::sales_analytics::exchange(btc_rate, usd_rate, *price, *currency))
                            |> combine(use left + right in sum[0] = [ left.sum + right.sum])
                            ~> map(customer: &'db[u8; 4] = customer_reference, total: u64 = sum)
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
                category: crate::valid::complex::sales_analytics::ProductCategory = *product_data.category, 
                money: u64 = (*purchase_data.quantity as u64) * crate::valid::complex::sales_analytics::exchange(
                    btc_rate, usd_rate, *purchase_data.price, *purchase_data.currency
                )
            )
            |> groupby(category for let category_purchase_data in {
                use category_purchase_data
                    |> combine(use left + right in money[0] = [left.money + right.money])
                    ~> map(category: crate::valid::complex::sales_analytics::ProductCategory = category, total: u64 = money)
                    ~> return;
            }) 
            |> collect(category_totals) 
            ~> return;
    }
}


pub fn test() {
    let mut ds = my_db::Datastore::new();
    let mut db = ds.db();

    let btc_rate = 10000.7;
    let usd_rate = 0.8;

    let tshirt = 1;
    let jeans = 2;
    let tv = 3;
    let phone = 4;
    let apple = 5;

    db.new_product(tshirt, "T-shirt".to_string(), ProductCategory::Clothing).unwrap();
    db.new_product(jeans, "Jeans".to_string(), ProductCategory::Clothing).unwrap();
    db.new_product(tv, "TV".to_string(), ProductCategory::Electronics).unwrap();
    db.new_product(phone, "Phone".to_string(), ProductCategory::Electronics).unwrap();
    db.new_product(apple, "Apple".to_string(), ProductCategory::Food).unwrap();

    let alice = [1, 2, 3, 4];
    let bob = [2, 3, 4, 5];
    let charlie = [3, 4, 5, 6]; 

    db.new_customer(alice, "Alice".to_string(), "1 Road".to_string()).unwrap();
    db.new_customer(bob, "Bob".to_string(), "2 Road".to_string()).unwrap();
    db.new_customer(charlie, "Charlie".to_string(), "3 Road".to_string()).unwrap();

     
    db.new_sale(alice, tshirt, 2, 100, Currency::GBP).unwrap();
    db.new_sale(alice, jeans, 1, 50, Currency::USD).unwrap();

    db.new_sale(bob, tv, 1, 200, Currency::USD).unwrap();
    db.new_sale(bob, phone, 1, 500, Currency::USD).unwrap();

    db.new_sale(charlie, apple, 3, 10, Currency::BTC).unwrap();
    db.new_sale(charlie, phone, 2, 100, Currency::GBP).unwrap();

    let alice_agg = db.customer_value(btc_rate, usd_rate, alice).unwrap().result;
    assert_eq!(alice_agg, Aggregate{
        clothes: 3,
        electronics: 0,
        food: 0,
        money_spent: 2 * exchange(btc_rate, usd_rate, 100, Currency::GBP) + exchange(btc_rate, usd_rate, 50, Currency::USD)
    });


    let bob_agg = db.customer_value(btc_rate, usd_rate, bob).unwrap().result;
    assert_eq!(bob_agg, Aggregate{
        clothes: 0,
        electronics: 2,
        food: 0,
        money_spent: exchange(btc_rate, usd_rate, 200, Currency::USD) + exchange(btc_rate, usd_rate, 500, Currency::USD)
    });

    let charlie_agg = db.customer_value(btc_rate, usd_rate, charlie).unwrap().result;
    assert_eq!(charlie_agg, Aggregate{
        clothes: 0,
        electronics: 2,
        food: 3,
        money_spent: 3 * exchange(btc_rate, usd_rate, 10, Currency::BTC) + 2 * exchange(btc_rate, usd_rate, 100, Currency::GBP)
    });

    let phone_customers = db.product_customers(phone, btc_rate, usd_rate).unwrap().customers.into_iter().map(|val| (val.customer, val.total)).collect::<HashMap<_,_>>();
    assert_eq!(phone_customers.len(), 2);
    assert_eq!(phone_customers[&bob], exchange(btc_rate, usd_rate, 500, Currency::USD));
    assert_eq!(phone_customers[&charlie], 2 * exchange(btc_rate, usd_rate, 100, Currency::GBP));

    for tot in db.category_sales(btc_rate, usd_rate).category_totals {
        println!("{} £{:>6}", tot.category, tot.total)
    }
}