#![no_std]
#![no_main]

use arduino_hal::simple_pwm::*;
use panic_halt as _;

#[arduino_hal::entry]
fn main() -> ! {
    let dp = arduino_hal::Peripherals::take().unwrap();
    let pins = arduino_hal::pins!(dp);

    let mut serial = arduino_hal::default_serial!(dp, pins, 57600);

    let timer2 = Timer2Pwm::new(dp.TC2, Prescaler::Prescale64);
    let mut output = pins.d11.into_output().into_pwm(&timer2);
    output.enable();
    output.set_duty(((0.6 / 5.0) * 255.0) as u8);


    let switch = pins.d10.into_pull_up_input();
    let mut power = true;

    let input1 = pins.d2.into_pull_up_input();
    let input2 = pins.d3.into_pull_up_input();
    let input3 = pins.d4.into_pull_up_input();

    let duty_1 = ((0.6 / 5.0) * 255.0) as u8;
    let duty_2 = ((2.3 / 5.0) * 255.0) as u8;
    let duty_3 = ((5.0 / 5.0) * 255.0) as u8;
    
    let mut already_pressed = false;
    loop {
        if switch.is_low() {
            arduino_hal::delay_ms(60);
            if already_pressed {continue;}
            already_pressed = true;

            if !power {
                ufmt::uwriteln!(&mut serial, "Enable\r").unwrap();
                output.enable();
            } else {
                ufmt::uwriteln!(&mut serial, "Disable\r").unwrap();
                output.disable();
            }
            power = !power;
        } else {
            already_pressed = false;
        }
        
        if !power { continue; }

        if input1.is_low() {
            output.set_duty(duty_1);
        }
        if input2.is_low() {
            output.set_duty(duty_2);
        }
        if input3.is_low() {
            output.set_duty(duty_3);
        }

        arduino_hal::delay_ms(60);
    }
}
