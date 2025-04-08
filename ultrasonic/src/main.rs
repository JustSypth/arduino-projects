#![no_std]
#![no_main]

use panic_halt as _;

#[arduino_hal::entry]
fn main() -> ! {
    let dp = arduino_hal::Peripherals::take().unwrap();
    let pins = arduino_hal::pins!(dp);

    let mut serial = arduino_hal::default_serial!(dp, pins, 57600);

    let mut trigger = pins.d9.into_output();
    let echo = pins.d8.into_pull_up_input();

    // let mut output = pins.d13.into_output();

    let max_duration = 23200;

    loop {
        trigger.set_high();
        arduino_hal::delay_us(10);
        trigger.set_low();

        while echo.is_low() {}

        let mut duration: u16 = 0;
        while echo.is_high() && duration < max_duration {
            duration += 1;
            arduino_hal::delay_us(1);
        }
        
        let distance_cm = (duration) / 58;
        
        ufmt::uwriteln!(&mut serial, "Distance: {} cm\r", distance_cm as u32).unwrap();
        
        arduino_hal::delay_ms(1000);
    }
}