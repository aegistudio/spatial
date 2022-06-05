use std::cmp::Ordering;
use std::ops::{Add, Mul};

use crate::{cfg_test, AABBQuery, AABBRelation, Vec3, AABB3};

cfg_test! {
	extern crate test;

	use std::ops::Sub;

	use test::Bencher;
	use crate::{prng, gen_vec3_i64};
}

/// Plane3 is a three dimensional plane denoted by a point in the
/// plane and the normal of the plane.
///
/// The plane binary partitions the space into three parts, the
/// points on the plane, the points below the plane (with respect
/// to the normal vector), and the points above the plane. We think
/// of those below the plane as inside an infinite body whose
/// surface is the plane.
///
/// This is query oriented object, and we'll precompute as many
/// value as possible, expecting a bulk of objects to be compared.
#[derive(Copy, Clone, Debug)]
pub struct Plane3<T, U> {
	normal: Vec3<T>,
	dir: Vec3<Ordering>,
	distance: U,
}

impl<T, U> Plane3<T, U>
where
	T: Ord + Copy + Mul<Output = U> + Default,
	U: Add<Output = U>,
{
	#[inline(always)]
	pub fn new(point: Vec3<T>, normal: Vec3<T>) -> Self {
		Self {
			normal: normal,
			dir: normal.to_ordering(),
			distance: point ^ normal,
		}
	}
}

impl<T, U> AABBQuery<AABB3<T>> for Plane3<T, U>
where
	T: Ord + Copy + Mul<Output = U>,
	U: Ord + Copy + Add<Output = U>,
{
	#[inline(always)]
	fn check(&self, bound: &AABB3<T>) -> AABBRelation {
		let (vn, vp) = bound.from_ordering(self.dir).unzip();
		let dp = vp ^ self.normal;
		let dn = vn ^ self.normal;
		if dp > self.distance {
			if dn >= self.distance {
				return AABBRelation::Interleave;
			}
		} else {
			if dn < self.distance {
				return AABBRelation::Include;
			}
		}
		AABBRelation::Intersect
	}
}

