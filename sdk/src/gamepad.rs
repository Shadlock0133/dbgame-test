use bitflags::bitflags;

use crate::db_internal::{
    gamepad_isConnected, gamepad_readState, gamepad_setRumble,
};

#[repr(C)]
#[derive(Clone, Copy)]
pub enum GamepadSlot {
    SlotA,
    SlotB,
    SlotC,
    SlotD,
}

bitflags! {
    #[repr(C)]
    #[derive(Clone, Copy, Debug)]
    pub struct GamepadButton: u16 {
        const A       = 1;
        const B       = (1 << 1);
        const X       = (1 << 2);
        const Y       = (1 << 3);
        const Up      = (1 << 4);
        const Down    = (1 << 5);
        const Left    = (1 << 6);
        const Right   = (1 << 7);
        const L1      = (1 << 8);
        const L2      = (1 << 9);
        const L3      = (1 << 10);
        const R1      = (1 << 11);
        const R2      = (1 << 12);
        const R3      = (1 << 13);
        const Select  = (1 << 14);
        const Start   = (1 << 15);
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct GamepadState {
    pub button_mask: GamepadButton,
    pub left_stick_x: i16,
    pub left_stick_y: i16,
    pub right_stick_x: i16,
    pub right_stick_y: i16,
}

impl GamepadState {
    /// Check if the given button is pressed
    pub fn is_pressed(self, button: GamepadButton) -> bool {
        self.button_mask.contains(button)
    }
}

pub struct Gamepad {
    pub slot: GamepadSlot,
}

impl Gamepad {
    /// Construct a new Gamepad for the given slot
    pub const fn new(slot: GamepadSlot) -> Gamepad {
        Gamepad { slot }
    }

    /// Check whether this gamepad is connected
    pub fn is_connected(&self) -> bool {
        unsafe { gamepad_isConnected(self.slot) }
    }

    /// Read the state of this gamepad
    pub fn read_state(&self) -> GamepadState {
        let mut state = GamepadState {
            button_mask: GamepadButton::empty(),
            left_stick_x: 0,
            left_stick_y: 0,
            right_stick_x: 0,
            right_stick_y: 0,
        };
        unsafe {
            gamepad_readState(self.slot, &mut state);
        }
        state
    }

    /// Set this gamepad's vibration on or off
    pub fn set_rumble(&self, enable: bool) {
        unsafe {
            gamepad_setRumble(self.slot, enable);
        }
    }
}
