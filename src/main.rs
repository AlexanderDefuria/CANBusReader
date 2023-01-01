#![no_std]
#![no_main]

// The macro for our start-up function
use rp_pico::entry;
use panic_halt as _;
use rp_pico::hal::prelude::*;
use rp_pico::hal::pac;
use rp_pico::hal::timer::Timer;
use rp_pico::hal;
// PIOExt for the split() method that is needed to bring
// PIO0 into useable form for Ws2812:
use rp_pico::hal::pio::PIOExt;
use smart_leds::{SmartLedsWrite, RGB8};
use usb_device::class_prelude::UsbBusAllocator;
use usb_device::prelude::{UsbDeviceBuilder, UsbVidPid};
use usbd_serial::SerialPort;
use ws2812_pio::Ws2812;
use core::fmt::Write;
use cortex_m::prelude::_embedded_hal_adc_OneShot;
use heapless::String;

// TODO CANBUS
// https://github.com/davidcole1340/mcp2515-rs

// Currently 5 consecutive LEDs are driven by this example
// to keep the power draw compatible with USB:
const STRIP_LEN: usize = 5;

#[entry]
fn main() -> ! {
    // Grab our singleton objects
    let mut pac = pac::Peripherals::take().unwrap();
    let core = pac::CorePeripherals::take().unwrap();

    // Set up the watchdog driver - needed by the clock setup code
    let mut watchdog = hal::Watchdog::new(pac.WATCHDOG);

    // Configure the clocks
    // The default is to generate a 125 MHz system clock
    let clocks = hal::clocks::init_clocks_and_plls(
        rp_pico::XOSC_CRYSTAL_FREQ,
        pac.XOSC,
        pac.CLOCKS,
        pac.PLL_SYS,
        pac.PLL_USB,
        &mut pac.RESETS,
        &mut watchdog,
    ).ok().unwrap();

    // Set up the USB driver
    let usb_bus = UsbBusAllocator::new(hal::usb::UsbBus::new(
        pac.USBCTRL_REGS,
        pac.USBCTRL_DPRAM,
        clocks.usb_clock,
        true,
        &mut pac.RESETS,
    ));

    // Set up the USB Communications Class Device driver
    let mut serial = SerialPort::new(&usb_bus);

    // Create a USB device with a fake VID and PID
    let mut usb_dev = UsbDeviceBuilder::new(&usb_bus, UsbVidPid(0x16c0, 0x27dd))
        .manufacturer("Fake company")
        .product("Serial port")
        .serial_number("TEST")
        .device_class(2) // from: https://www.usb.org/defined-class-codes
        .build();



    let mut delay = cortex_m::delay::Delay::new(core.SYST, clocks.system_clock.freq().to_Hz());

    // The single-cycle I/O block controls our GPIO pins
    let sio = hal::Sio::new(pac.SIO);

    // Set the pins up according to their function on this particular board
    let pins = rp_pico::Pins::new(
        pac.IO_BANK0,
        pac.PADS_BANK0,
        sio.gpio_bank0,
        &mut pac.RESETS,
    );


    // Enable ADC
    let mut adc = hal::Adc::new(pac.ADC, &mut pac.RESETS);

    // Enable the temperature sense channel
    let mut temperature_sensor = adc.enable_temp_sensor();

    // Configure GPIO26 as an ADC input
    let mut adc_pin_0 = pins.gpio28.into_floating_input();

    // Create a count down timer for the Ws2812 instance:
    let timer = Timer::new(pac.TIMER, &mut pac.RESETS);

    // Split the PIO state machine 0 into individual objects, so that
    // Ws2812 can use it:
    let (mut pio, sm0, _, _, _) = pac.PIO0.split(&mut pac.RESETS);

    // Instanciate a Ws2812 LED strip:
    let mut ws = Ws2812::new(
        pins.gpio21.into_mode(),
        &mut pio,
        sm0,
        clocks.peripheral_clock.freq(),
        timer.count_down(),
    );

    let leds: [RGB8; STRIP_LEN] = [(10, 0, 10).into(); STRIP_LEN];
    loop {
        // Read the raw ADC counts from the temperature sensor channel.
        let pin_adc_counts: u16 = adc.read(&mut adc_pin_0).unwrap();

        let mut text: String<64> = String::new();

        writeln!(
            text,
            "Time,{:02}",
            pin_adc_counts
        ).unwrap();

        // This only works reliably because the number of bytes written to
        // the serial port is smaller than the buffers available to the USB
        // peripheral. In general, the return value should be handled, so that
        // bytes not transferred yet don't get lost.
        let _ = serial.write(text.as_bytes());


        // Check for new data
        // This has to be called a minimum of every 10 milliseconds to be usb compliant
        let _ = usb_dev.poll(&mut [&mut serial]);

        ws.write(leds.iter().copied()).unwrap();

    }

}
