#[allow(unused_macros)]
macro_rules! cfg_test {
	($($i:item)*) => {
		$(
			#[cfg(test)]
			$i
		)*
	};
}
pub(crate) use cfg_test;
