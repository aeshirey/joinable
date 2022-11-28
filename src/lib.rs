//! `joinable` defines traits for joining iterables of values. Just as you can join two database
//! tables in SQL to produce matching records from each, `joinable` provides you with simple
//! functionality to achive the same in your Rust code.
//!
//! The `Joinable` trait lets you join left- and right-hand sides, yielding `(&L, &R)` for inner
//! joins and `(&L, Option<&R>)` for outer joins:
//!
//! ```
//! # struct Customer { id: u32 }
//! # struct Order { customer_id: u32 }
//! # fn get_customers() -> Vec<Customer> { Vec::new() }
//! # fn get_orders() -> Vec<Order> { Vec::new() }
//! use joinable::Joinable;
//!
//! let customers = get_customers();
//! let orders = get_orders();
//!
//! let it = customers
//!     .iter()
//!     .outer_join(&orders[..], |c, o| c.id.cmp(&o.customer_id));
//! ```
//!
//! The `JoinableGrouped` trait joins left- and right-hand sides with right-hand side values
//! collected into a `Vec`:
//!
//! ```
//! # struct Customer { id: u32, name: String }
//! # struct Order { customer_id: u32, amount_usd: f32 }
//! # fn get_customers() -> Vec<Customer> { Vec::new() }
//! # fn get_orders() -> Vec<Order> { Vec::new() }
//! use joinable::JoinableGrouped;
//!
//! let customers = get_customers();
//! let orders = get_orders();
//!
//! let it = customers
//!     .into_iter()
//!     .outer_join_grouped(&orders[..], |c, o| c.id.cmp(&o.customer_id));
//!
//! for (cust, ords) in it {
//!     if ords.is_empty() {
//!         println!("Customer '{}' has no orders", cust.name);
//!     } else {
//!         let total_spend = ords.iter().map(|o| o.amount_usd).sum::<f32>();
//!         println!("Customer '{}' has spent ${:0.2}", cust.name, total_spend);
//!     }
//! }
//! ```
//!
//! `JoinableGrouped` also exposes SEMIJOIN and ANTISEMIJOIN functionality, yielding only rows from
//! the left-hand side where a match is or is not found, respectively, in the right-hand side:
//!
//! ```
//! # struct Customer { id: u32 }
//! # struct Order { customer_id: u32 }
//! # fn get_customers() -> Vec<Customer> { Vec::new() }
//! # fn get_orders() -> Vec<Order> { Vec::new() }
//! use joinable::JoinableGrouped;
//!
//! let customers = get_customers();
//! let orders = get_orders();
//!
//! let customers_with_orders : Vec<&Customer> = customers
//!     .iter()
//!     .semi_join(&orders[..], |c, o| c.id.cmp(&o.customer_id))
//!     .collect();
//!
//! let customers_without_orders : Vec<Customer> = customers
//!     .into_iter()
//!     .anti_join(&orders[..], |c, o| c.id.cmp(&o.customer_id))
//!     .collect();
//! ```
mod joined_grouped;
pub use joined_grouped::JoinableGrouped;

mod joined;
pub use joined::Joinable;

mod rhs;
pub use rhs::RHS;
