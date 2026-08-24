use rv32i_emulator::Cpu;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!(
            "usage: rv32i-emulator program.bin [--trace] [--memory-size N] [--max-steps N]"
        );
        return;
    }

    let mut memory_size = 65_536_usize;
    let mut max_steps = 100_000_usize;
    let mut trace = false;
    let mut index = 2;

    while index < args.len() {
        match args[index].as_str() {
            "--trace" => trace = true,
            "--memory-size" => {
                index += 1;
                memory_size = args[index].parse().expect("memory size");
            }
            "--max-steps" => {
                index += 1;
                max_steps = args[index].parse().expect("step count");
            }
            _ => {}
        }
        index += 1;
    }

    let bytes = std::fs::read(&args[1]).expect("read program");
    let mut cpu = Cpu::new(memory_size);
    cpu.trace = trace;
    cpu.memory.load_program(&bytes).expect("load program");
    cpu.run(max_steps).expect("emulation");

    for (register, value) in cpu.regs.iter().enumerate() {
        println!("x{register:02} = 0x{value:08x}");
    }
}
