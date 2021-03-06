use libc::{c_int, c_ulong};
use std::{fmt, ptr};
use std::c_str::CString;
use std::num::{One, Zero};

use ffi;
use ssl::error::SslError;

pub struct BigNum(*mut ffi::BIGNUM);

#[repr(C)]
pub enum RNGProperty {
    MsbMaybeZero = -1,
    MsbOne = 0,
    TwoMsbOne = 1,
}

macro_rules! with_ctx(
    ($name:ident, $action:block) => ({
        let $name = ffi::BN_CTX_new();
        if ($name).is_null() {
            Err(SslError::get())
        } else {
            let r = $action;
            ffi::BN_CTX_free($name);
            r
        }
    });
)

macro_rules! with_bn(
    ($name:ident, $action:block) => ({
        let tmp = BigNum::new();
        match tmp {
            Ok($name) => {
                if $action {
                    Ok($name)
                } else {
                    Err(SslError::get())
                }
            },
            Err(err) => Err(err),
        }
    });
)

macro_rules! with_bn_in_ctx(
    ($name:ident, $ctx_name:ident, $action:block) => ({
        let tmp = BigNum::new();
        match tmp {
            Ok($name) => {
                let $ctx_name = ffi::BN_CTX_new();
                if ($ctx_name).is_null() {
                    Err(SslError::get())
                } else {
                    let r =
                        if $action {
                            Ok($name)
                        } else {
                            Err(SslError::get())
                        };
                    ffi::BN_CTX_free($ctx_name);
                    r
                }
            },
            Err(err) => Err(err),
        }
    });
)

impl BigNum {
    pub fn new() -> Result<BigNum, SslError> {
        unsafe {
            let v = ffi::BN_new();
            if v.is_null() {
                Err(SslError::get())
            } else {
                Ok(BigNum(v))
            }
        }
    }

    pub fn new_from(n: u64) -> Result<BigNum, SslError> {
        unsafe {
            let bn = ffi::BN_new();
            if bn.is_null() || ffi::BN_set_word(bn, n as c_ulong) == 0 {
                Err(SslError::get())
            } else {
                Ok(BigNum(bn))
            }
        }
    }

    pub fn new_from_slice(n: &[u8]) -> Result<BigNum, SslError> {
        unsafe {
            let bn = ffi::BN_new();
            if bn.is_null() || ffi::BN_bin2bn(n.as_ptr(), n.len() as c_int, bn).is_null() {
                Err(SslError::get())
            } else {
                Ok(BigNum(bn))
            }
        }
    }

    pub fn checked_sqr(&self) -> Result<BigNum, SslError> {
        unsafe {
            with_bn_in_ctx!(r, ctx, { ffi::BN_sqr(r.raw(), self.raw(), ctx) == 1 })
        }
    }

    pub fn checked_nnmod(&self, n: &BigNum) -> Result<BigNum, SslError> {
        unsafe {
            with_bn_in_ctx!(r, ctx, { ffi::BN_nnmod(r.raw(), self.raw(), n.raw(), ctx) == 1 })
        }
    }

    pub fn checked_mod_add(&self, a: &BigNum, n: &BigNum) -> Result<BigNum, SslError> {
        unsafe {
            with_bn_in_ctx!(r, ctx, { ffi::BN_mod_add(r.raw(), self.raw(), a.raw(), n.raw(), ctx) == 1 })
        }
    }

    pub fn checked_mod_sub(&self, a: &BigNum, n: &BigNum) -> Result<BigNum, SslError> {
        unsafe {
            with_bn_in_ctx!(r, ctx, { ffi::BN_mod_sub(r.raw(), self.raw(), a.raw(), n.raw(), ctx) == 1 })
        }
    }

    pub fn checked_mod_mul(&self, a: &BigNum, n: &BigNum) -> Result<BigNum, SslError> {
        unsafe {
            with_bn_in_ctx!(r, ctx, { ffi::BN_mod_mul(r.raw(), self.raw(), a.raw(), n.raw(), ctx) == 1 })
        }
    }

    pub fn checked_mod_sqr(&self, n: &BigNum) -> Result<BigNum, SslError> {
        unsafe {
            with_bn_in_ctx!(r, ctx, { ffi::BN_mod_sqr(r.raw(), self.raw(), n.raw(), ctx) == 1 })
        }
    }

