use std::f64;
use super::msb::MostSignificantBit;

/// Approximate the natural logarithm of the ratio of two unsigned integers to an accuracy of ±0.000025.
/// 
/// The algorithm follows this article: http://www.nezumi.demon.co.uk/consult/logx.htm):
/// 
/// ## Algorithm
///  
///   1. Range reduction to the interval [1, 2] by dividing by the largest power of two not exceeding the value:
///      - Change representation of numerator   → `2ⁿ·N where 1 ≤ N ≤ 2`
///      - Change representation of denominator → `2ᵈ·D where 1 ≤ D ≤ 2`
///   2. This makes the result `log(numerator/denominator) = log(2ⁿ·N / 2ᵈ·D) = (n-d)·log(2) + log(N) - log(D)`
///   3. To perform log(N), Taylor series does not converge in the neighborhood of zero, but it does near one...
///   4. ... since N is near one, substitute x = N - 1 so that we now need to evaluate log(1 + x)
///   5. Perform a substitution of `y = x/(2+x)`
///   6. Consider the related function `f(y) = Log((1+y)/(1-y))`
///      - `= Log((1 + x/(2+x)) / (1 - x/(2+x)))`
///      - `= Log( (2+2x) / 2)`
///      - `= Log(1 + x)`
///   7. f(y) has a Taylor Expansion which converges must faster than the expansion for Log(1+x) ... 
///      - For Log(1+x) → `x - x²/2 + x³/3 - y⁴/4 + ...`
///      - For Log((1+y)/(1-y)) → `y + y³/3 + y⁵/5 + ...`
///   8. Use the Padé Approximation for the truncated series `y + y³/3 + y⁵/5 ...`
///   9. ... Which is `2y·(15 - 4y²)/(15 - 9y²)`
///   10. Repeat for the denominator and combine the results.
/// 
/// ## Error Range
/// 
/// The interesting thing is to compare the error bars for the _Taylor series_ and its _Padé Approximation_: 
///    - Padé Approximation error is ±0.000025
///    - Taylor series has error ±0.00014 (five times worse)
pub fn log_ratio(numerator : u64, denominator : u64) -> f64 {
    // Ln(2) comes from The On-line Encyclopedia of Integer Sequences https://oeis.org/A002162
    const LOG2 : f64 = 0.6931471805599453; 
    if numerator == 0 || denominator == 0 { return f64::NAN; }

    // Range reduction 
    let n = numerator.msb();
    let d = denominator.msb();
    let reduced_numerator = numerator as f64 / (1 << n) as f64;
    let reduced_denominator = denominator as f64 / (1 << d) as f64;

    // Calculate logs of the products and dividends and combine.
    // To reduce from two calls to log_1_plus_x to a single call,
    //   - if reduced_numerator / reduced_denominator >= 1 use it as the operand and add the result,
    //   - otherwise use the inverse as the operand to log_1_plus_x and subtract the result.  
    let log_fraction = 
        if reduced_numerator >= reduced_denominator { 
            log_1_plus_x((reduced_numerator / reduced_denominator) - 1.0) 
        }
        else {
            -log_1_plus_x((reduced_denominator / reduced_numerator) - 1.0)
        };
    let approximate_log = (n as f64 - d as f64) * LOG2 + log_fraction;
    
    // let approximate_log = (n as f64 - d as f64) * LOG2 + log_1_plus_x(reduced_numerator - 1.0) - log_1_plus_x(reduced_denominator - 1.0);
    approximate_log
}

/// Approximate the natural logarithm of one plus a number in the range (0..1). 
/// 
/// Use a Padé Approximation for the truncated Taylor series for Log((1+y)/(1-y)).
/// 
///   - x - must be a value between zero and one, inclusive.
#[inline]
fn log_1_plus_x(x : f64) -> f64 {
    // This is private and its caller already checks for negatives, so no need to check again here. 
    // Also, though ln(1 + 0) == 0 is an easy case, it is not so much more likely to be the argument
    // than other values, so no need for a special test.
    let y = x / (2.0 + x);
    let y_squared = y * y;
    // Original Formula is this: 2y·(15 - 4y²)/(15 - 9y²)
    // 2.0 * y * (15.0 - 4.0 * y_squared) / (15.0 - 9.0 * y_squared)

    // Reduce multiplications: (8/9)y·(3.75 - y²)/((5/3) - y²)
    0.8888888888888889 * y * (3.75 - y_squared) / (1.6666666666666667 - y_squared)
}


#[cfg(test)]
/// Tests of the Clustering methods.
mod tests {
    #[allow(unused_imports)]
    use spectral::prelude::*;
    use crate::clustering::logarithm::log_ratio;

    /// Test the log of a rational approximation of e, which should be close to one.
    /// 
    ///  e - 2 = 0.71828182846... and is approximated by 12,993 / 18,089 = 0.71828182874.... 
    /// 
    /// This approximation is good to nine decimal places, which is higher than the five decimal places
    /// promised by the method under test.
    #[test]
    fn log_e() {
        let numerator : u64 = 2 * 18_089 + 12_993;
        let denominator = 18_089;
        let actual_log_e = log_ratio(numerator, denominator);
        let abs_error = (1.0 - actual_log_e).abs();
        asserting(&format!("Log of e should be one, but was {}", actual_log_e)).that(&(abs_error < 0.0001)).is_equal_to(true);
    }

    #[test]
    fn log_various() {
        for numerator in 1..1000_u64 {
            for denominator in 1..1000_u64 {
                let actual_log = log_ratio(numerator, denominator);
                let rational = numerator as f64 / denominator as f64;
                let expected_log = rational.ln();
                let abs_error = (actual_log - expected_log).abs();
                asserting(&format!("Log({}) should be {}, but was {}", rational, expected_log, actual_log)).that(&(abs_error < 0.0001)).is_equal_to(true);
            }
        }
    }

}
