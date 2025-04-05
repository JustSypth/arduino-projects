#![no_std]
#![no_main]

use panic_halt as _;

mod rng;

#[arduino_hal::entry]
fn setup() -> ! {
    let dp = arduino_hal::Peripherals::take().unwrap();
    let pins = arduino_hal::pins!(dp);

    // Analog input for rng seed
    let mut adc = arduino_hal::Adc::new(dp.ADC, Default::default());
    let a0 = pins.a0.into_analog_input(&mut adc);
    
    let mut rng = rng::Rng::new(a0.analog_read(&mut adc) as u32);
    let mut output = pins.d13.into_output();

    loop {
        output.toggle();

        
        let delay_ms = rng.random_range_u32(25, 250); 
        arduino_hal::delay_ms(delay_ms);
    }
}