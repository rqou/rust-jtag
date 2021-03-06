use bitvec::prelude::*;
use jtag::JTAGAdapter;

fn main() {
    println!("Hello world!");

    let mut adapter = jtag::drivers::FTDIJTAG::new();
    adapter.reset_to_tlr();
    adapter.go_rti();

    let idcode = adapter.shift_dr_inout(bits![0; 32], false);

    let mut idcode_ = 0u32;
    for (i, bit) in idcode.into_iter().enumerate() {
        if bit {
            idcode_ |= 1 << i;
        }
    }

    println!("idcode {idcode_:08X}");

    let idcode2 = adapter.shift_dr_inout(bits![0; 32], false);

    let mut idcode2_ = 0u32;
    for (i, bit) in idcode2.into_iter().enumerate() {
        if bit {
            idcode2_ |= 1 << i;
        }
    }

    println!("idcode2 {idcode2_:08X}");

    adapter.go_shiftdr();
    let idcode3_a = adapter.shift_bits_inout(bits![0; 16], false);
    let idcode3_b = adapter.shift_bits_inout(bits![0; 16], true);
    adapter.go_rti();
    adapter.flush();

    let mut idcode3 = 0u32;
    for (i, bit) in idcode3_a.into_iter().enumerate() {
        if bit {
            idcode3 |= 1 << i;
        }
    }
    for (i, bit) in idcode3_b.into_iter().enumerate() {
        if bit {
            idcode3 |= 1 << (i + 16);
        }
    }

    println!("idcode3 {idcode3:08X}");

    adapter.go_shiftdr();
    adapter.shift_bits_out(bits![0; 15], false);
    let idcode4_b = adapter.shift_bits_inout(bits![0; 17], true);
    adapter.go_rti();
    adapter.flush();

    let mut idcode4 = 0u32;
    for (i, bit) in idcode4_b.into_iter().enumerate() {
        if bit {
            idcode4 |= 1 << (i + 15);
        }
    }

    println!("idcode4 (parial) {idcode4:08X}");
}
