/// A wrapper around the right-hand side of your join.
pub enum RHS<'a, R> {
    /// Input which is not (necessarily) sorted. Searches of RHS will be O(n).
    ///
    /// This variant can be explicitly created with [RHS::new_unsorted] or
    /// implicitly via the `impl From<&[R]>`.
    Unsorted(&'a [R]),

    /// Input which is known to be sorted according to the join predicate. Searches of RHS will be O(lg n)
    ///
    /// This variant is explicitly created with [RHS::new_sorted]
    Sorted(&'a [R]),
}

impl<'a, R> From<&'a [R]> for RHS<'a, R> {
    fn from(rhs: &'a [R]) -> Self {
        RHS::Unsorted(rhs)
    }
}

impl<'a, R> RHS<'a, R> {
    /// Create a new RHS from the given slice.
    ///
    /// Provided records will be searched linearly.
    pub fn new_unsorted(rhs: &'a [R]) -> Self {
        RHS::Unsorted(rhs)
    }

    /// Create a new RHS from the given slice where records are assumed to be sorted
    /// according to how they will be searched.
    ///
    /// Provided records will be binary searched, yielding faster searches.
    pub fn new_sorted(rhs: &'a [R]) -> Self {
        RHS::Sorted(rhs)
    }

    pub(crate) fn has_value<L, P>(&self, l: &L, predicate: P) -> bool
    where
        P: Fn(&L, &R) -> std::cmp::Ordering,
    {
        match self {
            RHS::Unsorted(rs) => rs.iter().any(|r| (predicate)(l, r).is_eq()),
            RHS::Sorted(rs) => rs.binary_search_by(|r| (predicate)(l, r)).is_ok(),
        }
    }

    pub(crate) fn get_range<L, P>(&'a self, left: &L, predicate: &P) -> (usize, usize)
    where
        P: Fn(&L, &R) -> std::cmp::Ordering,
    {
        match self {
            RHS::Unsorted(rs) => (0, rs.len()),
            RHS::Sorted(rs) => {
                if let Ok(pos) = rs.binary_search_by(|r| (predicate)(left, r).reverse()) {
                    // We found *a* match, but it may not be the first one
                    let mut start = pos;
                    while start > 0 && (predicate)(left, &rs[start - 1]).is_eq() {
                        start -= 1;
                    }

                    // Same for the end
                    let mut end = pos;
                    while end < rs.len() && (predicate)(left, &rs[end]).is_eq() {
                        end += 1;
                    }

                    //panic!("sorted range from pos={pos}: ({start}, {end})"); // (4,6)

                    (start, end)
                } else {
                    // No match found; values here don't matter except that the range must be empty
                    (1, 0)
                }
            }
        }
    }
}

#[test]
fn test_has_value_sorted() {
    use crate::joined_grouped::JoinableGrouped;
    let left = [1, 2, 3];

    let right = vec![(1, "hello"), (2, "world"), (2, "!")];
    let right = RHS::new_sorted(&right);

    let mut joined = left
        .iter()
        .inner_join_grouped(right, |l, r| r.0.cmp(l))
        .flat_map(|x| x.1);

    assert_eq!(joined.next(), Some(&(1, "hello")));
    assert_eq!(joined.next(), Some(&(2, "world")));
    assert_eq!(joined.next(), Some(&(2, "!")));
}

#[test]
fn test_has_value_unsorted() {
    use crate::JoinableGrouped;
    let left = [1, 2, 3];

    let right = vec![(1, "hello"), (2, "world")];
    let right = RHS::new_unsorted(&right);

    let mut joined = left
        .iter()
        .inner_join_grouped(right, |l, r| (*l).cmp(&r.0))
        .flat_map(|x| x.1);

    assert_eq!(joined.next(), Some(&(1, "hello")));
    assert_eq!(joined.next(), Some(&(2, "world")));
}
