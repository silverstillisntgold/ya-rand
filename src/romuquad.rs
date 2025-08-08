use crate::rng::{Generator, SeedableGenerator};
use crate::util;

/// Rust implementation of the RomuQuad PRNG.
///
/// This generator is extremely fast, high-quality, and small,
/// but not cryptographically secure.
///
/// More information can be found at: <https://romu-random.org/>.
#[derive(Debug, PartialEq, Eq)]
pub struct RomuQuad {
    state: [u64; 4],
}

impl Default for RomuQuad {
    fn default() -> Self {
        Self::new_with_seed(0)
    }
}

impl SeedableGenerator for RomuQuad {
    fn new_with_seed(seed: u64) -> Self {
        let state = util::state_from_seed(seed);
        let mut ret = Self { state };
        let _discard_first = ret.u64();
        ret
    }
}

impl Generator for RomuQuad {
    #[inline]
    fn try_new() -> Result<Self, getrandom::Error> {
        let state = util::state_from_entropy()?;
        Ok(Self { state })
    }

    #[cfg_attr(feature = "inline", inline)]
    fn u64(&mut self) -> u64 {
        let tmp = self.state;
        self.state[0] = tmp[3].wrapping_mul(15241094284759029579);
        self.state[1] = tmp[3].wrapping_add(tmp[0].rotate_left(52));
        self.state[2] = tmp[2].wrapping_sub(tmp[1]);
        self.state[3] = tmp[2].wrapping_add(tmp[0]).rotate_left(19);
        tmp[1]
    }
}
