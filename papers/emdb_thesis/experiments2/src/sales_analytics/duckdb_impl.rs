use super::sales_analytics::{Database, Datastore};
use duckdb::{params, Connection};

pub struct DuckDB {
    conn: Connection,
}

pub struct DuckDBDatabase<'imm> {
    conn: &'imm mut Connection,
}

impl Datastore for DuckDB {
    type DB<'imm> = DuckDBDatabase<'imm>;

    fn new() -> Self {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch(
            "
            -- Product Categories are 'Electronics' (0), 'Clothing' (1) or 'Food' (2) 
            -- Currencies are 'GBP' (0), 'USD' (1) or 'BTC' (2) 
            -- Cannot use enums due to the sensible prices constraint, and this bug https://github.com/duckdb/duckdb-rs/issues/334
            -- Additionally difficult to convert back to rust type, so use u8s instead

            CREATE TABLE products (
                serial UBIGINT NOT NULL,
                name VARCHAR NOT NULL,
                category UTINYINT NOT NULL,
                CONSTRAINT unique_serial_number UNIQUE(serial)
            );

            CREATE TABLE purchases (
                customer_reference UBIGINT,
                product_serial UBIGINT,
                quantity UTINYINT,
                price UBIGINT,
                currency UTINYINT NOT NULL,
                CONSTRAINT sensible_prices CHECK (
                    (currency = 1 AND price <= 10000 * 100) OR
                    (currency = 2 AND price < 20) OR
                    (currency = 0)  -- No constraint for GBP
                )
            );

            CREATE TABLE customers (
                reference UBIGINT NOT NULL,
                name VARCHAR NOT NULL,
                address VARCHAR NOT NULL,
                CONSTRAINT unique_customer_reference UNIQUE(reference),
                CONSTRAINT unique_customer_address UNIQUE(address),
                CONSTRAINT sensible_name CHECK (LENGTH(name) > 2),
                CONSTRAINT non_empty_address CHECK (LENGTH(address) > 0)
            );

            CREATE TABLE old_customers (
                reference UBIGINT NOT NULL,
            );
        ",
        )
        .unwrap();
        Self { conn }
    }

    fn db(&mut self) -> Self::DB<'_> {
        DuckDBDatabase {
            conn: &mut self.conn,
        }
    }
}

