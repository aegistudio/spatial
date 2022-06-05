use std::cmp::{max, min, Ord, Ordering};

use crate::{cfg_test, Vec3};

/// AABBRelation shows the relationship between the AABB and the user
/// requested query body.
#[derive(PartialEq, Debug)]
pub enum AABBRelation {
	/// Interleave means the query body is disjoint from the AABB, and
	/// no further intersection test will be performed.
	Interleave,

	/// Intersect means the query body has intersection with the AABB,
	/// and we will furthermore perform intersection tests.
	Intersect,

	/// Include means the query body includes the AABB completely, and
	/// we will yield all elements inside it, without furthermore
	/// performing intersection tests.
	Include,
}

/// AABBQuery defines a query body for picking objects in AABB based
/// indexing structures, like BVH and KDTree.
pub trait AABBQuery<B> {
	fn check(&self, bound: &B) -> AABBRelation;
}

/// AABB3 represents a 3-dimensional axis-aligned bounding box with
/// various spatial operations defined upon it.
#[derive(Copy, Clone, Debug)]
pub struct AABB3<T>(Vec3<(T, T)>);

fn intersect_intervals<T: Ord + Copy>(
	a: (T, T), b: (T, T),
) -> Option<(T, T)> {
	if (&a.0).cmp(&b.0).is_le() && (&b.0).cmp(&a.1).is_le() {
		return Some((b.0, min(a.1, b.1)));
	}
	if (&a.0).cmp(&b.1).is_le() && (&b.1).cmp(&a.1).is_le() {
		return Some((max(a.0, b.0), b.1));
	}
	None
}

fn order_pair<T: Ord>(a: T, b: T) -> (T, T) {
	if a > b {
		(b, a)
	} else {
		(a, b)
	}
}

fn reorder_pair<T>(pair: (T, T), ord: Ordering) -> (T, T) {
	match ord {
		Ordering::Greater => (pair.1, pair.0),
		_ => (pair.0, pair.1),
	}
}

fn is_ne_pair<T: Ord>(a: T, b: T) -> Option<()> {
	(&a).cmp(&b).is_ne().then_some(())
}

impl<T: Ord + Copy> AABB3<T> {
	/// new creates an AABB instance.
	pub fn new(p0: Vec3<T>, p1: Vec3<T>) -> Self {
		Self(p0 / p1 | order_pair)
	}

	/// extends the current AABB with another specified bounding body.
	pub fn extends(&self, a: &Self) -> Self {
		Self(self.0 / a.0 | (|x, y| (min(x.0, y.0), max(x.1, y.1))))
	}

	/// intersects the current AABB with another bounding body.
	///
	/// Please notice that two bounding box shares the same surface is
	/// also considered a case of intersection. This can be eliminated
	/// by testing whether the bounding box is zero volume.
	pub fn intersects(&self, a: &Self) -> Option<Self> {
		Some(Self((self.0 / a.0 & intersect_intervals)?))
	}

	/// is_degraded checks whether the AABB is degraded.
	pub fn is_degraded(&self) -> bool {
		(self.0 & is_ne_pair).is_some()
	}

	/// does_intersects_with checks whether two AABB intersects.
	///
	/// Please notice bare surface intersection is considered to
	/// be disjoint, which simplifies the semantics.
	pub fn does_intersects_with(&self, a: &Self) -> bool {
		if let Some(body) = self.intersects(a) {
			return !body.is_degraded();
		}
		false
	}

	/// from_ordering reorders the vertex of AABB according to the
	/// given ordering.
	///
	/// The returned point pairs will always be on a body diagonal of
	/// the original AABB, with the one on outgoing direction as the
	/// first component.
	#[inline(always)]
	pub fn from_ordering(&self, v: Vec3<Ordering>) -> Vec3<(T, T)> {
		self.0 / v | reorder_pair
	}
}

impl<T: Ord + Copy> From<AABB3<T>> for Vec3<(T, T)> {
	fn from(v: AABB3<T>) -> Vec3<(T, T)> {
		v.0
	}
}

impl<T: Ord + Copy> From<Vec3<(T, T)>> for AABB3<T> {
	fn from(v: Vec3<(T, T)>) -> AABB3<T> {
		let (x, y) = v.unzip();
		AABB3::new(x, y)
	}
}

cfg_test! {
	#[test] fn test_aabb3_i64_new() {
		let a = AABB3::new(
			Vec3::new(1, 2, 3),
			Vec3::new(4, -5, 6),
		);
		assert_eq!(
			Vec3::<(i64, i64)>::from(a),
			Vec3::new((1, 4), (-5, 2), (3, 6)),
		);

		let b = AABB3::from(Vec3::new(
			(-7, 10), (8, 11), (12, -9),
		));
		assert_eq!(
			Vec3::<(i64, i64)>::from(b),
			Vec3::new((-7, 10), (8, 11), (-9, 12)),
		);

		let c = a.extends(&b);
		assert_eq!(
			Vec3::<(i64, i64)>::from(c),
			Vec3::new((-7, 10), (-5, 11), (-9, 12)),
		);
	}

	#[test] fn test_aabb3_i64_intersects() {
		let a = AABB3::new(
			Vec3::new(1, -5, 3),
			Vec3::new(4, 2, 6),
		);
		let b = AABB3::new(
			Vec3::new(-3, 1, -2),
			Vec3::new(2, 6, 5),
		);
		let c = a.intersects(&b).unwrap();
		assert_eq!(
			Vec3::<(i64, i64)>::from(c),
			Vec3::new((1, 2), (1, 2), (3, 5)),
		);
	}
}
