use joinable::Joinable;

fn main() {
    let customers = get_customers();
    let orders = get_orders();

    let it = customers
        .into_iter()
        .outer_join(&orders[..], |c, o| c.id.cmp(&o.customer_id));

    for (cust, ords) in it {
        if ords.is_empty() {
            println!("Customer '{}' has no orders", cust.name);
        } else {
            let total_spend = ords.iter().map(|o| o.amount_usd).sum::<f32>();
            println!("Customer '{}' has spent ${:0.2}", cust.name, total_spend);
        }
    }
}

#[derive(Debug)]
struct Customer {
    id: u32,
    name: String,
}
fn get_customers() -> Vec<Customer> {
    vec![
        Customer {
            id: 123,
            name: "ACME".to_string(),
        },
        Customer {
            id: 456,
            name: "Contoso".to_string(),
        },
        Customer {
            id: 789,
            name: "Foobar, Inc".to_string(),
        },
    ]
}

#[derive(Debug)]
struct Order {
    id: u32,
    customer_id: u32,
    amount_usd: f32,
}

fn get_orders() -> Vec<Order> {
    vec![
        Order {
            id: 1,
            customer_id: 123,
            amount_usd: 10.,
        },
        Order {
            id: 2,
            customer_id: 123,
            amount_usd: 11.,
        },
        Order {
            id: 3,
            customer_id: 123,
            amount_usd: 12.,
        },
        Order {
            id: 4,
            customer_id: 456,
            amount_usd: 35.,
        },
        Order {
            id: 5,
            customer_id: 456,
            amount_usd: 36.,
        },
    ]
}
