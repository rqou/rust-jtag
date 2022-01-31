use crate::*;

use ftdi_mpsse::{mpsse, ClockBits, MpsseCmd, MpsseCmdExecutor};

pub struct FTDIJTAG {
    jtag_state: JTAGAdapterState,
    chunkshift_state: ChunkShifterJTAGAdapterState,
    ftdi: ftdi::Device,
}
impl AsMut<JTAGAdapterState> for FTDIJTAG {
    fn as_mut(&mut self) -> &mut JTAGAdapterState {
        &mut self.jtag_state
    }
}
impl AsMut<ChunkShifterJTAGAdapterState> for FTDIJTAG {
    fn as_mut(&mut self) -> &mut ChunkShifterJTAGAdapterState {
        &mut self.chunkshift_state
    }
}
impl FTDIJTAG {
    pub fn new() -> Self {
        println!("new");

        let mut device = ftdi::find_by_vid_pid(0x0403, 0x8028).open().unwrap();

        let mpsse = ftdi_mpsse::MpsseSettings {
            reset: true,
            in_transfer_size: 64 * 1024,
            read_timeout: std::time::Duration::from_secs(1),
            write_timeout: std::time::Duration::from_secs(1),
            latency_timer: std::time::Duration::from_millis(10),
            mask: 0b1011,
            clock_frequency: Some(1_000_000),
        };

        device.init(&mpsse).unwrap();

        mpsse! {
            const (INIT_DATA, INIT_LEN) = {
                set_gpio_lower(0b0000, 0b1011);
            };
        }

        assert_eq!(INIT_LEN, 0);
        device.send(&INIT_DATA).unwrap();

        Self {
            jtag_state: JTAGAdapterState::new(),
            chunkshift_state: ChunkShifterJTAGAdapterState::new(),
            ftdi: device,
        }
    }
}

impl ChunkShifterJTAGAdapter for FTDIJTAG {
    fn delay_ns(&mut self, ns: u64) {
        std::thread::sleep(std::time::Duration::from_nanos(ns))
    }
    fn set_clk_speed(&mut self, clk_hz: u64) {
        println!("ignoring clock speed {clk_hz} hz");
    }

    fn shift_tms_chunk(&mut self, tms_chunk: &BitSlice) {
        println!("shift tms {tms_chunk:b}");

        let mut bytes = Vec::new();

        for subchunk in tms_chunk.chunks(7) {
            bytes.push(0b01001011u8); // tms out on -ve
            bytes.push((subchunk.len() - 1) as u8);
            let mut thisbyte = 0u8;
            thisbyte.view_bits_mut::<Lsb0>()[..subchunk.len()].clone_from_bitslice(subchunk);
            bytes.push(thisbyte);
        }

        println!("the resulting buffer is {bytes:x?}");

        self.ftdi.send(&bytes).unwrap();
    }
    fn shift_tdi_chunk(&mut self, tdi_chunk: &BitSlice, tms_exit: bool) {
        println!("shift tdi {tdi_chunk:b} tms? {tms_exit}");

        // super fixme
        self.shift_tditdo_chunk(tdi_chunk, tms_exit);
    }
    fn shift_tditdo_chunk(&mut self, tdi_chunk: &BitSlice, tms_exit: bool) -> BitVec {
        println!("shift tditdo {tdi_chunk:b} tms? {tms_exit}");

        assert!(tdi_chunk.len() > 1); // XXX

        let mut bytes = Vec::new();
        let mut rxbytes = 0;
        let mut bits_remaining = tdi_chunk.len() - 1; // need special TMS
        let mut inbitsi = 0;

        while bits_remaining > 0 {
            // fixme this is super inefficient
            let bits = if bits_remaining > 8 {
                8
            } else {
                bits_remaining
            };
            let mut thisbyte = 0u8;
            for i in 0..bits {
                if tdi_chunk[inbitsi + i] {
                    thisbyte |= 1 << i;
                }
            }

            bytes.push(ClockBits::LsbPosIn as u8); // tdi out on -ve, in on +ve
            bytes.push((bits - 1) as u8);
            bytes.push(thisbyte);

            rxbytes += 1;
            bits_remaining -= bits;
            inbitsi += bits;
        }

        // handle TMS
        bytes.push(0b01101011); // tms out on -ve, in on +ve
        bytes.push(0);
        if tms_exit {
            if tdi_chunk[tdi_chunk.len() - 1] {
                bytes.push(0b10000001);
            } else {
                bytes.push(0b00000001);
            }
        } else {
            if tdi_chunk[tdi_chunk.len() - 1] {
                bytes.push(0b10000000);
            } else {
                bytes.push(0b00000000);
            }
        }
        rxbytes += 1;

        bytes.push(MpsseCmd::SendImmediate as u8);

        println!("the resulting buffer is {bytes:x?} rx {rxbytes}");
        self.ftdi.send(&bytes).unwrap();

        let mut rxbytebuf = vec![0; rxbytes];
        self.ftdi.recv(&mut rxbytebuf).unwrap();
        println!("got back {rxbytebuf:x?}");

        // fixme fixme fixme
        let mut ret = Vec::new();
        let mut bits_remaining = tdi_chunk.len() - 1;
        let mut rxbytebuf_i = 0;
        while bits_remaining > 0 {
            let bits = if bits_remaining > 8 {
                8
            } else {
                bits_remaining
            };

            for i in (8 - bits)..8 {
                ret.push((rxbytebuf[rxbytebuf_i] & (1 << i)) != 0);
            }

            bits_remaining -= bits;
            rxbytebuf_i += 1;
        }
        assert_eq!(rxbytebuf_i, rxbytebuf.len() - 1);
        ret.push((rxbytebuf[rxbytebuf.len() - 1] & 0x80) != 0);
        assert_eq!(ret.len(), tdi_chunk.len());

        ret.iter().collect()
    }
}