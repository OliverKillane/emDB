use super::sales_analytics::{Database, Datastore};
use rusqlite::{params, Connection};

pub struct SQLite {
    conn: Connection,
}

pub struct SQLiteDatabase<'imm> {
    conn: &'imm mut Connection,
}

impl Datastore for SQLite {
    type DB<'imm> = SQLiteDatabase<'imm>;

    /// IMPORTANT NOTE: This implementation is less constrained than the emdb one,
    ///                 as it ommits the `sensible_prices` predicate on the `purchases`
    ///                 table.
    ///
    /// The schema should include:
    /// ```sql
    /// CREATE TABLE purchases (
    ///     customer_reference UNSIGNED BIG INT,
    ///     product_serial UNSIGNED BIG INT,
    ///     quantity UTINYINT,
    ///     price UNSIGNED BIG INT,
    ///     currency Currency NOT NULL,
    ///     CONSTRAINT sensible_prices CHECK (
    ///         (currency = 'USD' AND price <= 10000 * 100) OR
    ///         (currency = 'BTC' AND price < 20) OR
    ///         (currency = 'GBP')  -- No constraint for GBP
    ///     )
    /// );
    /// ```
    /// However due to a bug in duckdb, on insert the database will crash.
    /// - This bug has an issue [here](https://github.com/duckdb/duckdb-rs/issues/334)
    ///
    /// Hence we are forgiving, and do not enforce this constraint (to duckdb's
    /// performance advantage).
    fn new() -> Self {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch(
            "
            -- Currency is 'GBP', 'USD' or 'BTC'
            -- Product Category  is 'Electronics' (0), 'Clothing' (1) or 'Food' (2)

            CREATE TABLE products (
                serial UNSIGNED BIG INT NOT NULL,
                name VARCHAR NOT NULL,
                category INT8 NOT NULL,
                CONSTRAINT unique_serial_number UNIQUE(serial)
            );

            CREATE TABLE purchases (
                customer_reference UNSIGNED BIG INT,
                product_serial UNSIGNED BIG INT,
                quantity UTINYINT,
                price UNSIGNED BIG INT,
                currency INT8 NOT NULL
            );

            CREATE TABLE customers (
                reference UNSIGNED BIG INT NOT NULL,
                name VARCHAR NOT NULL,
                address VARCHAR NOT NULL,
                CONSTRAINT unique_customer_reference UNIQUE(reference),
                CONSTRAINT unique_customer_address UNIQUE(address),
                CONSTRAINT sensible_name CHECK (LENGTH(name) > 2),
                CONSTRAINT non_empty_address CHECK (LENGTH(address) > 0)
            );

            CREATE TABLE old_customers (
                reference UNSIGNED BIG INT NOT NULL
            );
        ",
        )
        .unwrap();
        Self { conn }
    }

    fn db(&mut self) -> Self::DB<'_> {
        SQLiteDatabase {
            conn: &mut self.conn,
        }
    }
}

impl<'imm> Database<'imm> for SQLiteDatabase<'imm> {
    type Datastore = SQLite;

    fn new_customer(&mut self, reference: usize, name: String, address: String) {
        self.conn
            .prepare_cached("INSERT INTO customers (reference, name, address) VALUES (?, ?, ?)")
            .unwrap()
            .execute(params![reference, name, address])
            .unwrap();
    }

    fn new_sale(
        &mut self,
        customer_reference: usize,
        product_serial: usize,
        quantity: u8,
        price: u64,
        currency: crate::sales_analytics::Currency,
    ) {
        self.conn
            .prepare_cached(" INSERT INTO purchases (customer_reference, product_serial, quantity, price, currency) VALUES (?, ?, ?, ?, ?)")
            .unwrap()
            .execute(params![
                customer_reference,
                product_serial,
                quantity,
                price,
                match currency {
                    super::Currency::GBP => 0,
                    super::Currency::USD => 1,
                    super::Currency::BTC => 2,
                }
            ]).unwrap();
    }

    fn customer_leaving(&mut self, reference: usize) {
        let trans = self.conn.transaction().unwrap();
        trans
            .prepare_cached("DELETE FROM customers WHERE reference = ?")
            .unwrap()
            .execute(params![reference])
            .unwrap();
        trans
            .prepare_cached("INSERT INTO old_customers (reference) VALUES (?)")
            .unwrap()
            .execute(params![reference])
            .unwrap();
        trans.commit().unwrap();
    }

    fn new_product(
        &mut self,
        serial: usize,
        name: String,
        category: crate::sales_analytics::ProductCategory,
    ) {
        self.conn
            .prepare_cached(" INSERT INTO products (serial, name, category) VALUES (?, ?, ?)")
            .unwrap()
            .execute(params![
                serial,
                name,
                match category {
                    super::ProductCategory::Electronics => 0,
                    super::ProductCategory::Clothing => 1,
                    super::ProductCategory::Food => 2,
                }
            ])
            .unwrap();
    }

    fn customer_value(
        &self,
        btc_rate: f64,
        usd_rate: f64,
        cust_ref_outer: usize,
    ) -> (usize, f64, usize, usize, usize) {
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
            .collect::<Result<Vec<_>, _>>()
            .unwrap();
        if res.is_empty() {
            (0, 0.0, 0, 0, 0)
        } else {
            res[0]
        }
    }

    fn product_customers(
        &self,
        serial: usize,
        btc_rate: f64,
        usd_rate: f64,
    ) -> Vec<(usize, u64, f64)> {
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

    fn category_sales(&self, btc_rate: f64, usd_rate: f64) -> Vec<(u8, f64)> {
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
