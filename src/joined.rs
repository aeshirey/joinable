use crate::RHS;

/// A trait allowing the joining of a left-hand side (LHS) and a right-hand side ([RHS]) dataset.
/// Results are yielded for pairs of values from LHS and RHS.
pub trait Joinable<'a, LIt, R, P, L> {
    /// Joins LHS and RHS, keeping only records from left that have one or more matches in right.
    ///
    /// The specified predicate returns a [std::cmp::Ordering] comparing left and right records.
    ///
    /// Unlike [Joinable::inner_join], this function returns one `(&L, &R)` for every match; that is, if a
    /// record, `L` has multiple matches in RHS, it will be yielded multiple times.
    fn inner_join(
        self,
        rhs: impl Into<RHS<'a, R>>,
        predicate: P,
    ) -> JoinedEachInner<'a, LIt, R, P, L>;

    fn outer_join(
        self,
        rhs: impl Into<RHS<'a, R>>,
        predicate: P,
    ) -> JoinedEachOuter<'a, LIt, R, P, L>;
}

impl<'a, LIt, R, P, L> Joinable<'a, LIt, R, P, L> for LIt
where
    LIt: Iterator<Item = &'a L>,
    L: 'a,
    R: 'a,
    P: Fn(&L, &R) -> std::cmp::Ordering,
{
    fn inner_join(
        self,
        rhs: impl Into<RHS<'a, R>>,
        predicate: P,
    ) -> JoinedEachInner<'a, LIt, R, P, L> {
        JoinedEachInner {
            lhs_iter: self,
            rhs: rhs.into(),
            rhs_range: (1, 0),
            predicate,
            current_left: None,
        }
    }

    fn outer_join(
        self,
        rhs: impl Into<RHS<'a, R>>,
        predicate: P,
    ) -> JoinedEachOuter<'a, LIt, R, P, L> {
        JoinedEachOuter {
            lhs_iter: self,
            rhs: rhs.into(),
            current_left: None,
            rhs_range: (1, 0),
            predicate,
        }
    }
}

/// The intermediate result of a semi- or anti-join that will yield `(L, &R)` values.
pub struct JoinedEachInner<'a, LIt, R, P, L> {
    /// User-supplied predicate that accepts (&L, &R) and returns an Ordering
    predicate: P,

    /// Iterator of LHS values which are necessarily borrowed.
    lhs_iter: LIt,

    /// The current LHS value. If None, one will be taken from [lhs_iter]. When no more matches are
    /// found in RHS, this will be set to None again.
    current_left: Option<&'a L>,

    /// The RHS values to search.
    rhs: RHS<'a, R>,

    /// The range in RHS where values will be taken.

    /// If RHS is Unsorted, the range starts off as 0..rhs.len(). If sorted, the values are set
    /// according to a binary search. Upon each iteration, the lower value is updated to restrict
    /// subsequent search space.
    rhs_range: (usize, usize),
}

impl<'a, LIt, R, P, L> Iterator for JoinedEachInner<'a, LIt, R, P, L>
where
    LIt: Iterator<Item = &'a L>,
    L: 'a,
    R: 'a,
    P: Fn(&L, &R) -> std::cmp::Ordering,
    R: std::fmt::Debug,
{
    type Item = (&'a L, &'a R);

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let left: &'a L = if let Some(l) = self.current_left {
                l
            } else if let Some(l) = self.lhs_iter.next() {
                self.rhs_range = self.rhs.get_range(l, &self.predicate);
                self.current_left = Some(l);
                l
            } else {
                // If LHS has no more, then we stop iteration altogether
                self.rhs_range = (1, 0);
                self.current_left = None;
                return None;
            };

            match self.rhs {
                RHS::Unsorted(u) => {
                    for i in self.rhs_range.0..self.rhs_range.1 {
                        if (self.predicate)(left, &u[i]).is_eq() {
                            self.rhs_range.0 = i + 1;
                            return Some((left, &u[i]));
                        }
                    }

                    // No matches remain for this LHS value
                    self.current_left.take();
                }
                RHS::Sorted(s) => {
                    if self.rhs_range.0 < self.rhs_range.1 {
                        // we can use one of these values
                        self.rhs_range.0 += 1;
                        return Some((left, &s[self.rhs_range.0 - 1]));
                    } else {
                        self.current_left.take();
                    }
                }
            }
        }
    }
}

