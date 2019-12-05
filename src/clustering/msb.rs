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

impl MostSignificantBit for u64 {
    fn msb(self) -> usize {
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
    }
}

impl MostSignificantBit for u32 {
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
