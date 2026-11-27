#![no_std]
#![no_main]

use defmt::*;
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_stm32::sai::{
    BitOrder, ClockStrobe, DataSize, FrameSyncOffset, FrameSyncPolarity, MasterClockDivider, Sai, word,
};
use embassy_stm32::time::Hertz;
use embassy_stm32::{Config, bind_interrupts, dma, peripherals, sai};
use embassy_time::{Duration, Ticker};
use embassy_stm32::dac::Dac;
use panic_probe as _;

bind_interrupts!(struct Irqs {
    DMA1_STREAM0   => dma::InterruptHandler<peripherals::DMA1_CH0>;
});
/// Convert a captured u32 (24-bit two's complement, right-aligned,
/// zero-extended into bits [31:24]) into a true signed sample.
#[inline]
fn to_sample(w: u32) -> i32 {
    ((w << 8) as i32) >> 8
}

#[embassy_executor::main]
async  fn main(_spawner: Spawner){
    info!("SAI test");

    

    let mut config = Config::default();{
        use embassy_stm32::rcc::*;
        // HSE = 24.576
        config.rcc.hse = Some(Hse { freq: Hertz(25_000_000), mode: HseMode::Oscillator });

        // Because we use a 24.576 MHz crystal
        config.rcc.pll2 = Some(Pll { 
            source: PllSource::Hse,
            prediv: PllPreDiv::Div25,
            mul: PllMul::Mul245,
            fracn: Some(6226),
            divp: Some(PllDiv::Div10),
            divq: None,
            divr: None
                 });

        config.rcc.mux.sai1sel = mux::Saisel::Pll2P;
    }

    info!("Config done");
    
    let p = embassy_stm32::init(config);
    info!("Peripheral done");

    // DMA buffer
    let mut sai_1_a_write_pcm_buffer: [u32; 256] = [0u32; 256]; // For the SAI to write what it receive
    let mut sai_1_a_read_buffer: [u32; 256] = [0u32; 256]; // To read what the SAI wrote in its buffer

    let (sai_a, _) = sai::split_subblocks(p.SAI1);

    info!("Sai pulled");
    // SAI configuration for I2S format
    let mut sai_config = sai::Config::default();
    sai_config.mode = sai::Mode::Master;
    sai_config.tx_rx = sai::TxRx::Receiver;
    sai_config.bit_order = BitOrder::MsbFirst; // I²S is MsbFirst 
    sai_config.slot_count = word::U4(2); // Stereo channels
    sai_config.data_size = DataSize::Data24; //PCM1808 send 24 bits data. If it doesn't work consider going back to 32 bit like in the example
    sai_config.frame_length = 64; // 2 channels * 32 bits (apparently, lumo says that SAI decouple the 24/32 bit problem) let see
    sai_config.frame_sync_active_level_length = word::U7(32);
    sai_config.master_clock_divider = MasterClockDivider::Div1;
    sai_config.clock_strobe = ClockStrobe::Rising;
    sai_config.frame_sync_offset = FrameSyncOffset::BeforeFirstBit;
    sai_config.frame_sync_polarity = FrameSyncPolarity::ActiveLow;

    info!("SAI config prepared ");
    let mut sai_1_a = Sai::new_asynchronous_with_mclk(
        sai_a,
        p.PE5,
        p.PE6,
        p.PE4, 
        p.PE2, 
        p.DMA1_CH0, 
        Irqs, 
        &mut sai_1_a_write_pcm_buffer,
        sai_config);

    info!("SAI config loaded");

    sai_1_a.start();
    
    let mut n_buffers: u32 = 0;

    info!("SAI started");

    
    // let mut dac = Dac::new_1(p.DAC1, p.PA4);
    loop{

       sai_1_a.read(&mut sai_1_a_read_buffer).await.ok();

        let mut min = i32::MAX;
        let mut max = i32::MIN;
        let mut sum: i64 = 0;

        for &w in &sai_1_a_read_buffer {
            let s = to_sample(w);
            if s < min { min = s; }
            if s > max { max = s; }
            sum += s as i64;
        }

        n_buffers += 1;
        if n_buffers >= 188 {
            let mean = (sum / sai_1_a_read_buffer.len() as i64) as i32;
            info!("span: min={} max={} dc={}", min, max, mean);
            n_buffers = 0;
        }
    }
}
