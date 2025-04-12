#![no_std]
#![no_main]

#![feature(abi_avr_interrupt)]

use core::cell;
use arduino_hal::{hal::port::{PB0, PB1}, port::{mode::{Input, Output, PullUp}, Pin}};
use panic_halt as _;

/* Pin placement:
    Output = D13
    Trigger = D9
    Echo = D8
    Set = D2
    Cancel = D3
*/


///         LIBRARY         ///
//////////////////////////////////////////////////////////////////////////////////////////////////////
const PRESCALER: u32 = 1024;
const TIMER_COUNTS: u32 = 125;

const MILLIS_INCREMENT: u32 = PRESCALER * TIMER_COUNTS / 16000;

static MILLIS_COUNTER: avr_device::interrupt::Mutex<cell::Cell<u32>> =
    avr_device::interrupt::Mutex::new(cell::Cell::new(0));

fn millis_init(tc0: arduino_hal::pac::TC0) {
    // Configure the timer for the above interval (in CTC mode)
    // and enable its interrupt.
    tc0.tccr0a.write(|w| w.wgm0().ctc());
    tc0.ocr0a.write(|w| w.bits(TIMER_COUNTS as u8));
    tc0.tccr0b.write(|w| match PRESCALER {
        8 => w.cs0().prescale_8(),
        64 => w.cs0().prescale_64(),
        256 => w.cs0().prescale_256(),
        1024 => w.cs0().prescale_1024(),
        _ => panic!(),
    });
    tc0.timsk0.write(|w| w.ocie0a().set_bit());

    // Reset the global millisecond counter
    avr_device::interrupt::free(|cs| {
        MILLIS_COUNTER.borrow(cs).set(0);
    });
}

#[avr_device::interrupt(atmega328p)]
fn TIMER0_COMPA() {
    avr_device::interrupt::free(|cs| {
        let counter_cell = MILLIS_COUNTER.borrow(cs);
        let counter = counter_cell.get();
        counter_cell.set(counter + MILLIS_INCREMENT);
    })
}

fn millis() -> u32 {
    avr_device::interrupt::free(|cs| MILLIS_COUNTER.borrow(cs).get())
}
//////////////////////////////////////////////////////////////////////////////////////////////////////

fn abs(n: i32) -> i32 
{
    if n == i32::MIN {
        panic!("abs(i32::MIN) would overflow");
    }
    if n < 0 { -n } else { n }
}

fn get_distance(trigger: &mut Pin<Output, PB1>, echo: &Pin<Input<PullUp>, PB0>) -> u16 {
    let max_duration = 23200;

    trigger.set_high();
    arduino_hal::delay_us(10);
    trigger.set_low();

    while echo.is_low() {}

    let mut duration: u16 = 0;
    while echo.is_high() && duration < max_duration {
        duration += 1;
        arduino_hal::delay_us(1);
    }
    
    let distance = (duration) / 58;
    distance
}

#[arduino_hal::entry]
fn main() -> ! {
    let dp = arduino_hal::Peripherals::take().unwrap();
    let pins = arduino_hal::pins!(dp);

    let mut serial = arduino_hal::default_serial!(dp, pins, 57600);

    millis_init(dp.TC0);
    unsafe { avr_device::interrupt::enable() };

    let mut output = pins.d13.into_output();

    let mut trigger = pins.d9.into_output();
    let echo = pins.d8.into_pull_up_input();

    let set = pins.d2.into_pull_up_input();
    let cancel = pins.d3.into_pull_up_input();


    let mut set_distance = 0;

    let mut last_check_time: u32 = 0;
    let check_interval: u32 = 1000;

    let mut set_pressed = false;
    let mut cancel_pressed = false;

    let mut is_set = false;
    let mut can_set = true;

    let mut check_debounce = false;

    loop {
        let time = millis();

        // Set button
        #[allow(unused_assignments)]
        if set.is_low() && can_set {
            if !set_pressed {
                set_pressed = true;
                can_set = false;
                is_set = false;
    
                set_distance = get_distance(&mut trigger, &echo);
                ufmt::uwriteln!(&mut serial, "Setting distance: {}cm", set_distance).unwrap();
                is_set = true;
            }
        } else {
            set_pressed = false;
        }

        // Cancel button
        if cancel.is_low() {
            if !cancel_pressed {
                cancel_pressed = true;

                ufmt::uwriteln!(&mut serial, "Stopping the alarm..").unwrap();

                output.set_low();
                is_set = false;
                can_set = true;
            }
        } else {
            cancel_pressed = false;
        }

        // Check distance
        if is_set {
            if !check_debounce {
                check_debounce = true;

                arduino_hal::delay_ms(50);
                last_check_time = time;
                
                let current_distance = get_distance(&mut trigger, &echo);
                let difference = abs(set_distance as i32 - current_distance as i32);

                ufmt::uwriteln!(&mut serial, "Difference: {}cm", difference).unwrap();

                if difference > 2 {
                    ufmt::uwriteln!(&mut serial, "Alerting!!!").unwrap();
                    output.set_high();
                    is_set = false;
                }
            }
        }
        if time - last_check_time >= check_interval {
            check_debounce = false;
        }

        arduino_hal::delay_ms(50);
    }
}