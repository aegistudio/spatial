use std::cmp::min;

use anyhow::Result;
use rand::prng::chacha::{ChaChaCore, ChaChaRng};
use rand::{Rng, SeedableRng};

/// prng creates a new instance of random number generator.
///
/// The caller can utilize the environment SPATIAL_TEST_SEED for
/// setting a seed for randomizing. When benchmarking, they will
/// generate the same series of data so that different cases will
/// operate on the same set of results.
pub fn prng() -> impl Rng {
	let mut seed = [0 as u8; 32];
	let _ = || -> Result<()> {
		let val = std::env::var("SPATIAL_TEST_SEED")?;
		if let Ok(_) =
			base64::decode_config_slice(&val, base64::STANDARD, &mut seed)
		{
			return Ok(());
		}
		let bytes = val.as_bytes();
		for i in 0..min(bytes.len(), 32) {
			seed[i] = bytes[i]
		}
		Ok(())
	}();
	ChaChaRng::from(ChaChaCore::from_seed(seed))
}
