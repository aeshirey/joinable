# joinable
`joinable` defines traits for joining iterables of values. Just as you can join two database
tables in SQL to produce matching records from each, `joinable` provides you with simple
functionality to achive the same in your Rust code.

The `Joinable` trait lets you join left- and right-hand sides, yielding `(&L, &R)` for inner
joins and `(&L, Option<&R>)` for outer joins. Because the same left value might be yielded
multiple times due to multiple right matches, `Joinable` uses borrowed values from the LHS:

```rust
use joinable::Joinable;
let customers = get_customers();
let orders = get_orders();
let it = customers
    .iter()
    .outer_join(&orders[..], |c, o| c.id.cmp(&o.customer_id));
```

The `JoinableGrouped` trait joins left- and right-hand sides with right-hand side values
collected into a `Vec`. This 'grouped' version yields each left value at most once, so it
can take ownership of the left-hand iterator:

```rust
use joinable::JoinableGrouped;
let customers = get_customers();
let orders = get_orders();
let it = customers
    .into_iter()
    .outer_join_grouped(&orders[..], |c, o| c.id.cmp(&o.customer_id));
for (cust, ords) in it {
    if ords.is_empty() {
        println!("Customer '{}' has no orders", cust.name);
    } else {
        let total_spend = ords.iter().map(|o| o.amount_usd).sum::<f32>();
        println!("Customer '{}' has spent ${:0.2}", cust.name, total_spend);
    }
}
```

`JoinableGrouped` also exposes SEMIJOIN and ANTISEMIJOIN functionality, yielding only rows from
the left-hand side where a match is or is not found, respectively, in the right-hand side:

```rust
use joinable::JoinableGrouped;
let customers = get_customers();
let orders = get_orders();
let customers_with_orders : Vec<&Customer> = customers
    .iter()
    .semi_join(&orders[..], |c, o| c.id.cmp(&o.customer_id))
    .collect();
let customers_without_orders : Vec<Customer> = customers
    .into_iter()
    .anti_join(&orders[..], |c, o| c.id.cmp(&o.customer_id))
    .collect();
```

## Search predicate
For all joins, the search predicate is of the type `Fn(&L, &R) -> std::cmp::Ordering`; that is, 
given some value from the left- and from the right-hand side, your predicate must identify how
the two values compare. If whatever type you use to match doesn't implement `PartialOrd`, you
can simply check for equality and return `Ordering::Equal`/some non-`Equal` value.

## Binary searching with `RHS::Sorted`
The `RHS` enum wraps the right-hand side of your join. By default, `RHS` assumes your data are
unordered:

```rust
let customers_with_orders : Vec<&Customer> = customers
    .iter()
    .semi_join(&orders[..], |c, o| c.id.cmp(&o.customer_id))
    //          ^^^^^^ orders is implicitly converted Into<RHS>
    .collect();
```

If your use case permits it and it makes sense, you can sort your right-hand side according to
the search predicate, allowing searches to be binary searched in O(ln n) instead of linearly O(n):

```rust
let customers_with_orders : Vec<&Customer> = customers
    .iter()
    .semi_join(RHS::Sorted(&orders[..]), |c, o| c.id.cmp(&o.customer_id))
    //         ^^^^^^^^^^^^^^^^^^^^^^^^ signal that orders is sorted by customer_id
    .collect();
```

`joinable` assumes that your ordered data are in _ascending order_. If you have ordered descending,
then you can reverse the ordering.