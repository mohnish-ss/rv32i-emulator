# RV32I Instruction Set Emulator

A non-cycle-accurate Rust emulator for a substantial subset of the base RV32I integer instruction set. It models registers, a program counter, byte-addressable memory, instruction decoding and execution, structured faults, optional tracing, and a raw-binary command-line workflow.

## Technical Highlights

- Models 32 32-bit integer registers and preserves the architectural zero register, `x0`.
- Fetches, decodes, and executes integer arithmetic, immediate, load/store, branch, jump, and upper-immediate instructions.
- Uses wrapping 32-bit arithmetic and explicit sign extension for immediates and signed loads.
- Provides byte-addressable little-endian memory with bounds and natural-alignment checks.
- Supports instruction tracing and a configurable execution step limit.
- Treats `ECALL` as a deliberate emulator halt convention.

## Architecture

`src/cpu.rs` implements register state, the program counter, immediate decoding, and fetch/decode/execute. `src/memory.rs` implements the checked little-endian memory model. `src/error.rs` defines structured errors for out-of-bounds access, misalignment, illegal instructions, and exhausted step limits. `src/main.rs` loads a raw binary at address zero and exposes tracing, memory-size, and step-limit options.

Implemented instructions include:

- Register arithmetic and logic: `ADD`, `SUB`, `SLL`, `SLT`, `SLTU`, `XOR`, `SRL`, `SRA`, `OR`, `AND`
- Immediate arithmetic and logic: `ADDI`, `SLTI`, `SLTIU`, `XORI`, `ORI`, `ANDI`, `SLLI`, `SRLI`, `SRAI`
- Loads and stores: `LB`, `LH`, `LW`, `LBU`, `LHU`, `SB`, `SH`, `SW`
- Control flow: `BEQ`, `BNE`, `BLT`, `BGE`, `BLTU`, `BGEU`, `JAL`, `JALR`
- Upper immediates: `LUI`, `AUIPC`
- Halt convention: `ECALL`

## Building

```bash
cargo build --release
```

## Running

Pass a raw little-endian instruction binary as the first argument:

```bash
cargo run -- program.bin --trace --memory-size 65536 --max-steps 100000
```

`--trace` is optional. The default memory size is 65,536 bytes and the default execution limit is 100,000 steps.

## Testing

```bash
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test
```

The integration tests cover arithmetic and `x0` behavior, the `ECALL` halt convention, little-endian memory, alignment faults, branching, and store/load execution.

## Technical Concepts

- Rust
- RISC-V
- RV32I
- CPU architecture
- Instruction decoding
- Sign extension
- Little-endian memory
- Bounds and alignment validation

## Limitations

The emulator loads raw binaries only. It does not implement an ELF loader, CSRs, interrupts, privilege levels, caches, floating point, atomics, compressed instructions, or the integer multiply/divide extension. Misaligned taken control-flow targets are reported by the next instruction fetch rather than at the branch or jump itself.
