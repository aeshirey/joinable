use crate::rhs::RHS;

enum JoinType {
    Inner,
    Outer,
    Semi,
    Anti,
}

/// A trait allowing the joining of a left-hand side (LHS) and a right-hand side ([RHS]) dataset.
///
/// Results for [inner_join_grouped](JoinableGrouped::inner_join_grouped) and
/// [outer_join_grouped](JoinableGrouped::outer_join_grouped) are individual LHS records and a
/// `Vec<R>`, which can be empty for outer joins if no match is found.
pub trait JoinableGrouped<'a, LIt, R, P, L> {
    /// Joins LHS and RHS, keeping only records from left that have one or more matches in right.
    ///
    /// The specified predicate returns a [std::cmp::Ordering] comparing left and right records.
    ///
    /// Like `outer_join_grouped`, this function returns a `(L, Vec<&R>)` with matching records from
    /// RHS being collected. If multiple records from left match a given record from right,
    /// right records may be returned multiple times.
    fn inner_join_grouped(
        self,
        rhs: impl Into<RHS<'a, R>>,
        predicate: P,
    ) -> JoinedGrouped<'a, LIt, R, P>;

    /// Joins LHS and RHS, keeping _all_ records from left.
    ///
    /// The specified predicate returns a [std::cmp::Ordering] comparing left and right records.
    ///
    /// Like `inner_join_grouped`, this function returns a `(L, Vec<&R>)` with matching records from
    /// RHS being collected. If multiple records from left match a given record from right,
    /// right records may be returned multiple times.
    fn outer_join_grouped(
        self,
        rhs: impl Into<RHS<'a, R>>,
        predicate: P,
    ) -> JoinedGrouped<'a, LIt, R, P>;

    /// Joins LHS and RHS, keeping all records from left that have one or more matches in right.
    ///
    /// The specified predicate returns a [std::cmp::Ordering] comparing left and right records.
    ///
    /// Like `anti_join`, this function only returns left records.
    fn semi_join(self, rhs: impl Into<RHS<'a, R>>, predicate: P) -> JoinedLeft<'a, LIt, R, P>;

    /// Joins LHS and RHS, keeping all records from left that have _no_ matches in right.
    ///
    /// The specified predicate returns a [std::cmp::Ordering] comparing left and right records.
    ///
    /// Like `semi_join`, this function only returns left records.
    fn anti_join(self, rhs: impl Into<RHS<'a, R>>, predicate: P) -> JoinedLeft<'a, LIt, R, P>;
}

impl<'a, LIt, R, P, L> JoinableGrouped<'a, LIt, R, P, L> for LIt
where
    LIt: Iterator<Item = L>,
    L: 'a,
    R: 'a,
    P: Fn(&L, &R) -> std::cmp::Ordering,
{
    fn inner_join_grouped(
        self,
        rhs: impl Into<RHS<'a, R>>,
        predicate: P,
    ) -> JoinedGrouped<'a, LIt, R, P> {
        JoinedGrouped {
            lhs_iter: self,
            rhs: rhs.into(),
            predicate,
            join_type: crate::joined_grouped::JoinType::Inner,
        }
    }

    fn outer_join_grouped(
        self,
        rhs: impl Into<RHS<'a, R>>,
        predicate: P,
    ) -> JoinedGrouped<'a, LIt, R, P> {
        JoinedGrouped {
            lhs_iter: self,
            rhs: rhs.into(),
            predicate,
            join_type: crate::joined_grouped::JoinType::Outer,
        }
    }

    fn semi_join(self, rhs: impl Into<RHS<'a, R>>, predicate: P) -> JoinedLeft<'a, LIt, R, P> {
        JoinedLeft {
            lhs_iter: self,
            rhs: rhs.into(),
            predicate,
            join_type: crate::joined_grouped::JoinType::Semi,
        }
    }

    fn anti_join(self, rhs: impl Into<RHS<'a, R>>, predicate: P) -> JoinedLeft<'a, LIt, R, P> {
        JoinedLeft {
            lhs_iter: self,
            rhs: rhs.into(),
            predicate,
            join_type: crate::joined_grouped::JoinType::Anti,
        }
    }
}

/// The intermediate result of an inner- or outer-join that will yield `(L, Vec<&R>)` values.
pub struct JoinedGrouped<'a, LIt, R, P> {
    /// The iterator over all left-hand side values
    lhs_iter: LIt,

    /// A value giving us access to all right-hand side values
    rhs: RHS<'a, R>,

    /// A comparison predicate: Fn(&L, &R) -> std::cmp::Ordering
    predicate: P,

    /// One of: Inner, Outer, Semi, Anti
    join_type: JoinType,
}

impl<'a, LIt, R, P, L> Iterator for JoinedGrouped<'a, LIt, R, P>
where
    LIt: Iterator<Item = L>,
    L: 'a,
    R: 'a,
    P: Fn(&L, &R) -> std::cmp::Ordering,
{
    type Item = (L, Vec<&'a R>);

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let left = self.lhs_iter.next()?;

            let rs = match self.rhs {
                RHS::Unsorted(inner) => inner
                    .iter()
                    .filter(|r| (self.predicate)(&left, r).is_eq())
                    .collect::<Vec<_>>(),
                RHS::Sorted(inner) => {
                    match inner.binary_search_by(|r| (self.predicate)(&left, r)) {
                        Ok(mut pos) => {
                            let mut rs = Vec::new();
                            // We found *a* match, but it may not be the first one
                            while pos > 0 && (self.predicate)(&left, &inner[pos - 1]).is_eq() {
                                pos -= 1;
                            }

                            // Found the first; now add every one in order until we reach a different one or the end
                            while pos < inner.len() && (self.predicate)(&left, &inner[pos]).is_eq()
                            {
                                rs.push(&inner[pos]);
                                pos += 1;
                            }

                            rs
                        }
                        Err(_) => vec![],
                    }
                }
            };

            match self.join_type {
                JoinType::Inner => {
                    if !rs.is_empty() {
                        return Some((left, rs));
                    }
                }

                JoinType::Outer => return Some((left, rs)),

                JoinType::Semi => unreachable!(),
                JoinType::Anti => unreachable!(),
            }
        }
    }
}

