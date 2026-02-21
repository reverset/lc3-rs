pub fn convert_str_to_i16_vec(str: &str) -> Vec<i16> {
    let mut res = Vec::with_capacity(str.len());
    for s in str.bytes() {
        res.push(s as i16);
    }

    res
}

pub fn i9_to_i16(x: i16) -> i16 {
    const MASK: i16 = 0b111111111;
    let val = x & MASK;

    if val & 0b100000000 != 0 {
        // is negative
        val | !MASK
    } else {
        val
    }
}

pub fn i6_to_i8(x: i8) -> i8 {
    const MASK: i8 = 0b111111;
    let val = x & MASK;

    if val & 0b100000 != 0 {
        // is negative
        val | !MASK
    } else {
        val
    }
}

pub fn i11_to_i16(x: i16) -> i16 {
    const MASK: i16 = 0b11111111111;
    let val = x & MASK;

    if val & 0b10000000000 != 0 {
        // is negative
        val | !MASK
    } else {
        val
    }
}

pub fn i5_to_i8(x: i8) -> i8 {
    const MASK: i8 = 0b11111;
    let val = x & MASK;

    if val & 0b10000 != 0 {
        // is negative
        val | !MASK
    } else {
        val
    }
}

pub fn check_i9_range(x: i16) {
    assert!((-256..=255).contains(&x));
}

pub fn check_i6_range(x: i8) {
    assert!((-32..=31).contains(&x))
}

pub fn check_i5_range(x: i8) {
    assert!((-8..=7).contains(&x))
}

pub fn check_i11_range(x: i16) {
    assert!((-1024..=1023).contains(&x))
}
