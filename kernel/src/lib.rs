#![no_std]

pub mod display;
pub mod psf;

use lazy_static::lazy_static;
use spin::Mutex;

lazy_static! {
    pub static ref SERIAL: Mutex<uart_16550::SerialPort> = unsafe { uart_16550::SerialPort::new(0x3F8).into() };
}