impl<'imm> Database<'imm> for DuckDBDatabase<'imm> {
    type Datastore = DuckDB;

    fn new_customer<'qy>(&'qy mut self, reference: usize, name: String, address: String) {
        self.conn
            .prepare_cached(" INSERT INTO customers (reference, name, address) VALUES (?, ?, ?) ")
            .unwrap()
            .query_row(params![reference, name, address], |_| Ok(()))
            .unwrap()
    }

    fn new_sale<'qy>(
        &'qy mut self,
        customer_reference: usize,
        product_serial: usize,
        quantity: u8,
        price: u64,
        currency: crate::sales_analytics::Currency,
    ) {
        self.conn
            .prepare_cached(" INSERT INTO purchases (customer_reference, product_serial, quantity, price, currency) VALUES (?, ?, ?, ?, ?)")
            .unwrap()
            .query_row(params![
                customer_reference,
                product_serial,
                quantity,
                price,
                match currency {
                    super::Currency::GBP => 0,
                    super::Currency::USD => 1,
                    super::Currency::BTC => 2,
                }
            ], |_| Ok(())).unwrap()
    }

    fn customer_leaving<'qy>(&'qy mut self, reference: usize) {
        let trans = self.conn.transaction().unwrap();
        trans
            .prepare_cached("DELETE FROM customers WHERE reference = ?")
            .unwrap()
            .query_row(params![reference], |_| Ok(()))
            .unwrap();
        trans
            .prepare_cached("INSERT INTO old_customers (reference) VALUES (?)")
            .unwrap()
            .query_row(params![reference], |_| Ok(()))
            .unwrap();
        trans.commit().unwrap();
    }

    fn new_product<'qy>(
        &'qy mut self,
        serial: usize,
        name: String,
        category: crate::sales_analytics::ProductCategory,
    ) {
        self.conn
            .prepare_cached(" INSERT INTO products (serial, name, category) VALUES (?, ?, ?)")
            .unwrap()
            .query_row(
                params![
                    serial,
                    name,
                    match category {
                        super::ProductCategory::Electronics => 0,
                        super::ProductCategory::Clothing => 1,
                        super::ProductCategory::Food => 2,
                    }
                ],
                |_| Ok(()),
            )
            .unwrap()
    }

    fn customer_value<'qy>(
        &'qy self,
        btc_rate: f64,
        usd_rate: f64,
        cust_ref_outer: usize,
    ) -> (usize, u64, usize, usize, usize) {
        let res = self
            .conn
            .prepare_cached(
                "
        WITH customer_purchases AS (
            SELECT
                p.customer_reference,
                p.product_serial,
                p.quantity,
                p.price,
                p.currency,
                pr.category
            FROM
                purchases p
            JOIN
                products pr ON p.product_serial = pr.serial
            WHERE
                p.customer_reference = ? -- cust_ref_outer
        ),
        purchase_totals AS (
            SELECT
                customer_reference,
                SUM(
                    CASE
                        WHEN currency = 1 THEN price * ?
                        WHEN currency = 2 THEN price * ?
                        ELSE price
                    END * quantity
                ) AS money_spent,
                SUM(
                    CASE
                        WHEN category = 0 THEN quantity
                        ELSE 0
                    END
                ) AS electronics,
                SUM(
                    CASE
                        WHEN category = 1 THEN quantity
                        ELSE 0
                    END
                ) AS clothes,
                SUM(
                    CASE
                        WHEN category = 2 THEN quantity
                        ELSE 0
                    END
                ) AS food
            FROM
                customer_purchases
            GROUP BY
                customer_reference
        )
        SELECT
            ct.customer_reference,
            ct.money_spent,
            ct.electronics,
            ct.clothes,
            ct.food
        FROM
            purchase_totals ct
        JOIN
            customers cc ON ct.customer_reference = cc.reference;
        ",
            )
            .unwrap()
            .query_map(params![cust_ref_outer, usd_rate, btc_rate], |row| {
                Ok((
                    row.get(0)?,
                    row.get(1)?,
                    row.get(2)?,
                    row.get(3)?,
                    row.get(4)?,
                ))
            })
            .unwrap()
            .into_iter()
            .collect::<Result<Vec<_>, _>>()
            .unwrap();
        if res.is_empty() {
            (0, 0, 0, 0, 0)
        } else {
            res[0]
        }
    }

    fn product_customers<'qy>(
        &'qy self,
        serial: usize,
        btc_rate: f64,
        usd_rate: f64,
    ) -> Vec<(usize, u64, u64)> {
        self.conn
            .prepare_cached(
                "
        WITH filtered_purchases AS (
            SELECT
                customer_reference,
                quantity,
                price,
                currency
            FROM
                purchases
            WHERE
                product_serial = ?
        ),
        exchange_rates AS (
            SELECT
                CASE
                    WHEN currency = 1 THEN ROUND(price * ?, 0)
                    WHEN currency = 2 THEN ROUND(price * ?, 0)
                    ELSE price
                END * quantity AS total_spent,
                customer_reference
            FROM
                filtered_purchases
        ),
        total_spent_by_customer AS (
            SELECT
                customer_reference,
                SUM(total_spent) AS total_spent
            FROM
                exchange_rates
            GROUP BY
                customer_reference
        )
        SELECT
            ?
            AS product_serial,
            customer_reference,
            total_spent
        FROM
            total_spent_by_customer;
        ",
            )
            .unwrap()
            .query_map(params![serial, usd_rate, btc_rate, serial], |row| {
                Ok((row.get(0)?, row.get(1)?, row.get(2)?))
            })
            .unwrap()
            .collect::<Result<Vec<_>, _>>()
            .unwrap()
    }

    fn category_sales<'qy>(&'qy self, btc_rate: f64, usd_rate: f64) -> Vec<(u8, u64)> {
        self.conn
            .prepare_cached(
                "
            WITH joined_data AS (
                SELECT
                    pr.category,
                    pu.quantity,
                    pu.price,
                    pu.currency,
                    CASE
                        WHEN pu.currency = 1 THEN ROUND(pu.price * ?, 0)
                        WHEN pu.currency = 2 THEN ROUND(pu.price * ?, 0)
                        ELSE pu.price
                    END * pu.quantity AS money
                FROM
                    purchases pu
                INNER JOIN
                    products pr ON pu.product_serial = pr.serial
            ),
            aggregated_data AS (
                SELECT
                    category,
                    SUM(money) AS total
                FROM
                    joined_data
                GROUP BY
                    category
            )
            SELECT
                category,
                total
            FROM
                aggregated_data;
        ",
            )
            .unwrap()
            .query_map(params![usd_rate, btc_rate], |row| {
                Ok((row.get(0)?, row.get(1)?))
            })
            .unwrap()
            .collect::<Result<Vec<_>, _>>()
            .unwrap()
    }
}
