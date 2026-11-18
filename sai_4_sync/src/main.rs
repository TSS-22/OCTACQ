#![no_std]
#![no_main]

use defmt::info;
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_stm32::sai::{
    BitOrder, ClockStrobe, DataSize, FrameSyncOffset, FrameSyncPolarity, MasterClockDivider, Sai, word, SlotSize,
};
use embassy_stm32::time::Hertz;
use embassy_stm32::{Config, bind_interrupts, dma, peripherals, sai};
use panic_probe as _;

bind_interrupts!(struct Irqs {
    DMA1_STREAM0   => dma::InterruptHandler<peripherals::DMA1_CH0>;
    DMA1_STREAM1   => dma::InterruptHandler<peripherals::DMA1_CH1>;
    DMA2_STREAM0   => dma::InterruptHandler<peripherals::DMA2_CH0>;
    DMA2_STREAM1   => dma::InterruptHandler<peripherals::DMA2_CH1>;
});

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    // Based on https://github.com/embassy-rs/embassy/blob/main/examples/stm32h5/src/bin/sai.rs
    let mut config = embassy_stm32::Config::default();

    let p = embassy_stm32::init(config);

    // write buffer for the DMA
    let mut sai1_a_dma_buffer = [0u32; 256];
    let mut sai1_b_dma_buffer = [0u32; 256];
    let mut sai4_a_dma_buffer = [0u32; 256];
    let mut sai4_b_dma_buffer = [0u32; 256];
    // Get the SAI subblocks
    let (sai1_a, sai1_b) = sai::split_subblocks(p.SAI1);
    let (sai4_a, sai4_b) = sai::split_subblocks(p.SAI4);

    // SAI configuration for I²S format 
    let mut sai_config = sai::Config::default();
    sai_config.bit_order = BitOrder::MsbFirst; // I guess that' the bit order of I²s
    sai_config.slot_count = word::U4(2); // Stereo
    sai_config.slot_size = sai::SlotSize::DataSize;
    sai_config.data_size = DataSize::Data32; // 24 bits with the PCM1808, but this make things a lot easier. Maybe we will go back to that later to push the 24 bit and gain in performance (I suppose)
    sai_config.frame_length = 64; //2 channels * 24 bits. Same because we use 32 bits for facility, it is then 2 * 32 
    sai_config.frame_sync_active_level_length = word::U7(24); // I suppose 24 is the right number. Original doc is 32 but they work with 32 bit acq bit depth.
    sai_config.master_clock_divider = MasterClockDivider::DIV1; // Because they do that
    sai_config.clock_strobe = ClockStrobe::Rising; // Same. I suppose this might be verifiable in the signal of the data sheet of the PCM1808 and the chip.
    sai_config.frame_sync_offset = FrameSyncOffset::BeforeFirstBit; // Same
    sai_config.frame_sync_polarity = FrameSyncPolarity::ActiveHigh; // Same

    // SAI configuration master/slave
    let mut sai_config_master = sai_config.clone();
    let mut sai_config_slave = sai_config.clone();
    sai_config_master.mode = sai::Mode::Master;
    sai_config_slave.mode = sai::Mode::Slave;

    let mut sai1_a_master = Sai::new_asynchronous_with_mclk(
        sai1_a,
        p.PE5,
        p.PE6,
        p.PE4,
        p.PE2,
        p.DMA1_CH0,
        &mut sai1_a_dma_buffer,
        Irqs,
        sai_config_master
    );

    let mut sai1_b_slave = Sai::new_asynchronous_with_mclk(
        sai1_b,
        p.PE5,
        p.PE6,
        p.PE4,
        p.PE2,
        p.DMA1_CH1,
        &mut sai1_b_dma_buffer,
        Irqs,
        sai_config_slave
    );

    let mut sai4_a_slave = Sai::new_asynchronous_with_mclk(
        sai4_a,
        p.PE12,
        p.PE3,
        p.PE13,
        p.PE14,
        p.DMA2_CH0,
        &mut sai4_a_dma_buffer,
        Irqs,
        sai_config_slave
    );

    let mut sai4_b_slave = Sai::new_asynchronous_with_mclk(
        sai4_b,
        p.,
        p.,
        p.,
        p.,
        p.DMA2_CH1,
        &mut sai4_b_dma_buffer,
        Irqs,
        sai_config_slave
    );
    
}