pub struct JoinedEachOuter<'a, LIt, R, P, L> {
    lhs_iter: LIt,
    current_left: Option<&'a L>,
    rhs: RHS<'a, R>,
    rhs_range: (usize, usize),
    predicate: P,
}

impl<'a, LIt, R, P, L> Iterator for JoinedEachOuter<'a, LIt, R, P, L>
where
    LIt: Iterator<Item = &'a L>,
    L: 'a,
    R: 'a,
    P: Fn(&L, &R) -> std::cmp::Ordering,
    R: std::fmt::Debug,
{
    type Item = (&'a L, Option<&'a R>);

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let (left, is_first) = if let Some(l) = self.current_left {
                (l, false)
            } else if let Some(l) = self.lhs_iter.next() {
                self.rhs_range = self.rhs.get_range(l, &self.predicate);
                self.current_left = Some(l);
                (l, true)
            } else {
                // If LHS has no more, then we stop iteration altogether
                self.rhs_range = (1, 0);
                self.current_left = None;
                return None;
            };

            match self.rhs {
                RHS::Unsorted(u) => {
                    for i in self.rhs_range.0..self.rhs_range.1 {
                        if (self.predicate)(left, &u[i]).is_eq() {
                            self.rhs_range.0 = i + 1;
                            return Some((left, Some(&u[i])));
                        }
                    }

                    if is_first {
                        // We pulled the LHS value without any return; indicate no RHS with a None
                        return Some((left, None));
                    } else {
                        // No matches remain for this LHS value
                        self.current_left.take();
                    }
                }
                RHS::Sorted(s) => {
                    if self.rhs_range.0 < self.rhs_range.1 {
                        // we can use one of these values
                        self.rhs_range.0 += 1;
                        return Some((left, Some(&s[self.rhs_range.0 - 1])));
                    } else if is_first {
                        return Some((left, None));
                    } else {
                        self.current_left.take();
                    }
                }
            }
        }
    }
}

