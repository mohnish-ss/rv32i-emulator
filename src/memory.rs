use crate::EmulatorError;

#[derive(Clone)]
pub struct Memory {
    data: Vec<u8>,
}

impl Memory {
    pub fn new(size: usize) -> Self {
        Self {
            data: vec![0; size],
        }
    }

    pub fn load_program(&mut self, bytes: &[u8]) -> Result<(), EmulatorError> {
        self.check(0, bytes.len())?;
        self.data[..bytes.len()].copy_from_slice(bytes);
        Ok(())
    }

    fn check(&self, address: u32, width: usize) -> Result<(), EmulatorError> {
        if (address as usize)
            .checked_add(width)
            .filter(|&end| end <= self.data.len())
            .is_none()
        {
            Err(EmulatorError::MemoryOutOfBounds { address, width })
        } else {
            Ok(())
        }
    }

    fn require_alignment(address: u32, width: usize) -> Result<(), EmulatorError> {
        if address % (width as u32) != 0 {
            Err(EmulatorError::Unaligned { address, width })
        } else {
            Ok(())
        }
    }

    pub fn read8(&self, address: u32) -> Result<u8, EmulatorError> {
        self.check(address, 1)?;
        Ok(self.data[address as usize])
    }

    pub fn read16(&self, address: u32) -> Result<u16, EmulatorError> {
        Self::require_alignment(address, 2)?;
        self.check(address, 2)?;
        Ok(u16::from_le_bytes([
            self.data[address as usize],
            self.data[address as usize + 1],
        ]))
    }

    pub fn read32(&self, address: u32) -> Result<u32, EmulatorError> {
        Self::require_alignment(address, 4)?;
        self.check(address, 4)?;
        Ok(u32::from_le_bytes(
            self.data[address as usize..address as usize + 4]
                .try_into()
                .expect("memory bounds were checked"),
        ))
    }

    pub fn write8(&mut self, address: u32, value: u8) -> Result<(), EmulatorError> {
        self.check(address, 1)?;
        self.data[address as usize] = value;
        Ok(())
    }

    pub fn write16(&mut self, address: u32, value: u16) -> Result<(), EmulatorError> {
        Self::require_alignment(address, 2)?;
        self.check(address, 2)?;
        self.data[address as usize..address as usize + 2]
            .copy_from_slice(&value.to_le_bytes());
        Ok(())
    }

    pub fn write32(&mut self, address: u32, value: u32) -> Result<(), EmulatorError> {
        Self::require_alignment(address, 4)?;
        self.check(address, 4)?;
        self.data[address as usize..address as usize + 4]
            .copy_from_slice(&value.to_le_bytes());
        Ok(())
    }
}