cfg_test! {
	fn testdata_aabb3_plane3_i64(
		size: usize,
	) -> Vec<(Vec3<i64>, Vec3<i64>, AABB3<i64>)> {
		let rng = &mut prng();
		let mut result = Vec::new();
		for _ in 0..size {
			// Generate and regenerate normal vectors.
			let point = gen_vec3_i64(rng);
			let mut normal = gen_vec3_i64(rng);
			while (normal ^ normal) == 0 {
				normal = gen_vec3_i64(rng);
			}
			let v1 = gen_vec3_i64(rng);
			let v2 = gen_vec3_i64(rng);
			let aabb = AABB3::new(v1, v2);
			result.push((point, normal, aabb));
		}
		result
	}

	// PlaneNaive3 performs the naive comparison with the
	// query bodies. We will also generate random data for
	// comparing the queries results.
	struct PlaneNaive3<T> {
		point: Vec3<T>,
		normal: Vec3<T>,
	}

	impl<T: Copy> PlaneNaive3<T> {
		fn new(point: Vec3<T>, normal: Vec3<T>) -> Self {
			Self{
				point: point,
				normal: normal,
			}
		}
	}

	impl<T, U, V> AABBQuery<AABB3<T>> for PlaneNaive3<T>
	where
		T: Ord + Copy + Sub<Output = U> + Default,
		U: Copy + Mul<T, Output = V>,
		V: Copy + Add<Output = V> + Ord + Default,
	{
		#[inline(always)]
		fn check(&self, bound: &AABB3<T>) -> AABBRelation {
			let zero = V::default();
			let v = Vec3::<(T, T)>::from(*bound);
			let vs : [Vec3<T>; 8] = [
				Vec3::new(v.0 .0, v.1 .0, v.2 .0),
				Vec3::new(v.0 .0, v.1 .0, v.2 .1),
				Vec3::new(v.0 .0, v.1 .1, v.2 .0),
				Vec3::new(v.0 .0, v.1 .1, v.2 .1),
				Vec3::new(v.0 .1, v.1 .0, v.2 .0),
				Vec3::new(v.0 .1, v.1 .0, v.2 .1),
				Vec3::new(v.0 .1, v.1 .1, v.2 .0),
				Vec3::new(v.0 .1, v.1 .1, v.2 .1),
			];

			let mut less = false;
			let mut equal = false;
			let mut greater = false;
			for v in vs {
				let d = (v - self.point) ^ self.normal;
				match &d.cmp(&zero) {
					Ordering::Less => {
						if greater {
							return AABBRelation::Intersect;
						}
						less = true;
					},
					Ordering::Equal => equal = true,
					Ordering::Greater => {
						if less {
							return AABBRelation::Intersect;
						}
						greater = true;
					},
				}
			}
			if greater {
				AABBRelation::Interleave
			} else if less {
				AABBRelation::Include
			} else {
				if !equal {
					unreachable!();
				}
				AABBRelation::Intersect
			}
		}
	}

	#[test] fn test_plane3_i64_random_query() {
		const NUM: usize = 1000000;
		let data = testdata_aabb3_plane3_i64(NUM);
		let mut include = 0usize;
		let mut intersect = 0usize;
		let mut interleave = 0usize;
		for (p, n, aabb) in data {
			let actual = Plane3::new(p, n).check(&aabb);
			let expected = PlaneNaive3::new(p, n).check(&aabb);
			assert_eq!(
				actual, expected,
				"point = {:?}, normal = {:?}, aabb = {:?}",
				p, n, aabb,
			);
			match actual {
				AABBRelation::Include => include += 1,
				AABBRelation::Intersect => intersect += 1,
				AABBRelation::Interleave => interleave += 1,
			}
		}
		println!(
			"include = {}, intersect = {}, interleave = {}",
			include, intersect, interleave,
		);
	}

	fn fixture_bench_plane3_i64<Q, F>(
		b: &mut Bencher, f: F,
	)
	where
		Q: AABBQuery<AABB3<i64>> + Sized,
		F: Fn(Vec3<i64>, Vec3<i64>) -> Q,
	{
		const POW2: usize = 1 << 16;
		let vs = testdata_aabb3_plane3_i64(POW2);
		let mut i = 0;
		b.iter(|| {
			let j = i;
			i = (i + 1) & (POW2 - 1);
			let (p, n, aabb) = vs[j];
			f(p, n).check(&aabb)
		});
	}

	#[bench] fn bench_plane3_i64_query(b: &mut Bencher) {
		fixture_bench_plane3_i64(b, Plane3::new);
	}

	#[bench] fn bench_plane_naive3_i64_query(b: &mut Bencher) {
		fixture_bench_plane3_i64(b, PlaneNaive3::new);
	}

	fn fixture_bench_plane3_i64_bulky<Q, F>(
		b: &mut Bencher, f: F,
	)
	where
		Q: AABBQuery<AABB3<i64>> + Sized,
		F: Fn(Vec3<i64>, Vec3<i64>) -> Q,
	{
		const POW2: usize = 1 << 16;
		let vs = testdata_aabb3_plane3_i64(POW2);
		let mut i = 0;
		let query = f(vs[0].0, vs[0].1);
		b.iter(|| {
			let j = i;
			i = (i + 1) & (POW2 - 1);
			query.check(&vs[j].2)
		});
	}

	#[bench] fn bench_plane3_i64_bulky_query(b: &mut Bencher) {
		fixture_bench_plane3_i64_bulky(b, Plane3::new);
	}

	#[bench] fn bench_plane_naive3_i64_bulky_query(b: &mut Bencher) {
		fixture_bench_plane3_i64_bulky(b, PlaneNaive3::new);
	}
}
