use clap::Parser;
use std::env;

#[derive(Parser, Debug)]
#[command(name = "sysinfo")]
#[command(author, version, about = "Cross-platform system information tool using native APIs", long_about = None)]
struct Args {
    /// Show detailed information
    #[arg(short, long)]
    verbose: bool,

    /// Output format
    #[arg(short, long, value_enum, default_value_t = OutputFormat::Text)]
    format: OutputFormat,
}

#[derive(Debug, Clone, clap::ValueEnum)]
enum OutputFormat {
    Text,
    Json,
}

#[cfg(target_os = "windows")]
fn get_system_info(verbose: bool, format: &OutputFormat) {
    use std::mem;

    // Windows API structures
    #[repr(C)]
    #[allow(non_snake_case)]
    struct SystemInfo {
        w_processor_architecture: u16,
        w_reserved: u16,
        dw_page_size: u32,
        lp_minimum_application_address: *mut u8,
        lp_maximum_application_address: *mut u8,
        dw_active_processor_mask: usize,
        dw_number_of_processors: u32,
        dw_processor_type: u32,
        dw_allocation_granularity: u32,
        w_processor_level: u16,
        w_processor_revision: u16,
    }

    #[link(name = "kernel32")]
    unsafe extern "system" {
        fn GetSystemInfo(lpSystemInfo: *mut SystemInfo);
        fn GetComputerNameW(lpBuffer: *mut u16, nSize: *mut u32) -> i32;
    }

    unsafe {
        let mut sys_info: SystemInfo = mem::zeroed();
        GetSystemInfo(&mut sys_info);

        match format {
            OutputFormat::Json => {
                let mut buffer: [u16; 256] = [0; 256];
                let mut size: u32 = 256;
                let computer_name = if GetComputerNameW(buffer.as_mut_ptr(), &mut size) != 0 {
                    String::from_utf16_lossy(&buffer[..size as usize])
                } else {
                    "Unknown".to_string()
                };

                println!("{{");
                println!("  \"platform\": \"windows\",");
                println!("  \"processors\": {},", sys_info.dw_number_of_processors);
                println!("  \"page_size\": {},", sys_info.dw_page_size);
                println!("  \"architecture\": {},", sys_info.w_processor_architecture);
                println!(
                    "  \"allocation_granularity\": {},",
                    sys_info.dw_allocation_granularity
                );
                println!("  \"computer_name\": \"{}\"", computer_name);
                println!("}}");
            }
            OutputFormat::Text => {
                println!("=== System Information (via Windows API) ===");
                println!("Number of Processors: {}", sys_info.dw_number_of_processors);
                println!("Page Size: {} bytes", sys_info.dw_page_size);
                println!(
                    "Processor Architecture: {}",
                    sys_info.w_processor_architecture
                );

                if verbose {
                    println!("Processor Type: {}", sys_info.dw_processor_type);
                    println!("Processor Level: {}", sys_info.w_processor_level);
                    println!("Processor Revision: {}", sys_info.w_processor_revision);
                }

                println!(
                    "Allocation Granularity: {} bytes",
                    sys_info.dw_allocation_granularity
                );

                // Get computer name
                let mut buffer: [u16; 256] = [0; 256];
                let mut size: u32 = 256;
                if GetComputerNameW(buffer.as_mut_ptr(), &mut size) != 0 {
                    let computer_name = String::from_utf16_lossy(&buffer[..size as usize]);
                    println!("Computer Name: {}", computer_name);
                }
            }
        }
    }
}

#[cfg(target_os = "linux")]
fn get_system_info(verbose: bool, format: &OutputFormat) {
    use std::fs;

    match format {
        OutputFormat::Json => {
            println!("{{");
            println!("  \"platform\": \"linux\",");
            if let Ok(contents) = fs::read_to_string("/proc/cpuinfo") {
                let processor_count = contents
                    .lines()
                    .filter(|line| line.starts_with("processor"))
                    .count();
                println!("  \"processors\": {},", processor_count);
            }
            if let Ok(hostname) = fs::read_to_string("/etc/hostname") {
                println!("  \"hostname\": \"{}\"", hostname.trim());
            }
            println!("}}");
        }
        OutputFormat::Text => {
            println!("=== System Information (Linux) ===");

            if let Ok(contents) = fs::read_to_string("/proc/cpuinfo") {
                let processor_count = contents
                    .lines()
                    .filter(|line| line.starts_with("processor"))
                    .count();
                println!("Number of Processors: {}", processor_count);
            }

            if let Ok(contents) = fs::read_to_string("/proc/meminfo") {
                let lines_to_show = if verbose { 10 } else { 3 };
                for line in contents.lines().take(lines_to_show) {
                    println!("{}", line);
                }
            }

            if let Ok(hostname) = fs::read_to_string("/etc/hostname") {
                println!("Hostname: {}", hostname.trim());
            }
        }
    }
}

#[cfg(target_os = "macos")]
fn get_system_info(verbose: bool, format: &OutputFormat) {
    use std::process::Command;

    match format {
        OutputFormat::Json => {
            println!("{{");
            println!("  \"platform\": \"macos\",");
            if let Ok(output) = Command::new("sysctl").args(&["-n", "hw.ncpu"]).output() {
                if let Ok(cpu_count) = String::from_utf8(output.stdout) {
                    println!("  \"cpus\": {},", cpu_count.trim());
                }
            }
            if let Ok(output) = Command::new("sysctl").args(&["-n", "hw.memsize"]).output() {
                if let Ok(mem_size) = String::from_utf8(output.stdout) {
                    println!("  \"memory_bytes\": {}", mem_size.trim());
                }
            }
            println!("}}");
        }
        OutputFormat::Text => {
            println!("=== System Information (macOS) ===");

            if let Ok(output) = Command::new("sysctl").args(&["-n", "hw.ncpu"]).output() {
                if let Ok(cpu_count) = String::from_utf8(output.stdout) {
                    println!("Number of CPUs: {}", cpu_count.trim());
                }
            }

            if let Ok(output) = Command::new("sysctl").args(&["-n", "hw.memsize"]).output() {
                if let Ok(mem_size) = String::from_utf8(output.stdout) {
                    println!("Memory Size: {} bytes", mem_size.trim());
                }
            }

            if verbose {
                if let Ok(output) = Command::new("sysctl")
                    .args(&["-n", "machdep.cpu.brand_string"])
                    .output()
                {
                    if let Ok(cpu_brand) = String::from_utf8(output.stdout) {
                        println!("CPU: {}", cpu_brand.trim());
                    }
                }
            }
        }
    }
}

#[cfg(not(any(target_os = "windows", target_os = "linux", target_os = "macos")))]
fn get_system_info(_verbose: bool, _format: &OutputFormat) {
    println!("Unsupported platform");
}

fn main() {
    let args = Args::parse();

    if matches!(args.format, OutputFormat::Text) {
        println!("System Info Tool - Cross-platform CLI");
        println!("Built for: {} ({})", env::consts::OS, env::consts::ARCH);
        println!();
    }

    get_system_info(args.verbose, &args.format);

    if matches!(args.format, OutputFormat::Text) && args.verbose {
        println!();
        println!("This tool uses platform-specific system libraries:");
        println!("- Windows: kernel32.dll (Windows API)");
        println!("- Linux: /proc filesystem");
        println!("- macOS: sysctl command");
    }
}
