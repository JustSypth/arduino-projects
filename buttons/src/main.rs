#![no_std]
#![no_main]

use arduino_hal::simple_pwm::*;
use panic_halt as _;

mod rng;

#[arduino_hal::entry]
fn main() -> ! {
    let dp = arduino_hal::Peripherals::take().unwrap();
    let pins = arduino_hal::pins!(dp);

    let mut serial = arduino_hal::default_serial!(dp, pins, 57600);

    let timer2 = Timer2Pwm::new(dp.TC2, Prescaler::Prescale64);
    let mut output = pins.d11.into_output().into_pwm(&timer2);
    output.enable();
    output.set_duty(((0.6 / 5.0) * 255.0) as u8);


    let power = pins.d10.into_pull_up_input();
    let mut powered = true;

    let mut adc = arduino_hal::Adc::new(dp.ADC, Default::default());
    let analog0 = pins.a0.into_analog_input(&mut adc);

    let mut rng = rng::Rng::new(analog0.analog_read(&mut adc) as u32);

    let input1 = pins.d2.into_pull_up_input();
    let input2 = pins.d3.into_pull_up_input();
    let input3 = pins.d4.into_pull_up_input();

    let duty_1 = ((0.6 / 5.0) * 255.0) as u8;
    let duty_2 = ((2.3 / 5.0) * 255.0) as u8;
    let duty_3 = ((5.0 / 5.0) * 255.0) as u8;
    
    let mut power_pressed = false;
    let mut random_pressed = false;
    let mut random_mode = false;

    loop {
        // Power handling
        if power.is_low() {
            arduino_hal::delay_ms(60);
            if power_pressed {continue;}
            power_pressed = true;

            if !powered {
                ufmt::uwriteln!(&mut serial, "Enable\r").unwrap();
                output.enable();
            } else {
                ufmt::uwriteln!(&mut serial, "Disable\r").unwrap();
                output.disable();
            }
            powered = !powered;
        } else {
            power_pressed = false;
        }
        
        // Restricts execution if power is set to off
        if !powered { continue; }

        // Set random mode on all button press
        if input1.is_low() && input2.is_low() && input3.is_low() {
            if random_pressed {continue;}
            random_pressed = true;

            random_mode = !random_mode;
        } else {
            random_pressed = false;
        }

        // Set voltage
        if input1.is_low() {
            output.set_duty(duty_1);
        }
        if input2.is_low() {
            output.set_duty(duty_2);
        }
        if input3.is_low() {
            output.set_duty(duty_3);
        }

        if random_mode {
            output.disable();
            arduino_hal::delay_ms(rng.random_range_u32(10, 180));
            output.enable();
            arduino_hal::delay_ms(rng.random_range_u32(10, 180));
            
            continue;
        }

        arduino_hal::delay_ms(60);
    }
}
