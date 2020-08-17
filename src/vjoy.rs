pub type BOOL = ::std::os::raw::c_int;
pub type BYTE = ::std::os::raw::c_uchar;
pub type CHAR = ::std::os::raw::c_char;
pub type DWORD = ::std::os::raw::c_ulong;
pub type FLOAT = f32;
pub type INT = ::std::os::raw::c_int;
pub type LONG = ::std::os::raw::c_long;
pub type ULONG = ::std::os::raw::c_ulong;
pub type USHORT = ::std::os::raw::c_ushort;
pub type SHORT = ::std::os::raw::c_short;
pub type UCHAR = ::std::os::raw::c_uchar;
pub type UINT = ::std::os::raw::c_uint;
pub type WORD = ::std::os::raw::c_ushort;
pub type PVOID = *mut ::std::os::raw::c_void;
pub type VOID = ::std::os::raw::c_void;

extern "C" {
    pub fn AcquireVJD(rID: UINT) -> BOOL;
    pub fn FfbStart(rID: UINT) -> BOOL;
    pub fn vJoyEnabled() -> BOOL;
    pub fn ResetAll() -> BOOL;
    pub fn SetAxis(Value: LONG, rID: UINT, Axis: UINT) -> BOOL;
    pub fn SetDiscPov(Value: INT, rID: UINT, nPov: UCHAR) -> BOOL;
    pub fn SetBtn(Value: BOOL, rID: UINT, nBtn: UCHAR) -> BOOL; // Write Value to a given button defined in the specified VDJ
}

pub struct Joystick {
    device: UINT,
}

pub enum Axis {
    X = 0x30,
    Y = 0x31,
    Z = 0x32,
    RX = 0x33,
    RY = 0x34,
    RZ = 0x35,
    SL0 = 0x36,
    SL1 = 0x37,
    WHL = 0x38,
    POV = 0x39,
}

#[derive(Copy, Clone)]
pub enum PovDirection {
    NORTH = 0,
    EAST = 1,
    SOUTH = 2,
    WEST = 3,
    NEUTRAL = -1
}

impl Joystick {
    pub fn new(device: UINT) -> Joystick {
        Joystick { device: device }
    }

    pub fn is_enabled(&self) -> bool {
        unsafe { vJoyEnabled() == 1 }
    }

    pub fn acquire(&mut self) -> Result<(), String> {
        if !self.is_enabled() {
            return Err("vJoy not enabled.".to_owned());
        }

        if unsafe { AcquireVJD(self.device) == 0 } {
            return Err(format!("Could not acquire device {}", self.device));
        }

        if unsafe { FfbStart(self.device) == 0 } {
            return Err(format!("Could not start FFB on device {}", self.device));
        }

        Ok(())
    }

    pub fn reset(&mut self) -> Result<(), String> {
        if unsafe { ResetAll() == 0 } {
            return Err("Could not reset device.".to_owned());
        }
        Ok(())
    }

    pub fn set_axis(&mut self, axis: Axis, value: i32) -> Result<(), String> {
        if unsafe { SetAxis(value, self.device, axis as u32) } == 0 {
            return Err(format!("Could not update device {}", self.device));
        }
        Ok(())
    }

    pub fn set_btn(&mut self, button: u8, state: bool) -> Result<(), String> {
        if unsafe { SetBtn(state as i32, self.device, button as UCHAR) } == 0 {
            return Err(format!("Could not update device {}", self.device));
        }
        Ok(())
    }

    pub fn set_pov(&mut self, pov: u8, value: PovDirection) -> Result<(), String> {
        if unsafe { SetDiscPov(value as i32, self.device, pov) } == 0 {
            return Err(format!("Could not update device {}", self.device));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::*;
    #[test]
    fn it_works() -> Result<(), String> {
        let mut joystick = Joystick::new(1);
        joystick.acquire()?;
        joystick.reset()?;
        joystick.set_axis(Axis::X, 1000)?;
        joystick.set_axis(Axis::Y, 3000)?;
        joystick.set_axis(Axis::Z, 5000)?;
        joystick.set_axis(Axis::RX, 9000)?;
        joystick.set_axis(Axis::RY, 11000)?;
        joystick.set_axis(Axis::RZ, 13000)?;
        joystick.set_axis(Axis::SL0, 17000)?;
        joystick.set_axis(Axis::SL1, 19000)?;
        Ok(())
    }
}