#[cfg(test)]
const LEFT_ITEMS: [(usize, &'static str); 12] = [
    (0, "zero"),
    (0, "nil"),
    (1, "one"),
    (2, "two"),
    (3, "three"),
    (4, "four"),
    (5, "five"),
    (6, "six"),
    (7, "seven"),
    (8, "eight"),
    (9, "nine"),
    (10, "ten"),
];

#[cfg(test)]
const RIGHT_ITEMS: [(usize, &'static str); 8] = [
    (0, "zéro"),
    (1, "un"),
    (1, "uno"),
    (1, "ichi"),
    (2, "dos"),
    (2, "deux"),
    (3, "trois"),
    (4, "quatre"),
];

#[test]
fn test_inner() {
    let mut joined = LEFT_ITEMS
        .iter()
        .inner_join(&RIGHT_ITEMS[..], |l, r| l.0.cmp(&r.0));

    assert_eq!(joined.next(), Some((&(0, "zero"), &(0, "zéro"))));
    assert_eq!(joined.next(), Some((&(0, "nil"), &(0, "zéro"))));

    assert_eq!(joined.next(), Some((&(1, "one"), &(1, "un"))));
    assert_eq!(joined.next(), Some((&(1, "one"), &(1, "uno"))));
    assert_eq!(joined.next(), Some((&(1, "one"), &(1, "ichi"))));

    assert_eq!(joined.next(), Some((&(2, "two"), &(2, "dos"))));
    assert_eq!(joined.next(), Some((&(2, "two"), &(2, "deux"))));

    assert_eq!(joined.next(), Some((&(3, "three"), &(3, "trois"))));

    assert_eq!(joined.next(), Some((&(4, "four"), &(4, "quatre"))));

    assert_eq!(joined.next(), None);
}

#[test]
fn test_inner_sorted() {
    let rhs = RHS::Sorted(&RIGHT_ITEMS);
    let mut joined = LEFT_ITEMS.iter().inner_join(rhs, |l, r| l.0.cmp(&r.0));

    assert_eq!(joined.next(), Some((&(0, "zero"), &(0, "zéro"))));
    assert_eq!(joined.next(), Some((&(0, "nil"), &(0, "zéro"))));

    assert_eq!(joined.next(), Some((&(1, "one"), &(1, "un"))));
    assert_eq!(joined.next(), Some((&(1, "one"), &(1, "uno"))));
    assert_eq!(joined.next(), Some((&(1, "one"), &(1, "ichi"))));

    assert_eq!(joined.next(), Some((&(2, "two"), &(2, "dos"))));
    assert_eq!(joined.next(), Some((&(2, "two"), &(2, "deux"))));

    assert_eq!(joined.next(), Some((&(3, "three"), &(3, "trois"))));

    assert_eq!(joined.next(), Some((&(4, "four"), &(4, "quatre"))));

    assert_eq!(joined.next(), None);
}

#[test]
fn test_outer() {
    let mut joined = LEFT_ITEMS
        .iter()
        .outer_join(&RIGHT_ITEMS[..], |l, r| l.0.cmp(&r.0));

    assert_eq!(joined.next(), Some((&(0, "zero"), Some(&(0, "zéro")))));
    assert_eq!(joined.next(), Some((&(0, "nil"), Some(&(0, "zéro")))));

    assert_eq!(joined.next(), Some((&(1, "one"), Some(&(1, "un")))));
    assert_eq!(joined.next(), Some((&(1, "one"), Some(&(1, "uno")))));
    assert_eq!(joined.next(), Some((&(1, "one"), Some(&(1, "ichi")))));

    assert_eq!(joined.next(), Some((&(2, "two"), Some(&(2, "dos")))));
    assert_eq!(joined.next(), Some((&(2, "two"), Some(&(2, "deux")))));

    assert_eq!(joined.next(), Some((&(3, "three"), Some(&(3, "trois")))));

    assert_eq!(joined.next(), Some((&(4, "four"), Some(&(4, "quatre")))));

    // The remaining LHS values have no RHS match, so they'll return None

    assert_eq!(joined.next(), Some((&(5, "five"), None)));
    assert_eq!(joined.next(), Some((&(6, "six"), None)));
    assert_eq!(joined.next(), Some((&(7, "seven"), None)));
    assert_eq!(joined.next(), Some((&(8, "eight"), None)));
    assert_eq!(joined.next(), Some((&(9, "nine"), None)));
    assert_eq!(joined.next(), Some((&(10, "ten"), None)));

    assert_eq!(joined.next(), None);
}

#[test]
fn test_outer_sorted() {
    let rhs = RHS::Sorted(&RIGHT_ITEMS);
    let mut joined = LEFT_ITEMS.iter().outer_join(rhs, |l, r| l.0.cmp(&r.0));

    assert_eq!(joined.next(), Some((&(0, "zero"), Some(&(0, "zéro")))));
    assert_eq!(joined.next(), Some((&(0, "nil"), Some(&(0, "zéro")))));

    assert_eq!(joined.next(), Some((&(1, "one"), Some(&(1, "un")))));
    assert_eq!(joined.next(), Some((&(1, "one"), Some(&(1, "uno")))));
    assert_eq!(joined.next(), Some((&(1, "one"), Some(&(1, "ichi")))));

    assert_eq!(joined.next(), Some((&(2, "two"), Some(&(2, "dos")))));
    assert_eq!(joined.next(), Some((&(2, "two"), Some(&(2, "deux")))));

    assert_eq!(joined.next(), Some((&(3, "three"), Some(&(3, "trois")))));

    assert_eq!(joined.next(), Some((&(4, "four"), Some(&(4, "quatre")))));

    // The remaining LHS values have no RHS match, so they'll return None

    assert_eq!(joined.next(), Some((&(5, "five"), None)));
    assert_eq!(joined.next(), Some((&(6, "six"), None)));
    assert_eq!(joined.next(), Some((&(7, "seven"), None)));
    assert_eq!(joined.next(), Some((&(8, "eight"), None)));
    assert_eq!(joined.next(), Some((&(9, "nine"), None)));
    assert_eq!(joined.next(), Some((&(10, "ten"), None)));

    assert_eq!(joined.next(), None);
}