    pub fn checked_exp(&self, p: &BigNum) -> Result<BigNum, SslError> {
        unsafe {
            with_bn_in_ctx!(r, ctx, { ffi::BN_exp(r.raw(), self.raw(), p.raw(), ctx) == 1 })
        }
    }

    pub fn checked_mod_exp(&self, p: &BigNum, n: &BigNum) -> Result<BigNum, SslError> {
        unsafe {
            with_bn_in_ctx!(r, ctx, { ffi::BN_mod_exp(r.raw(), self.raw(), p.raw(), n.raw(), ctx) == 1 })
        }
    }

    pub fn checked_mod_inv(&self, n: &BigNum) -> Result<BigNum, SslError> {
        unsafe {
            with_bn_in_ctx!(r, ctx, { !ffi::BN_mod_inverse(r.raw(), self.raw(), n.raw(), ctx).is_null() })
        }
    }

    pub fn checked_gcd(&self, a: &BigNum) -> Result<BigNum, SslError> {
        unsafe {
            with_bn_in_ctx!(r, ctx, { ffi::BN_gcd(r.raw(), self.raw(), a.raw(), ctx) == 1 })
        }
    }

    pub fn checked_generate_prime(bits: i32, safe: bool, add: Option<&BigNum>, rem: Option<&BigNum>) -> Result<BigNum, SslError> {
        unsafe {
            with_bn_in_ctx!(r, ctx, {
                let add_arg = add.map(|a| a.raw()).unwrap_or(ptr::null_mut());
                let rem_arg = rem.map(|r| r.raw()).unwrap_or(ptr::null_mut());

                ffi::BN_generate_prime_ex(r.raw(), bits as c_int, safe as c_int, add_arg, rem_arg, ptr::null()) == 1
            })
        }
    }

    pub fn is_prime(&self, checks: i32) -> Result<bool, SslError> {
        unsafe {
            with_ctx!(ctx, {
                Ok(ffi::BN_is_prime_ex(self.raw(), checks as c_int, ctx, ptr::null()) == 1)
            })
        }
    }

    pub fn is_prime_fast(&self, checks: i32, do_trial_division: bool) -> Result<bool, SslError> {
        unsafe {
            with_ctx!(ctx, {
                Ok(ffi::BN_is_prime_fasttest_ex(self.raw(), checks as c_int, ctx, do_trial_division as c_int, ptr::null()) == 1)
            })
        }
    }

    pub fn checked_new_random(bits: i32, prop: RNGProperty, odd: bool) -> Result<BigNum, SslError> {
        unsafe {
            with_bn_in_ctx!(r, ctx, { ffi::BN_rand(r.raw(), bits as c_int, prop as c_int, odd as c_int) == 1 })
        }
    }

    pub fn checked_new_pseudo_random(bits: i32, prop: RNGProperty, odd: bool) -> Result<BigNum, SslError> {
        unsafe {
            with_bn_in_ctx!(r, ctx, { ffi::BN_pseudo_rand(r.raw(), bits as c_int, prop as c_int, odd as c_int) == 1 })
        }
    }

    pub fn checked_rand_in_range(&self) -> Result<BigNum, SslError> {
        unsafe {
            with_bn_in_ctx!(r, ctx, { ffi::BN_rand_range(r.raw(), self.raw()) == 1 })
        }
    }

    pub fn checked_pseudo_rand_in_range(&self) -> Result<BigNum, SslError> {
        unsafe {
            with_bn_in_ctx!(r, ctx, { ffi::BN_pseudo_rand_range(r.raw(), self.raw()) == 1 })
        }
    }

    pub fn set_bit(&mut self, n: i32) -> Result<(), SslError> {
        unsafe {
            if ffi::BN_set_bit(self.raw(), n as c_int) == 1 {
                Ok(())
            } else {
                Err(SslError::get())
            }
        }
    }

    pub fn clear_bit(&mut self, n: i32) -> Result<(), SslError> {
        unsafe {
            if ffi::BN_clear_bit(self.raw(), n as c_int) == 1 {
                Ok(())
            } else {
                Err(SslError::get())
            }
        }
    }

