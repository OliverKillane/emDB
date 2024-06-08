//! ## A complex analytical workload
//! To test [`emdb`]'s OLAP performance.

use emdb::macros::emql;

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum ProductCategory {
    Electronics,
    Clothing,
    Food,
}

#[derive(Clone, Copy)]
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

#[derive(Clone)]
struct Aggregate {
    clothes: usize,
    electronics: usize,
    food: usize,
    money_spent: u64,
}

emql!{
    impl analytical as Interface{
        pub = on,
    };
    impl emdb_impl as Serialized {
        ds_name = EmDB,
        interface = analytical,
        pub = on,
    };

    table products {
        serial: usize,
        name: String,
        category: crate::analytical::ProductCategory,
    } @ [unique(serial) as unique_serial_number]

    table purchases {
        customer_reference: [u8; 4],
        product_serial: usize,
        quantity: u8,
        price: u64,
        currency: crate::analytical::Currency,
    } @ [pred(crate::analytical::validate_price(price, currency)) as sensible_prices]

    // We delete old customers, but keep their references
    table current_customers {
        reference: [u8; 4],
        name: String,
        address: String,
    } @ [
        unique(reference) as unique_customer_reference, 
        unique(address) as unique_customer_address,
        pred(name.len() > 3) as sensible_name, 
        pred(address.len() > 0) as non_empty_address,
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
        currency: crate::analytical::Currency,
    ) {
        row(
            customer_reference: [u8; 4] = customer_reference,
            product_serial: usize = product_serial,
            quantity: u8 = quantity,
            price: u64 = price,
            currency: crate::analytical::Currency = currency,
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
        category: crate::analytical::ProductCategory,
    ) {
        row(
            serial: usize = serial,
            name: String = name,
            category: crate::analytical::ProductCategory = category,
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
    query customer_value(btc_rate: f64, usd_rate: f64, cust_ref: [u8; 4]) {
        row(cust_ref: [u8;4] = cust_ref)
            ~> unique(cust_ref for current_customers.reference as ref customer_ref)
            ~> deref(customer_ref as customer)
            ~> lift(
                use purchases
                    |> filter(**customer_reference == cust_ref)
                    |> let customer_purchases;
                
                use products |> let all_prods;

                join(use all_prods [inner equi(serial = product_serial)] use customer_purchases)
                    |> map(result: crate::analytical::Aggregate = {
                        use crate::analytical::ProductCategory::*;
                        
                        let (clothes, electronics, food) = match all_prods.category {
                            Electronics => (1, 0, 0),
                            Clothing => (0, 1, 0),
                            Food => (0, 0, 1),
                        };

                        crate::analytical::Aggregate {
                            clothes,
                            electronics,
                            food,
                            money_spent: crate::analytical::exchange(btc_rate, usd_rate, *customer_purchases.price, *customer_purchases.currency),
                        }
                    })
                    |> combine(use left + right in result = crate::analytical::Aggregate {
                        clothes: left.result.clothes + right.result.clothes,
                        electronics: left.result.electronics + right.result.electronics,
                        food: left.result.food + right.result.food,
                        money_spent: left.result.money_spent + right.result.money_spent,
                    })
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
                    |> groupby(customer_reference for let purchases in {
                        use purchases 
                            |> map(sum: u64 = crate::analytical::exchange(btc_rate, usd_rate, *price, *currency))
                            |> combine(use left + right in sum = left.sum + right.sum)
                            ~> map(customer: &'db[u8; 4] = customer_reference, product_serial: usize = serial, total: u64 = sum)
                            ~> return;
                    }) 
                    |> collect(customers)
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
                category: crate::analytical::ProductCategory = *product_data.category, 
                money: u64 = crate::analytical::exchange(
                    btc_rate, usd_rate, *purchase_data.price, *purchase_data.currency
                )
            )
            |> groupby(category for let category_purchase_data in {
                use category_purchase_data
                    |> combine(use left + right in money = left.money + right.money)
                    ~> map(category: crate::analytical::ProductCategory = category, total: u64 = money)
                    ~> return;
            }) 
            |> collect(category_totals) 
            ~> return;
    }
}
