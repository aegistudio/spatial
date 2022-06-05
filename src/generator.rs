use std::iter::Iterator;
use std::ops::{Generator, GeneratorState};
use std::pin::Pin;

pub(crate) struct Enumerator<G> {
	g: G,
}

impl<'a, X, G> Enumerator<G>
where
	G: Generator<(), Yield = X, Return = ()> + Unpin,
{
	pub fn new(g: G) -> Self {
		Self { g: g }
	}
}

impl<'a, X, G> Iterator for Enumerator<G>
where
	G: Generator<(), Yield = X, Return = ()> + Unpin,
{
	type Item = X;

	fn next(&mut self) -> Option<X> {
		match Pin::new(&mut self.g).resume(()) {
			GeneratorState::Yielded(x) => Some(x),
			GeneratorState::Complete(_) => None,
		}
	}
}