    pub fn is_bit_set(&self, n: i32) -> bool {
        unsafe {
            ffi::BN_is_bit_set(self.raw(), n as c_int) == 1
        }
    }

    pub fn mask_bits(&mut self, n: i32) -> Result<(), SslError> {
        unsafe {
            if ffi::BN_mask_bits(self.raw(), n as c_int) == 1 {
                Ok(())
            } else {
                Err(SslError::get())
            }
        }
    }

    pub fn checked_shl1(&self) -> Result<BigNum, SslError> {
        unsafe {
            with_bn!(r, { ffi::BN_lshift1(r.raw(), self.raw()) == 1 })
        }
    }

    pub fn checked_shr1(&self) -> Result<BigNum, SslError> {
        unsafe {
            with_bn!(r, { ffi::BN_rshift1(r.raw(), self.raw()) == 1 })
        }
    }

    pub fn checked_add(&self, a: &BigNum) -> Result<BigNum, SslError> {
        unsafe {
            with_bn!(r, { ffi::BN_add(r.raw(), self.raw(), a.raw()) == 1 })
        }
    }

    pub fn checked_sub(&self, a: &BigNum) -> Result<BigNum, SslError> {
        unsafe {
            with_bn!(r, { ffi::BN_sub(r.raw(), self.raw(), a.raw()) == 1 })
        }
    }

    pub fn checked_mul(&self, a: &BigNum) -> Result<BigNum, SslError> {
        unsafe {
            with_bn_in_ctx!(r, ctx, { ffi::BN_mul(r.raw(), self.raw(), a.raw(), ctx) == 1 })
        }
    }

    pub fn checked_div(&self, a: &BigNum) -> Result<BigNum, SslError> {
        unsafe {
            with_bn_in_ctx!(r, ctx, { ffi::BN_div(r.raw(), ptr::null_mut(), self.raw(), a.raw(), ctx) == 1 })
        }
    }

    pub fn checked_mod(&self, a: &BigNum) -> Result<BigNum, SslError> {
        unsafe {
            with_bn_in_ctx!(r, ctx, { ffi::BN_div(ptr::null_mut(), r.raw(), self.raw(), a.raw(), ctx) == 1 })
        }
    }

    pub fn checked_shl(&self, a: &i32) -> Result<BigNum, SslError> {
        unsafe {
            with_bn!(r, { ffi::BN_lshift(r.raw(), self.raw(), *a as c_int) == 1 })
        }
    }

    pub fn checked_shr(&self, a: &i32) -> Result<BigNum, SslError> {
        unsafe {
            with_bn!(r, { ffi::BN_rshift(r.raw(), self.raw(), *a as c_int) == 1 })
        }
    }

    pub fn negate(&mut self) {
        unsafe {
            ffi::BN_set_negative(self.raw(), !self.is_negative() as c_int)
        }
    }

    pub fn abs_cmp(&self, oth: BigNum) -> Ordering {
        unsafe {
            let res = ffi::BN_ucmp(self.raw(), oth.raw()) as i32;
            if res < 0 {
                Less
            } else if res > 0 {
                Greater
            } else {
                Equal
            }
        }
    }

    pub fn is_negative(&self) -> bool {
        unsafe {
            (*self.raw()).neg == 1
        }
    }

    pub fn num_bits(&self) -> i32 {
        unsafe {
            ffi::BN_num_bits(self.raw()) as i32
        }
    }

    pub fn num_bytes(&self) -> i32 {
        (self.num_bits() + 7) / 8
    }

    unsafe fn raw(&self) -> *mut ffi::BIGNUM {
        let BigNum(n) = *self;
        n
    }

    pub fn to_vec(&self) -> Vec<u8> {
        let size = self.num_bytes() as uint;
        let mut v = Vec::with_capacity(size);
        unsafe {
            ffi::BN_bn2bin(self.raw(), v.as_mut_ptr());
            v.set_len(size);
        }
        v
    }

    pub fn to_dec_str(&self) -> String {
        unsafe {
            let buf = ffi::BN_bn2dec(self.raw());
            assert!(!buf.is_null());
            let c_str = CString::new(buf, false);
            let str = c_str.as_str().unwrap().to_string();
            ffi::CRYPTO_free(buf);
            str
        }
    }
}

