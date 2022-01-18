//! `joinable` is a Rust trait for joining iterables of values. Just as you can join two database tables in SQL to produce matching records from each, `joinable` provides you with simple functionality to achive the same in your Rust code:
//!
//! ```rust
//! let customers = get_customers();
//! let orders = get_orders();
//!
//! let it = customers
//!     .into_iter()
//!     .outer_join(&orders[..], |c, o| c.id.cmp(&o.customer_id));
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
mod joinable;
pub use joinable::Joinable;
mod joined;
pub use joined::{Joined, JoinedLeft};
mod rhs;
pub use rhs::RHS;
