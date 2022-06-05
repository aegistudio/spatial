use std::cmp::Ordering;
use std::ops::{Add, BitAnd, BitOr, BitXor, Div, Mul, Sub};

use crate::cfg_test;

cfg_test! {
	extern crate test;

	use rand::Rng;
	use crate::prng;
	use test::Bencher;
}

/// Vec3 represents a three-dimensional vector, offered with some
/// convenient operations.
#[derive(Copy, Clone, Debug)]
pub struct Vec3<T>(pub T, pub T, pub T);

impl<T> Vec3<T> {
	#[inline(always)]
	pub fn new(v1: T, v2: T, v3: T) -> Self {
		Self(v1, v2, v3)
	}
}

cfg_test! {
	pub(crate) fn gen_vec3_i64(rng: &mut impl Rng) -> Vec3<i64> {
		Vec3::new(
			(rng.gen::<i32>() / 2) as i64,
			(rng.gen::<i32>() / 2) as i64,
			(rng.gen::<i32>() / 2) as i64,
		)
	}

	pub(crate) fn testdata_vec3_i64(size: usize) -> Vec<Vec3<i64>> {
		let rng = &mut prng();
		let mut vs = Vec::<Vec3<i64>>::new();
		for _ in 0..size {
			vs.push(gen_vec3_i64(rng));
		}
		vs
	}

	fn fixture_bench_vec3_i64<T: Sized>(
		b: &mut Bencher,
		f: impl Fn(Vec3<i64>, Vec3<i64>) -> T + Copy,
	) {
		const POW2: usize = 1<<10;
		let vs = testdata_vec3_i64(POW2);
		let mut i = 0;
		b.iter(|| {
			let j = i;
			i = (i + 2) & (POW2 - 1);
			f(vs[j], vs[j + 1])
		});
	}

	#[bench] fn bench_vec3_i64_ground(b: &mut Bencher) {
		fixture_bench_vec3_i64(b, |_, _| ());
	}
}

impl<T> From<Vec3<T>> for (T, T, T) {
	#[inline(always)]
	fn from(v: Vec3<T>) -> (T, T, T) {
		(v.0, v.1, v.2)
	}
}

impl<T> From<(T, T, T)> for Vec3<T> {
	#[inline(always)]
	fn from(v: (T, T, T)) -> Vec3<T> {
		Self(v.0, v.1, v.2)
	}
}

/// BitOr is bitwise mapping of each components.
impl<T, U, F: FnMut<T, Output = U>> BitOr<F> for Vec3<T> {
	type Output = Vec3<U>;
	#[inline(always)]
	fn bitor(self, mut f: F) -> Self::Output {
		Vec3::new(
			f.call_mut(self.0),
			f.call_mut(self.1),
			f.call_mut(self.2),
		)
	}
}

/// BitAnd is failfast bitwise mapping of each components.
impl<T, U: Sized, F: FnMut<T, Output = Option<U>>> BitAnd<F> for Vec3<T> {
	type Output = Option<Vec3<U>>;
	#[inline(always)]
	fn bitand(self, mut f: F) -> Self::Output {
		Some(Vec3::new(
			f.call_mut(self.0)?,
			f.call_mut(self.1)?,
			f.call_mut(self.2)?,
		))
	}
}

/// Div defines the zip operation for joining vector components of two
/// vectors into tuples bitwisely.
impl<T, U> Div<Vec3<U>> for Vec3<T> {
	type Output = Vec3<(T, U)>;
	#[inline(always)]
	fn div(self, a: Vec3<U>) -> Self::Output {
		Vec3::new((self.0, a.0), (self.1, a.1), (self.2, a.2))
	}
}

impl<T, U> Vec3<(T, U)> {
	/// unzip separate the vector of two components back to two vectors.
	#[inline(always)]
	pub fn unzip(self) -> (Vec3<T>, Vec3<U>) {
		(
			Vec3::new(self.0 .0, self.1 .0, self.2 .0),
			Vec3::new(self.0 .1, self.1 .1, self.2 .1),
		)
	}
}

