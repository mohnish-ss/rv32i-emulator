use rv32i_emulator::{Cpu, EmulatorError};

fn instruction(cpu: &mut Cpu, address: u32, word: u32) {
    cpu.memory.write32(address, word).unwrap();
}

#[test]
fn arithmetic_x0_and_ecall() {
    let mut cpu = Cpu::new(64);
    instruction(&mut cpu, 0, 0x00500093); // addi x1, x0, 5
    instruction(&mut cpu, 4, 0x00308113); // addi x2, x1, 3
    instruction(&mut cpu, 8, 0x002081b3); // add x3, x1, x2
    instruction(&mut cpu, 12, 0x00000073); // ecall

    cpu.run(4).unwrap();

    assert_eq!(cpu.regs[1], 5);
    assert_eq!(cpu.regs[3], 13);
    assert_eq!(cpu.regs[0], 0);
    assert!(cpu.halted);
}

#[test]
fn little_endian_and_alignment() {
    let mut cpu = Cpu::new(8);
    cpu.memory.write32(0, 0x11223344).unwrap();

    assert_eq!(cpu.memory.read8(0).unwrap(), 0x44);
    assert_eq!(cpu.memory.read16(2).unwrap(), 0x1122);
    assert!(matches!(
        cpu.memory.read32(1),
        Err(EmulatorError::Unaligned { .. })
    ));
}

#[test]
fn store_and_load() {
    let mut cpu = Cpu::new(128);
    instruction(&mut cpu, 0, 0x00800093); // addi x1, x0, 8
    instruction(&mut cpu, 4, 0x00102023); // sw x1, 0(x0)
    instruction(&mut cpu, 8, 0x00002103); // lw x2, 0(x0)
    instruction(&mut cpu, 12, 0x00000073); // ecall

    cpu.run(4).unwrap();
    assert_eq!(cpu.regs[2], 8);
}

#[test]
fn taken_branch_skips_an_instruction() {
    let mut cpu = Cpu::new(64);
    instruction(&mut cpu, 0, 0x00100093); // addi x1, x0, 1
    instruction(&mut cpu, 4, 0x00108463); // beq x1, x1, +8
    instruction(&mut cpu, 8, 0x00700113); // addi x2, x0, 7 (skipped)
    instruction(&mut cpu, 12, 0x00900113); // addi x2, x0, 9
    instruction(&mut cpu, 16, 0x00000073); // ecall

    cpu.run(4).unwrap();
    assert_eq!(cpu.regs[2], 9);
}

#[test]
fn signed_byte_load_sign_extends() {
    let mut cpu = Cpu::new(64);
    instruction(&mut cpu, 0, 0x02000093); // addi x1, x0, 32
    instruction(&mut cpu, 4, 0x00008103); // lb x2, 0(x1)
    instruction(&mut cpu, 8, 0x00000073); // ecall
    cpu.memory.write8(32, 0xff).unwrap();

    cpu.run(3).unwrap();
    assert_eq!(cpu.regs[2], u32::MAX);
}

#[test]
fn execution_limit_is_reported_without_ecall() {
    let mut cpu = Cpu::new(16);
    instruction(&mut cpu, 0, 0x00000013); // addi x0, x0, 0

    assert_eq!(
        cpu.run(1),
        Err(EmulatorError::StepLimitExceeded(1))
    );
}
