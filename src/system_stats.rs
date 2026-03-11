use sysinfo::System;

pub fn obter_dados_hardware(sys: &mut System) -> (f32, f32) {
    // Refresh apenas da CPU e Memória para ser mais leve
    sys.refresh_cpu_all();
    sys.refresh_memory();

    // Cálculo da CPU: média de todos os núcleos
    let cpus = sys.cpus();
    let cpu_uso = if !cpus.is_empty() {
        cpus.iter().map(|cpu| cpu.cpu_usage()).sum::<f32>() / cpus.len() as f32
    } else {
        0.0
    };

    // Cálculo da RAM: Convertendo de KB para GB
   let ram_uso = sys.used_memory() as f32 / 1_048_576.0; // KB para GB é 1024 / 1024

   (cpu_uso, ram_uso)
}