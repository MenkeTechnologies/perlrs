//! NaN-boxed `u64` for [`crate::value::PerlValue`]: immediates in `0x7FF8…`, heap pointers in `0x7FF9…`.

/// Base for immediate tags (quiet NaN).
pub(crate) const QNAN_IMM: u64 = 0x7FF8_0000_0000_0000;
pub(crate) const SUB_UNDEF: u64 = 1;
pub(crate) const SUB_INT32: u64 = 2;

/// High bits for heap [`Arc`](crate::value::HeapObject) pointer (low 48 bits).
pub(crate) const HEAP_TAG_HI: u64 = 0x7FF9;

#[inline]
pub(crate) const fn encode_imm_undef() -> u64 {
    QNAN_IMM | (SUB_UNDEF << 32)
}

#[inline]
pub(crate) const fn encode_imm_int32(n: i32) -> u64 {
    QNAN_IMM | (SUB_INT32 << 32) | (n as u32 as u64)
}

#[inline]
pub(crate) fn is_imm(u: u64) -> bool {
    (u >> 48) == 0x7FF8
}

#[inline]
pub(crate) fn is_imm_undef(u: u64) -> bool {
    u == encode_imm_undef()
}

#[inline]
pub(crate) fn as_imm_int32(u: u64) -> Option<i32> {
    if !is_imm(u) {
        return None;
    }
    let sub = (u >> 32) & 0xFFFF;
    if sub != SUB_INT32 {
        return None;
    }
    Some((u & 0xFFFF_FFFF) as i32)
}

#[inline]
pub(crate) fn is_raw_float_bits(u: u64) -> bool {
    let exp = (u >> 52) & 0x7FF;
    exp != 0x7FF
}

#[inline]
pub(crate) fn float_needs_box(f: f64) -> bool {
    let exp = (f.to_bits() >> 52) & 0x7FF;
    exp == 0x7FF
}

#[inline]
pub(crate) fn encode_heap_ptr<T>(ptr: *const T) -> u64 {
    let p = ptr as usize as u64;
    debug_assert!(
        p >> 48 == 0,
        "heap pointer must fit in low 48 bits for NaN-box encoding"
    );
    (HEAP_TAG_HI << 48) | (p & 0x0000_FFFF_FFFF_FFFF)
}

#[inline]
pub(crate) fn is_heap(u: u64) -> bool {
    (u >> 48) == HEAP_TAG_HI
}

#[inline]
pub(crate) fn decode_heap_ptr<T>(u: u64) -> *const T {
    debug_assert!(is_heap(u));
    (u & 0x0000_FFFF_FFFF_FFFF) as *const T
}
