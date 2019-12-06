/// Provide `msb` method for numeric types to obtain the zero-based
/// position of the most significant bit set.
/// 
/// Algorithms used based on this article: 
/// https://prismoskills.appspot.com/lessons/Bitwise_Operators/Find_position_of_MSB.jsp
pub trait MostSignificantBit {
    /// Get the zero-based position of the most significant bit of an integer type.
    /// If the number is zero, return zero. 
    /// 
    /// ## Examples: 
    /// 
    /// ```
    ///    use clusterphobia::clustering::msb::MostSignificantBit;
    /// 
    ///    assert!(0_u64.msb() == 0);
    ///    assert!(1_u64.msb() == 0);
    ///    assert!(2_u64.msb() == 1);
    ///    assert!(3_u64.msb() == 1);
    ///    assert!(4_u64.msb() == 2);
    ///    assert!(255_u64.msb() == 7);
    ///    assert!(1023_u64.msb() == 9);
    /// ```
    fn msb(self) -> usize;
}

/*
#[inline]
/// Return whether floor(log2(x))!=floor(log2(y))
/// with zero for false and 1 for true, because this came from C!
fn ld_neq(x : u64, y : u64) -> u64 {
    let neq = (x^y) > (x&y);
    if neq { 1 } else { 0 }
}
*/

impl MostSignificantBit for u64 {
    #[inline]
    fn msb(self) -> usize {
        // FIRST ATTEMPT:

        /*
        // Bisection guarantees performance of O(Log B) where B is number of bits in integer.
        let mut high = 63_usize;
        let mut low = 0_usize;
        while (high - low) > 1
        {
            let mid = (high+low)/2;
            let mask_high = (1 << high) - (1 << mid);
            if (mask_high & self) != 0 { low = mid; }
            else { high = mid; }
        }
        low
        */
   
        // SECOND ATTEMPT:

        /*
        // This algorithm found on pg 16 of "Matters Computational" at  https://www.jjj.de/fxt/fxtbook.pdf
        // It avoids most if-branches and has no looping.
        // Using this instead of Bisection and looping shaved off 1/3 of the time.
        const MU0 : u64 = 0x5555555555555555; // MU0 == ((-1UL)/3UL) == ...01010101_2
        const MU1 : u64 = 0x3333333333333333; // MU1 == ((-1UL)/5UL) == ...00110011_2
        const MU2 : u64 = 0x0f0f0f0f0f0f0f0f; // MU2 == ((-1UL)/17UL) == ...00001111_2
        const MU3 : u64 = 0x00ff00ff00ff00ff; // MU3 == ((-1UL)/257UL) == (8 ones)
        const MU4 : u64 = 0x0000ffff0000ffff; // MU4 == ((-1UL)/65537UL) == (16 ones)
        const MU5 : u64 = 0x00000000ffffffff; // MU5 == ((-1UL)/4294967297UL) == (32 ones)
        let r : u64 = ld_neq(self, self & MU0)
        + (ld_neq(self, self & MU1) << 1)
        + (ld_neq(self, self & MU2) << 2)
        + (ld_neq(self, self & MU3) << 3)
        + (ld_neq(self, self & MU4) << 4)
        + (ld_neq(self, self & MU5) << 5);
        r as usize
       */

        // THIRD ATTEMPT
        let z = self.leading_zeros();
        if z == 64 { 0 }
        else { 63 - z as usize }
    }
}

impl MostSignificantBit for u32 {
    #[inline]
    fn msb(self) -> usize {
        // Bisection guarantees performance of O(Log B) where B is number of bits in integer.
        let mut high = 31_usize;
        let mut low = 0_usize;
        while (high - low) > 1
        {
            let mid = (high+low)/2;
            let mask_high = (1 << high) - (1 << mid);
            if (mask_high & self) != 0 { low = mid; }
            else { high = mid; }
        }
        low
    }
}

impl MostSignificantBit for u16 {
    #[inline]
    fn msb(self) -> usize {
        // Bisection guarantees performance of O(Log B) where B is number of bits in integer.
        let mut high = 15_usize;
        let mut low = 0_usize;
        while (high - low) > 1
        {
            let mid = (high+low)/2;
            let mask_high = (1 << high) - (1 << mid);
            if (mask_high & self) != 0 { low = mid; }
            else { high = mid; }
        }
        low
    }
}

#[cfg(test)]
/// Tests of the Clustering methods.
mod tests {
    #[allow(unused_imports)]
    use spectral::prelude::*;
    use crate::clustering::msb::MostSignificantBit;

    #[test] 
    fn msb_for_u64()
    {
        let cases : Vec<(u64, usize)> = vec![(0,0), (1,0), (2,1), (3,1), (4,2), (255,7), (1023, 9), (1024,10)];
        for (number, expected_position) in cases {
            let actual_position = number.msb();
            asserting(&format!("For input {}, expect MSB of {} but got {}", number, expected_position, actual_position))
                .that(&actual_position)
                .is_equal_to(expected_position);
        }
    }

    #[test] 
    fn msb_for_u32()
    {
        let cases : Vec<(u32, usize)> = vec![(0,0), (1,0), (2,1), (3,1), (4,2), (255,7), (1023, 9), (1024,10)];
        for (number, expected_position) in cases {
            let actual_position = number.msb();
            asserting(&format!("For input {}, expect MSB of {} but got {}", number, expected_position, actual_position))
                .that(&actual_position)
                .is_equal_to(expected_position);
        }
    }

    #[test] 
    fn msb_for_u16()
    {
        let cases : Vec<(u16, usize)> = vec![(0,0), (1,0), (2,1), (3,1), (4,2), (255,7), (1023, 9), (1024,10)];
        for (number, expected_position) in cases {
            let actual_position = number.msb();
            asserting(&format!("For input {}, expect MSB of {} but got {}", number, expected_position, actual_position))
                .that(&actual_position)
                .is_equal_to(expected_position);
        }
    }

}
