## No Deletions With Hashtable

A basic database of products, products can be added, but never updated or deleted (insert new product).

```rust
database!(
    name Products;

    table product_index {
        id: usize @ unique, pred(it != 0),
        name: smalltext[20],
        description: bigtext,
        price: f64,
        curr: ref currency[symb],
    }

    table currency {
        symb: fixedtext[5] @ unique,
        nation: smalltext[10],
    }

    query add_currency(symbol: `&str`, country: `&str`) = {
        currency
            <| insert(symb = symbol, nation = country)
    }

    query add_product(name: `&str`, description: `&str`, price: f64, curr: `&str`) = {
        product
            |> size()
            |> let new_id;

        product
            <| insert(id = new_id, name, description, price, curr);

        new_id |> return;
    }

    query get_products(name: `&str`) = {
        product * currency
            |> where(it[left][name] = name && it[right][symb] = it[left][curr])
            |> flatten()
            |> map(name, description, price, currency = symb)
            |> return;
    }
)
```

### Information Advantage

- References to products and currencies are valid forever (never mutated, never deleted)
- We never actually use the unique product ID (presumably the develope intends to extend this later)
- Only read accesses are for `add_product`, and the join for `get_products`

### To Optimise

Apply rule based opts, must consider checks (e.g refs and unique checks).

- Because we know deletions never occur, tracking updates to both tables becomes trivial (track update number for each, references is view will always be valid)
- Hence the performance gain is from caching the expensive join in a view, and reducing the work required to maintain the view.

![](./../diagrams/no_delete_hash.drawio.svg)

### Generated Code

Can return `&str` references to strings (large strings) in the database safely (same lifetime as database).

Concurrent interactions are for the
