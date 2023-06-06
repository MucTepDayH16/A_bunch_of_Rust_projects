pub fn carrying_mul(a: u64, b: u64, c: u64) -> (u64, u64) {
    #[cfg(target_arch = "aarch64")]
    unsafe {
        let lo;
        let hi;

        core::arch::asm!(
            "cmn xzr, xzr", // clear carry
            "mul {lo}, {a}, {b}",
            "add {lo}, {lo}, {c}",
            "umulh {hi}, {a}, {b}",
            "cset {hi}, cs",
            a = in(reg) a,
            b = in(reg) b,
            c = in(reg) c,
            lo = out(reg) lo,
            hi = out(reg) hi,
        );

        (lo, hi)
    }

    #[cfg(target_arch = "x86_64")]
    unsafe {
        let mut lo = b;
        let hi;

        core::arch::asm!(
            "mul {a}",
            "add rax, {c}",
            "adc rdx, 0",
            a = in(reg) a,
            c = in(reg) c,
            inout("rax") lo,
            out("rdx") hi,
        );

        (lo, hi)
    }
}

pub fn widening_mul(a: u64, b: u64) -> (u64, u64) {
    #[cfg(target_arch = "aarch64")]
    unsafe {
        let lo;
        let hi;

        core::arch::asm!(
            // "cmn xzr, xzr", // clear carry
            "mul {lo}, {a}, {b}",
            "umulh {hi}, {a}, {b}",
            a = in(reg) a,
            b = in(reg) b,
            lo = out(reg) lo,
            hi = out(reg) hi,
        );

        (lo, hi)
    }

    #[cfg(target_arch = "x86_64")]
    unsafe {
        let mut lo = b;
        let hi;

        core::arch::asm!(
            "mul {a}",
            a = in(reg) a,
            inout("rax") lo,
            out("rdx") hi,
        );

        (lo, hi)
    }
}

fn main() {
    println!("{:?}", carrying_mul(456, 123, 0));
    println!("{}", 456 * 123);
}
