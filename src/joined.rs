use crate::rhs::RHS;

pub(crate) enum JoinType {
    Inner,
    Outer,
    Semi,
    Anti,
}

/// The intermediate result of an inner- or outer-join that will yield `(L, Vec<&R>)` values.
pub struct Joined<'a, LIt, R, P> {
    /// The iterator over all left-hand side values
    pub(crate) lhs_iter: LIt,

    /// A value giving us access to all right-hand side values
    pub(crate) rhs: RHS<'a, R>,

    /// A comparison predicate: Fn(&L, &R) -> std::cmp::Ordering
    pub(crate) predicate: P,

    /// One of: Inner, Outer, Semi, Anti
    pub(crate) join_type: JoinType,
}

impl<'a, LIt, R, P, L> Iterator for Joined<'a, LIt, R, P>
where
    LIt: Iterator<Item = L>,
    L: 'a,
    R: 'a,
    P: Fn(&L, &R) -> std::cmp::Ordering,

    R: std::fmt::Debug,
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
    pub(crate) lhs_iter: LIt,
    pub(crate) rhs: RHS<'a, R>,
    pub(crate) predicate: P,
    pub(crate) join_type: JoinType,
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
