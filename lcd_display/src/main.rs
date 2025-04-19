#![no_std]
#![no_main]

use panic_halt as _;
use arduino_hal::port::{mode::{self}, Pin, PinOps};

const LCD_ADDRESS: u8 = 0x27;

/* Pin placement:
    SDA = A4
    SCL = A5
    Yes = D2
    No = D3
*/

#[arduino_hal::entry]
fn main() -> ! {
    let dp = arduino_hal::Peripherals::take().unwrap();
    let pins = arduino_hal::pins!(dp);

    let mut i2c = arduino_hal::I2c::new(
        dp.TWI, 
        pins.a4.into_pull_up_input(), 
        pins.a5.into_pull_up_input(), 
        50000
    );

    let mut delay = arduino_hal::Delay::new();

    let mut lcd = lcd_lcm1602_i2c::sync_lcd::Lcd::new(&mut i2c, &mut delay)
    .with_address(LCD_ADDRESS)
    .with_cursor_on(true)
    .with_rows(2)
    .init().unwrap();

    let y = pins.d2.into_pull_up_input();
    let n = pins.d3.into_pull_up_input();

    loop {
        lcd.clear().unwrap();
        lcd.write_str("Do you want a").unwrap();
        lcd.set_cursor(1, 0).unwrap();
        lcd.write_str("cookie? (Y/N)").unwrap();
    
        let response = get_response(&y, &n);
        match response {
            true => {
                lcd.clear().unwrap();
                lcd.write_str("Cookie is yes").unwrap();
            },
            false => {
                lcd.clear().unwrap();
                lcd.write_str("Cookie is no cuz").unwrap();
                lcd.set_cursor(1, 0).unwrap();
                lcd.write_str("u suck..").unwrap();
            },
        }

        arduino_hal::delay_ms(6000);
    }
}

fn is_pressed<M>(pin: &Pin<mode::Input<mode::PullUp>, M>, debounce: &mut bool) -> bool 
where
    M: PinOps
{
    let is_pressed = pin.is_low();

    if is_pressed {
        if !*debounce {
            *debounce = true;
            return true;
        } else {
            return false;
        }
    } else {
        *debounce = false;
        return false;
    }
}

fn get_response<M, N>(y: &Pin<mode::Input<mode::PullUp>, M>, n: &Pin<mode::Input<mode::PullUp>, N>) -> bool
where
    M: PinOps,
    N: PinOps
{
    let mut debounce_accept = false;
    let mut debounce_decline = false;
    loop {
        if is_pressed(y, &mut debounce_accept) {
            return true;
        }

        if is_pressed(n, &mut debounce_decline) {
            return false;
        }

        arduino_hal::delay_ms(50);
    }
}