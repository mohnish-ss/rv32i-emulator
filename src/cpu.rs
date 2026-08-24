use crate::{memory::Memory, EmulatorError};

pub struct Cpu {
    pub regs: [u32; 32],
    pub pc: u32,
    pub memory: Memory,
    pub trace: bool,
    pub halted: bool,
}

impl Cpu {
    pub fn new(memory_size: usize) -> Self {
        Self {
            regs: [0; 32],
            pc: 0,
            memory: Memory::new(memory_size),
            trace: false,
            halted: false,
        }
    }

    fn rd(instruction: u32) -> usize {
        ((instruction >> 7) & 31) as usize
    }

    fn rs1(instruction: u32) -> usize {
        ((instruction >> 15) & 31) as usize
    }

    fn rs2(instruction: u32) -> usize {
        ((instruction >> 20) & 31) as usize
    }

    fn sign_extend(value: u32, bits: u32) -> u32 {
        (((value << (32 - bits)) as i32) >> (32 - bits)) as u32
    }

    fn immediate_i(instruction: u32) -> u32 {
        Self::sign_extend(instruction >> 20, 12)
    }

    fn immediate_s(instruction: u32) -> u32 {
        Self::sign_extend(
            ((instruction >> 25) << 5) | ((instruction >> 7) & 31),
            12,
        )
    }

    fn immediate_b(instruction: u32) -> u32 {
        Self::sign_extend(
            (((instruction >> 31) & 1) << 12)
                | (((instruction >> 7) & 1) << 11)
                | (((instruction >> 25) & 63) << 5)
                | (((instruction >> 8) & 15) << 1),
            13,
        )
    }

    fn immediate_j(instruction: u32) -> u32 {
        Self::sign_extend(
            (((instruction >> 31) & 1) << 20)
                | (((instruction >> 12) & 255) << 12)
                | (((instruction >> 20) & 1) << 11)
                | (((instruction >> 21) & 1023) << 1),
            21,
        )
    }

    fn write_register(&mut self, register: usize, value: u32) {
        if register != 0 {
            self.regs[register] = value;
        }
    }

    pub fn run(&mut self, limit: usize) -> Result<(), EmulatorError> {
        for _ in 0..limit {
            if self.halted {
                return Ok(());
            }
            self.step()?;
        }

        if self.halted {
            Ok(())
        } else {
            Err(EmulatorError::StepLimitExceeded(limit))
        }
    }

    pub fn step(&mut self) -> Result<(), EmulatorError> {
        let pc = self.pc;
        let instruction = self.memory.read32(pc)?;
        self.pc = pc.wrapping_add(4);

        let opcode = instruction & 127;
        let destination = Self::rd(instruction);
        let source_1 = self.regs[Self::rs1(instruction)];
        let source_2 = self.regs[Self::rs2(instruction)];
        let funct3 = (instruction >> 12) & 7;
        let funct7 = (instruction >> 25) & 127;

        if self.trace {
            eprintln!("pc={pc:08x} insn={instruction:08x}");
        }

        match opcode {
            0x33 => {
                let value = match (funct3, funct7) {
                    (0, 0) => source_1.wrapping_add(source_2),
                    (0, 0x20) => source_1.wrapping_sub(source_2),
                    (1, 0) => source_1 << (source_2 & 31),
                    (2, 0) => ((source_1 as i32) < (source_2 as i32)) as u32,
                    (3, 0) => (source_1 < source_2) as u32,
                    (4, 0) => source_1 ^ source_2,
                    (5, 0) => source_1 >> (source_2 & 31),
                    (5, 0x20) => ((source_1 as i32) >> (source_2 & 31)) as u32,
                    (6, 0) => source_1 | source_2,
                    (7, 0) => source_1 & source_2,
                    _ => return Err(EmulatorError::IllegalInstruction(instruction)),
                };
                self.write_register(destination, value);
            }
            0x13 => {
                let immediate = Self::immediate_i(instruction);
                let value = match funct3 {
                    0 => source_1.wrapping_add(immediate),
                    2 => ((source_1 as i32) < (immediate as i32)) as u32,
                    3 => (source_1 < immediate) as u32,
                    4 => source_1 ^ immediate,
                    6 => source_1 | immediate,
                    7 => source_1 & immediate,
                    1 if funct7 == 0 => source_1 << (immediate & 31),
                    5 if funct7 == 0 => source_1 >> (immediate & 31),
                    5 if funct7 == 0x20 => ((source_1 as i32) >> (immediate & 31)) as u32,
                    _ => return Err(EmulatorError::IllegalInstruction(instruction)),
                };
                self.write_register(destination, value);
            }
            0x03 => {
                let address = source_1.wrapping_add(Self::immediate_i(instruction));
                let value = match funct3 {
                    0 => Self::sign_extend(self.memory.read8(address)? as u32, 8),
                    1 => Self::sign_extend(self.memory.read16(address)? as u32, 16),
                    2 => self.memory.read32(address)?,
                    4 => self.memory.read8(address)? as u32,
                    5 => self.memory.read16(address)? as u32,
                    _ => return Err(EmulatorError::IllegalInstruction(instruction)),
                };
                self.write_register(destination, value);
            }
            0x23 => {
                let address = source_1.wrapping_add(Self::immediate_s(instruction));
                match funct3 {
                    0 => self.memory.write8(address, source_2 as u8)?,
                    1 => self.memory.write16(address, source_2 as u16)?,
                    2 => self.memory.write32(address, source_2)?,
                    _ => return Err(EmulatorError::IllegalInstruction(instruction)),
                }
            }
            0x63 => {
                let take_branch = match funct3 {
                    0 => source_1 == source_2,
                    1 => source_1 != source_2,
                    4 => (source_1 as i32) < (source_2 as i32),
                    5 => (source_1 as i32) >= (source_2 as i32),
                    6 => source_1 < source_2,
                    7 => source_1 >= source_2,
                    _ => return Err(EmulatorError::IllegalInstruction(instruction)),
                };
                if take_branch {
                    self.pc = pc.wrapping_add(Self::immediate_b(instruction));
                }
            }
            0x6f => {
                self.write_register(destination, self.pc);
                self.pc = pc.wrapping_add(Self::immediate_j(instruction));
            }
            0x67 if funct3 == 0 => {
                let next_pc = source_1.wrapping_add(Self::immediate_i(instruction)) & !1;
                self.write_register(destination, self.pc);
                self.pc = next_pc;
            }
            0x37 => self.write_register(destination, instruction & 0xfffff000),
            0x17 => self.write_register(destination, pc.wrapping_add(instruction & 0xfffff000)),
            0x73 if instruction == 0x00000073 => self.halted = true,
            _ => return Err(EmulatorError::IllegalInstruction(instruction)),
        }

        self.regs[0] = 0;
        Ok(())
    }
}