/// Add defines the vector add for vectors.
impl<U, T: Add<S, Output = U>, S> Add<Vec3<S>> for Vec3<T> {
	type Output = Vec3<U>;
	#[inline(always)]
	fn add(self, a: Vec3<S>) -> Self::Output {
		self / a | (|x, y| x + y)
	}
}

cfg_test! {
	#[test] fn test_vec3_i64_add() {
		let v1 = Vec3::new(1, 2, 3);
		let v2 = Vec3::new(4, 5, 6);
		let v = v1 + v2;
		assert_eq!(v.0, 5);
		assert_eq!(v.1, 7);
		assert_eq!(v.2, 9);
	}

	#[bench] fn bench_vec3_i64_add(b: &mut Bencher) {
		fixture_bench_vec3_i64(b, |x, y| x + y);
	}
}

/// Sub defines the vector sub for vectors.
impl<U, T: Sub<S, Output = U>, S> Sub<Vec3<S>> for Vec3<T> {
	type Output = Vec3<U>;
	#[inline(always)]
	fn sub(self, a: Vec3<S>) -> Self::Output {
		self / a | (|x, y| x - y)
	}
}

/// Mul between vectors define the cross product operation.
impl<V, U, T, S> Mul<Vec3<S>> for Vec3<T>
where
	U: Sub<Output = V>,
	T: Copy + Mul<S, Output = U>,
	S: Copy,
{
	type Output = Vec3<V>;
	#[inline(always)]
	fn mul(self, a: Vec3<S>) -> Self::Output {
		Vec3::new(
			self.1 * a.2 - self.2 * a.1,
			self.2 * a.0 - self.0 * a.2,
			self.0 * a.1 - self.1 * a.0,
		)
	}
}

cfg_test! {
	#[test] fn test_vec3_i64_cross() {
		let v1 = Vec3::new(1, 2, 3);
		let v2 = Vec3::new(4, 5, 6);
		let v = v1 * v2;
		assert_eq!(v.0, -3);
		assert_eq!(v.1, 6);
		assert_eq!(v.2, -3);
	}

	#[bench] fn bench_vec3_i64_cross(b: &mut Bencher) {
		fixture_bench_vec3_i64(b, |x, y| x * y);
	}
}

/// BitXor defines the vector dot product operation.
impl<U, T, S> BitXor<Vec3<S>> for Vec3<T>
where
	U: Add<Output = U>,
	T: Mul<S, Output = U>,
{
	type Output = U;
	#[inline(always)]
	fn bitxor(self, a: Vec3<S>) -> Self::Output {
		let v = self / a | (|x, y| x * y);
		v.0 + v.1 + v.2
	}
}

cfg_test! {
	#[test] fn test_vec3_i64_dot() {
		let v1 = Vec3::new(1, 2, 3);
		let v2 = Vec3::new(4, 5, 6);
		assert_eq!(v1 ^ v2, 32);
	}

	#[bench] fn bench_vec3_i64_dot(b: &mut Bencher) {
		fixture_bench_vec3_i64(b, |x, y| x ^ y);
	}
}

impl<T: Copy + Default> Default for Vec3<T> {
	#[inline(always)]
	fn default() -> Self {
		let zero = T::default();
		Vec3::new(zero, zero, zero)
	}
}

impl<T: Copy + Ord + Default> Vec3<T> {
	/// to_ordering evaluates the spatial orientation of an vector.
	///
	/// This is usually combined with struct like AABB to pick up
	/// most proximate point for intersection comparison, which
	/// eliminate the need for comparing all points.
	///
	/// Luckily we the parity of each components is enough for judging
	/// the orientations, so there's no need for normalizing.
	#[inline(always)]
	pub fn to_ordering(self) -> Vec3<Ordering> {
		let zero = T::default();
		Vec3::new(
			(&self.0).cmp(&zero),
			(&self.1).cmp(&zero),
			(&self.2).cmp(&zero),
		)
	}
}

impl<T: Copy + Eq> PartialEq for Vec3<T> {
	fn eq(&self, a: &Self) -> bool {
		((*self) / (*a) & (|x, y| (x == y).then_some(()))).is_some()
	}
}

impl<T: Copy + Eq> Eq for Vec3<T> {}
