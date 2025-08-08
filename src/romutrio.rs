use crate::rng::{Generator, SeedableGenerator};
use crate::util;

/// Rust implementation of the RomuTrio PRNG.
///
/// This generator is extremely fast, high-quality, and small,
/// but not cryptographically secure.
///
/// More information can be found at: <https://romu-random.org/>.
#[derive(Debug, PartialEq, Eq)]
pub struct RomuTrio {
    state: [u64; 3],
}

impl Default for RomuTrio {
    fn default() -> Self {
        Self::new_with_seed(0)
    }
}

impl SeedableGenerator for RomuTrio {
    fn new_with_seed(seed: u64) -> Self {
        let state = util::state_from_seed(seed);
        let mut ret = Self { state };
        let _discard_first = ret.u64();
        ret
    }
}

impl Generator for RomuTrio {
    #[inline]
    fn try_new() -> Result<Self, getrandom::Error> {
        let state = util::state_from_entropy()?;
        Ok(Self { state })
    }

    #[cfg_attr(feature = "inline", inline)]
    fn u64(&mut self) -> u64 {
        let tmp = self.state;
        self.state[0] = tmp[2].wrapping_mul(15241094284759029579);
        self.state[1] = tmp[1].wrapping_sub(tmp[0]).rotate_left(12);
        self.state[2] = tmp[2].wrapping_sub(tmp[1]).rotate_left(44);
        tmp[0]
    }
}