impl fmt::Show for BigNum {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.to_dec_str())
    }
}

impl One for BigNum {
    fn one() -> BigNum {
        BigNum::new_from(1).unwrap()
    }
}

impl Zero for BigNum {
    fn zero() -> BigNum {
        BigNum::new_from(0).unwrap()
    }
    fn is_zero(&self) -> bool {
        unsafe {
            ffi::BN_is_zero(self.raw()) == 1
        }
    }
}

impl Eq for BigNum { }
impl PartialEq for BigNum {
    fn eq(&self, oth: &BigNum) -> bool {
        unsafe {
            ffi::BN_cmp(self.raw(), oth.raw()) == 0
        }
    }
}

impl Ord for BigNum {
    fn cmp(&self, oth: &BigNum) -> Ordering {
        self.partial_cmp(oth).unwrap()
    }
}

impl PartialOrd for BigNum {
    fn partial_cmp(&self, oth: &BigNum) -> Option<Ordering> {
        unsafe {
            let v = ffi::BN_cmp(self.raw(), oth.raw());
            let ret =
                if v == 0 {
                    Equal
                } else if v < 0 {
                    Less
                } else {
                    Greater
                };
            Some(ret)
        }
    }
}

impl Drop for BigNum {
    fn drop(&mut self) {
        unsafe {
            if !self.raw().is_null() {
                ffi::BN_clear_free(self.raw());
            }
        }
    }
}

pub mod unchecked {
    use ffi;
    use super::{BigNum};

    impl Add<BigNum, BigNum> for BigNum {
        fn add(&self, oth: &BigNum) -> BigNum {
            self.checked_add(oth).unwrap()
        }
    }

    impl Sub<BigNum, BigNum> for BigNum {
        fn sub(&self, oth: &BigNum) -> BigNum {
            self.checked_sub(oth).unwrap()
        }
    }

    impl Mul<BigNum, BigNum> for BigNum {
        fn mul(&self, oth: &BigNum) -> BigNum {
            self.checked_mul(oth).unwrap()
        }
    }

    impl Div<BigNum, BigNum> for BigNum {
        fn div(&self, oth: &BigNum) -> BigNum {
            self.checked_div(oth).unwrap()
        }
    }

    impl Rem<BigNum, BigNum> for BigNum {
        fn rem(&self, oth: &BigNum) -> BigNum {
            self.checked_mod(oth).unwrap()
        }
    }

    impl Shl<i32, BigNum> for BigNum {
        fn shl(&self, n: &i32) -> BigNum {
            self.checked_shl(n).unwrap()
        }
    }

    impl Shr<i32, BigNum> for BigNum {
        fn shr(&self, n: &i32) -> BigNum {
            self.checked_shr(n).unwrap()
        }
    }

    impl Clone for BigNum {
        fn clone(&self) -> BigNum {
            unsafe {
                let r = ffi::BN_dup(self.raw());
                if r.is_null() {
                    fail!("Unexpected null pointer from BN_dup(..)")
                } else {
                    BigNum(r)
                }
            }
        }
    }

    impl Neg<BigNum> for BigNum {
        fn neg(&self) -> BigNum {
            let mut n = self.clone();
            n.negate();
            n
        }
    }
}

#[cfg(test)]
mod tests {
    use bn::BigNum;

    #[test]
    fn test_to_from_slice() {
        let v0 = BigNum::new_from(10203004_u64).unwrap();
        let vec = v0.to_vec();
        let v1 = BigNum::new_from_slice(vec.as_slice()).unwrap();

        assert!(v0 == v1);
    }

    #[test]
    fn test_negation() {
        let a = BigNum::new_from(909829283_u64).unwrap();

        assert!(!a.is_negative());
        assert!((-a).is_negative());
    }


    #[test]
    fn test_prime_numbers() {
        let a = BigNum::new_from(19029017_u64).unwrap();
        let p = BigNum::checked_generate_prime(128, true, None, Some(&a)).unwrap();

        assert!(p.is_prime(100).unwrap());
        assert!(p.is_prime_fast(100, true).unwrap());
    }
}
