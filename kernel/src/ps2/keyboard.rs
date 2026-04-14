use core::panic;

use bitflags::bitflags;
use lazy_static::lazy_static;
use super::controller::PS2_CONTROLLER;
use super::keymap::PS2_SET2_KEYMAP_UK;
use spin::Mutex;
use crate::{print, println};

lazy_static! {
    pub static ref KEYBOARD: Mutex<PS2Keyboard> = Mutex::new(PS2Keyboard::new());
}

enum Command {
    GetSetScancodeSet = 0xF0,
    EnableScanning = 0xF4,
}

#[derive(Debug, PartialEq, Eq)]
enum State {
    Wait,
    Break,
}

pub struct PS2Keyboard {
    state: State
}

impl PS2Keyboard {
    pub fn new() -> Self {
        Self {
            state: State::Wait,
        }
    }

    pub fn init(&mut self) {
        unsafe {
            PS2_CONTROLLER.lock().enable_port1(true);
            PS2_CONTROLLER.lock().write_command_device(Command::EnableScanning as u8);
            PS2_CONTROLLER.lock().write_command_val_device(Command::GetSetScancodeSet as u8, 2);    
        }
    }

    pub fn irq_handler(&mut self) {
        unsafe {
            let key = PS2_CONTROLLER.lock().read_no_wait();
            
            if key == 0xF0 {
                self.state = State::Break;
                return;
            }
            
            if self.state == State::Break {
                self.state = State::Wait;
                return;
            }

            let key = PS2_SET2_KEYMAP_UK[key as usize];
            print!("{}", key);
        }
    }
}