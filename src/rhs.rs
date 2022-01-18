/// A wrapper around the right-hand side of your join.
pub enum RHS<'a, R> {
    /// Input which is not (necessarily) sorted. Searches of RHS will be O(n).
    /// 
    /// This variant can be explicitly created with [RHS::new_unsorted] or 
    /// implicitly via the `impl From<&[R]>`.
    Unsorted(&'a [R]),

    /// Input which is known to be sorted according to the join predicate. Searches of RHS will be O(n*lg n)
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
}

#[test]
fn test_has_value_sorted() {
    use crate::joinable::Joinable;
    let left = [1, 2, 3];

    let right = vec![(1, "hello"), (2, "world"), (2, "!")];
    let right = RHS::new_sorted(&right);

    let mut joined = left
        .iter()
        .inner_join(right, |l, r| r.0.cmp(l))
        .flat_map(|x| x.1);

    assert_eq!(joined.next(), Some(&(1, "hello")));
    assert_eq!(joined.next(), Some(&(2, "world")));
    assert_eq!(joined.next(), Some(&(2, "!")));
}

#[test]
fn test_has_value_unsorted() {
    use crate::joinable::Joinable;
    let left = [1, 2, 3];

    let right = vec![(1, "hello"), (2, "world")];
    let right = RHS::new_unsorted(&right);

    let mut joined = left
        .iter()
        .inner_join(right, |l, r| (*l).cmp(&r.0))
        .flat_map(|x| x.1);

    assert_eq!(joined.next(), Some(&(1, "hello")));
    assert_eq!(joined.next(), Some(&(2, "world")));
}
