use core::fmt;


/// These is an abstraction for the process status register
/// The process status register is defined in the CPU class
/// and changes depending on the operation last preformed
#[derive(Debug)]
#[repr(u8)]
pub enum ProcessorStatusFlags
{
    /// just 0, used for initialization
    Default =             0b00000000,

    /// This flag is set if the last operation caused an overflow
    CarryFlag =           0b00000001,

    /// this is set if the result of the last operation was zero
    ZeroFlag =            0b00000010,

    /// this is called if the program called a SEI instruction
    /// the processor does not respond to interrupts will this flag is set
    InterruptDisable =    0b00000100,
    
    /// While this flag is set the processor will obey the rules
    /// of Binary coded decimal
    DecimalMode =         0b00001000,

    /// This is set when the BRK instruction is called
    BreakCommand =        0b00010000,

    /// This is another break bit
    BreakCommand2 =       0b00100000,

    /// this overflow is set if the result of an arithmetic operation
    /// has yielded an invalid 2's complement
    Overflow =            0b01000000,
    
    /// this flag is set if the result of the last operation had bit 7 set to a one
    Negative =            0b10000000,
}

/// Wrapper for a u8 with bit flag functions that uses the ProcessorStatusFlags enum
pub struct ProcessorStatus (pub u8);


impl fmt::Display for ProcessorStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        
        let ProcessorStatus(status) = self;
        for i in 0u8..8 {
            let flag = 1 << i;
            if status & flag != 0 {
                match flag {
                    0b00000000 => write!(f, "").unwrap_or_default(),
                    0b00000001 => write!(f, "CarryFlag, ").unwrap_or_default(),
                    0b00000010 => write!(f, "ZeroFlag").unwrap_or_default(),
                    0b00000100 => write!(f, "InterruptDisable, ").unwrap_or_default(),
                    0b00001000 => write!(f, "DecimalMode, ").unwrap_or_default(),
                    0b00010000 => write!(f, "BreakCommand, ").unwrap_or_default(),
                    0b00100000 => write!(f, "BreakCommand2, ").unwrap_or_default(),
                    0b01000000 => write!(f, "Overflow, ").unwrap_or_default(),
                    0b10000000 => write!(f, "Negative, ").unwrap_or_default(),
                    _ => panic!("Invalid flags"),
                }
            }
        }

        return Ok(());
    }
}

impl ProcessorStatus {
    pub fn update_zero_and_negative_flags(&mut self, value: u8) {
        if value == 0 {
            self.set_flag_true(ProcessorStatusFlags::ZeroFlag);
        } else {
            self.set_flag_false(ProcessorStatusFlags::ZeroFlag);
        }

        if value & 0b1000_0000 != 0 {
            self.set_flag_true(ProcessorStatusFlags::Negative);
        } else {
            self.set_flag_false(ProcessorStatusFlags::Negative);
        }
    }

    /// Flips whatever flag you give it
    pub fn set_flag_true(&mut self, flag: ProcessorStatusFlags) {
        let ProcessorStatus(status) = self;
        *status |= flag as u8;
    }

    pub fn set_flag_false(&mut self, flag: ProcessorStatusFlags) {
        let ProcessorStatus(status) = self;
        *status &= !(flag as u8);
    }

    pub fn toggle_flag(&mut self, flag: ProcessorStatusFlags) {
        let ProcessorStatus(status) = self;
        *status &= !(flag as u8);
    }

    pub fn has_flag_set(&self, flag: ProcessorStatusFlags) -> bool
    {
        let ProcessorStatus(status) = self;
        return status & (flag as u8) != 0;
    }

    /// Sets them all back to zero
    pub fn reset_flags(&mut self) {
        let ProcessorStatus(status) = self;
        *status = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn set_flag() {
        let mut status = ProcessorStatus(0);
        status.set_flag_true(ProcessorStatusFlags::CarryFlag);

        assert_eq!(status.has_flag_set(ProcessorStatusFlags::CarryFlag), true);

        status.set_flag_true(ProcessorStatusFlags::DecimalMode);
        assert_eq!(status.has_flag_set(ProcessorStatusFlags::DecimalMode), true);

        status.set_flag_false(ProcessorStatusFlags::DecimalMode);
        assert_eq!(status.has_flag_set(ProcessorStatusFlags::DecimalMode), false);
    }

    #[test]
    fn toggle_flag()
    {
        let mut status = ProcessorStatus(0);
        status.set_flag_true(ProcessorStatusFlags::BreakCommand);
        assert_eq!(status.has_flag_set(ProcessorStatusFlags::BreakCommand), true);

        status.toggle_flag(ProcessorStatusFlags::BreakCommand);
        assert_eq!(status.has_flag_set(ProcessorStatusFlags::BreakCommand), false);
    }

    #[test]
    fn zero_flags() {
        let mut status = ProcessorStatus(0);

        status.update_zero_and_negative_flags(2);
        assert_eq!(status.has_flag_set(ProcessorStatusFlags::ZeroFlag), false);

        status.update_zero_and_negative_flags(0);
        assert_eq!(status.has_flag_set(ProcessorStatusFlags::ZeroFlag), true);
    }

    #[test]
    fn sign_flags() {
        let mut status = ProcessorStatus(0);

        status.update_zero_and_negative_flags(0b1000_0010); // this is the binary representation of a u8 with a sign bit (holds -2)
        assert_eq!(status.has_flag_set(ProcessorStatusFlags::Negative), true);

        status.update_zero_and_negative_flags(30);
        assert_eq!(status.has_flag_set(ProcessorStatusFlags::Negative), false);
    }
}
