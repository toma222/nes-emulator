
/// These is an abstraction for the process status register
/// The process status register is defined in the CPU class
/// and changes depending on the operation last preformed
/// 
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

    /// this overflow is set if the result of an arithmetic operation
    /// has yielded an invalid 2's complement
    Overflow =            0b00100000,
    
    /// this flag is set if the result of the last operation had bit 7 set to a one
    Negative =            0b01000000,
}

/// Wrapper for a u8 with bit flag functions that uses the ProcessorStatusFlags enum
pub struct ProcessorStatus (pub u8);

impl ProcessorStatus {

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
}
