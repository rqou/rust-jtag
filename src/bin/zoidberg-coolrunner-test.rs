use jtag::JTAGAdapter;

fn main() {
    println!("Hello world!");

    let mut adapter = jtag::drivers::FTDIJTAG::new();
    adapter.reset_to_tlr();
    adapter.go_rti();

    let idcode = adapter.shift_dr_inout(&[false; 32], false);

    let mut idcode_ = 0u32;
    for (i, bit) in idcode.into_iter().enumerate() {
        if bit {
            idcode_ |= 1 << i;
        }
    }

    println!("idcode {idcode_:08X}");

    let idcode2 = adapter.shift_dr_inout(&[false; 32], false);

    let mut idcode2_ = 0u32;
    for (i, bit) in idcode2.into_iter().enumerate() {
        if bit {
            idcode2_ |= 1 << i;
        }
    }

    println!("idcode2 {idcode2_:08X}");
}