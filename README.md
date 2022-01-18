# joinable
`joinable` is a Rust trait for joining iterables of values. Just as you can join two database tables in SQL to produce matching records from each, `joinable` provides you with simple functionality to achive the same in your Rust code:

```rust
use joinable::Joinable;
let joined = customers
    .iter()
    .inner_join(&orders[..], |cust, ord| cust.id.cmp(&ord.customer_id))
    .map(|(cust, ords)| {
        // Translate from (&Customer, Vec<&Order>)
        (
            &cust.name,
            ords.iter().map(|ord| ord.amount_usd).sum::<f32>(),
        )
    })
    .collect::<Vec<_>>();
```

`joinable` joins two expressions - the _left-hand-side_ (LHS) and _right-hand-side_ (RHS). The trait provides four functions:
* `inner_join` -- emit each value in LHS with one or more matching RHS records
* `outer_join` -- emit each value in LHS with zero or more matching RHS records
* `semi_join` -- emit each value in LHS if and only if RHS has a matching record
* `anti_join` -- emit each value in LHS if and only if RHS does not have a matching record

All four functions operate on an `Iterator<Item=L>` for LHS and accept an `Into<RHS<R>>`, which can take a slice `&[R]`. If you know your RHS is sorted according to your join condition, you can optionally create an explicitly ordered RHS:

```rust
let orders = joinable::rhs::RHS::new_sorted(&orders[..]);
let joined = customers
    .iter()
    .inner_join(orders, |cust, ord| cust.id.cmp(&ord.customer_id));
```

This permits binary searching of the RHS.