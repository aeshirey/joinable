use crate::{
    joined::{Joined, JoinedLeft},
    rhs::RHS,
};

/// A trait allowing the joining of a left-hand side (LHS) and a right-hand side ([RHS]) dataset.
pub trait Joinable<'a, LIt, R, P> {
    /// Joins LHS and RHS, keeping only records from left that have one or more matches in right.
    /// 
    /// The specified predicate returns a [std::cmp::Ordering] comparing left and right records.
    /// 
    /// Like [Joinable::outer_join], this function returns a `(L, Vec<&R>)` with matching records from
    /// RHS being collected. If multiple records from left match a given record from right,
    /// right records may be returned multiple times.
    fn inner_join(self, rhs: impl Into<RHS<'a, R>>, predicate: P) -> Joined<'a, LIt, R, P>;

    /// Joins LHS and RHS, keeping _all_ records from left.
    /// 
    /// The specified predicate returns a [std::cmp::Ordering] comparing left and right records.
    /// 
    /// Like [Joinable::inner_join], this function returns a `(L, Vec<&R>)` with matching records from
    /// RHS being collected. If multiple records from left match a given record from right,
    /// right records may be returned multiple times.
    fn outer_join(self, rhs: impl Into<RHS<'a, R>>, predicate: P) -> Joined<'a, LIt, R, P>;

    /// Joins LHS and RHS, keeping all records from left that have one or more matches in right.
    /// 
    /// The specified predicate returns a [std::cmp::Ordering] comparing left and right records.
    /// 
    /// Like [Joinable::anti_join], this function only returns left records.
    fn semi_join(self, rhs: impl Into<RHS<'a, R>>, predicate: P) -> JoinedLeft<'a, LIt, R, P>;

    /// Joins LHS and RHS, keeping all records from left that have _no_ matches in right.
    /// 
    /// The specified predicate returns a [std::cmp::Ordering] comparing left and right records.
    /// 
    /// Like [Joinable::semi_join], this function only returns left records.
    fn anti_join(self, rhs: impl Into<RHS<'a, R>>, predicate: P) -> JoinedLeft<'a, LIt, R, P>;
}

impl<'a, LIt, R, P, L> Joinable<'a, LIt, R, P> for LIt
where
    LIt: Iterator<Item = L>,
    L: 'a,
    R: 'a,
    P: Fn(&L, &R) -> std::cmp::Ordering,
{
    fn inner_join(self, rhs: impl Into<RHS<'a, R>>, predicate: P) -> Joined<'a, LIt, R, P> {
        Joined {
            lhs_iter: self,
            rhs: rhs.into(),
            predicate,
            join_type: crate::joined::JoinType::Inner,
        }
    }

    fn outer_join(self, rhs: impl Into<RHS<'a, R>>, predicate: P) -> Joined<'a, LIt, R, P> {
        Joined {
            lhs_iter: self,
            rhs: rhs.into(),
            predicate,
            join_type: crate::joined::JoinType::Outer,
        }
    }

    fn semi_join(self, rhs: impl Into<RHS<'a, R>>, predicate: P) -> JoinedLeft<'a, LIt, R, P> {
        JoinedLeft {
            lhs_iter: self,
            rhs: rhs.into(),
            predicate,
            join_type: crate::joined::JoinType::Semi,
        }
    }

    fn anti_join(self, rhs: impl Into<RHS<'a, R>>, predicate: P) -> JoinedLeft<'a, LIt, R, P> {
        JoinedLeft {
            lhs_iter: self,
            rhs: rhs.into(),
            predicate,
            join_type: crate::joined::JoinType::Anti,
        }
    }
}