/// The intermediate result of a semi- or anti-join that will yield `L` values.
pub struct JoinedLeft<'a, LIt, R, P> {
    lhs_iter: LIt,
    rhs: RHS<'a, R>,
    predicate: P,
    join_type: JoinType,
}

impl<'a, LIt, R, P, L> Iterator for JoinedLeft<'a, LIt, R, P>
where
    LIt: Iterator<Item = L>,
    P: Fn(&L, &R) -> std::cmp::Ordering,
{
    type Item = L;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let left = self.lhs_iter.next()?;

            let has_right = self.rhs.has_value(&left, &self.predicate);

            match self.join_type {
                JoinType::Semi if has_right => return Some(left),
                JoinType::Anti if !has_right => return Some(left),

                JoinType::Semi => {}
                JoinType::Anti => {}

                JoinType::Inner => unreachable!(),
                JoinType::Outer => unreachable!(),
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
fn test_left_semi() {
    let joined = LEFT_ITEMS
        .iter()
        .semi_join(&RIGHT_ITEMS[..], |l, r| l.0.cmp(&r.0))
        .collect::<Vec<_>>();

    assert_eq!(joined.len(), 6);

    assert_eq!(joined[0], &(0, "zero"));
    assert_eq!(joined[1], &(0, "nil"));
    assert_eq!(joined[2], &(1, "one"));
    assert_eq!(joined[3], &(2, "two"));
    assert_eq!(joined[4], &(3, "three"));
    assert_eq!(joined[5], &(4, "four"));
}

#[test]
fn test_left_anti() {
    let joined = LEFT_ITEMS
        .iter()
        .anti_join(&RIGHT_ITEMS[..], |l, r| l.0.cmp(&r.0))
        .collect::<Vec<_>>();

    assert_eq!(joined.len(), 6);

    assert_eq!(joined[0], &(5, "five"));
    assert_eq!(joined[1], &(6, "six"));
    assert_eq!(joined[2], &(7, "seven"));
    assert_eq!(joined[3], &(8, "eight"));
    assert_eq!(joined[4], &(9, "nine"));
    assert_eq!(joined[5], &(10, "ten"));
}

#[test]
fn test_left_inner_grouped() {
    let joined = LEFT_ITEMS
        .iter()
        .inner_join_grouped(&RIGHT_ITEMS[..], |l, r| l.0.cmp(&r.0))
        .collect::<Vec<_>>();

    assert_eq!(joined.len(), 6);

    let mut it = joined.into_iter();

    assert_eq!(it.next(), Some((&(0, "zero"), vec![&(0, "zéro")])));
    assert_eq!(it.next(), Some((&(0, "nil"), vec![&(0, "zéro")])));
    assert_eq!(
        it.next(),
        Some((&(1, "one"), vec![&(1, "un"), &(1, "uno"), &(1, "ichi")]))
    );
    assert_eq!(
        it.next(),
        Some((&(2, "two"), vec![&(2, "dos"), &(2, "deux")]))
    );
    assert_eq!(it.next(), Some((&(3, "three"), vec![&(3, "trois")])));
    assert_eq!(it.next(), Some((&(4, "four"), vec![&(4, "quatre")])));
}

#[test]
fn test_left_outer_grouped() {
    let joined = LEFT_ITEMS
        .iter()
        .outer_join_grouped(&RIGHT_ITEMS[..], |l, r| l.0.cmp(&r.0))
        .collect::<Vec<_>>();

    assert_eq!(joined.len(), 12);

    let mut it = joined.into_iter();

    assert_eq!(it.next(), Some((&(0, "zero"), vec![&(0, "zéro")])));
    assert_eq!(it.next(), Some((&(0, "nil"), vec![&(0, "zéro")])));
    assert_eq!(
        it.next(),
        Some((&(1, "one"), vec![&(1, "un"), &(1, "uno"), &(1, "ichi")]))
    );
    assert_eq!(
        it.next(),
        Some((&(2, "two"), vec![&(2, "dos"), &(2, "deux")]))
    );
    assert_eq!(it.next(), Some((&(3, "three"), vec![&(3, "trois")])));
    assert_eq!(it.next(), Some((&(4, "four"), vec![&(4, "quatre")])));

    // No matches here
    assert_eq!(it.next(), Some((&(5, "five"), vec![])));
    assert_eq!(it.next(), Some((&(6, "six"), vec![])));
    assert_eq!(it.next(), Some((&(7, "seven"), vec![])));
    assert_eq!(it.next(), Some((&(8, "eight"), vec![])));
    assert_eq!(it.next(), Some((&(9, "nine"), vec![])));
    assert_eq!(it.next(), Some((&(10, "ten"), vec![])));
}
