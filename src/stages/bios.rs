use super::InstallationStage;
use crate::config::BiosConfig;
use crate::ui::{ProgressBar, ProgressStyle, Spinner};
use chrono::Local;
use colored::*;
use rand::Rng;
use std::io::{self, Write};
use std::thread;
use std::time::Duration;
use sysinfo::System;

static BIOS_BOX_WIDTH: usize = 41;

pub struct BiosStage {
    config: BiosConfig,
}

struct SystemInfo {
    cpu_brand: String,
    cpu_count: usize,
    cpu_freq: u64,
    total_memory_kb: u64,
    hostname: String,
    network_count: usize,
    disk_count: usize,
    os_name: String,
}

impl BiosStage {
    pub fn new(config: BiosConfig) -> Self {
        Self { config }
    }

    fn get_system_info() -> SystemInfo {
        use sysinfo::{Disks, Networks};

        let mut sys = System::new_all();
        sys.refresh_all();

        let cpu_brand = sys
            .cpus()
            .first()
            .map(|cpu| cpu.brand().to_string())
            .unwrap_or_else(|| "Unknown CPU".to_string());

        let cpu_freq = sys.cpus().first().map(|cpu| cpu.frequency()).unwrap_or(0);

        let total_memory_kb = sys.total_memory() / 1024;
        let cpu_count = sys.cpus().len();

        let hostname = System::host_name().unwrap_or_else(|| "SYSTEM-PC".to_string());

        let networks = Networks::new_with_refreshed_list();
        let network_count = networks.iter().count();

        let disks = Disks::new_with_refreshed_list();
        let disk_count = disks.iter().count();

        let os_name = System::name().unwrap_or_else(|| "Unknown OS".to_string());

        SystemInfo {
            cpu_brand,
            cpu_count,
            cpu_freq,
            total_memory_kb,
            hostname,
            network_count,
            disk_count,
            os_name,
        }
    }
}

impl InstallationStage for BiosStage {
    fn name(&self) -> &'static str {
        "BIOS/Firmware Update Sequence"
    }

    fn run(&self, exit_check: &dyn Fn() -> bool) -> io::Result<()> {
        println!("\n{}", format!("> {}", self.name()).bright_yellow().bold());
        println!();

        let mut rng = rand::thread_rng();
        let sys_info = Self::get_system_info();

        let now = Local::now();
        let bios_serial = format!(
            "{:04X}-{:04X}-{:04X}-{:04X}",
            rng.gen::<u16>(),
            rng.gen::<u16>(),
            rng.gen::<u16>(),
            rng.gen::<u16>()
        );

        let box_width = BIOS_BOX_WIDTH;
        let inner_width = box_width - 3;

        println!(
            "{}",
            format!("╔{}╗", "═".repeat(box_width - 2)).bright_cyan()
        );
        println!(
            "{}",
            format!("║ {:<width$}║", self.config.vendor, width = inner_width).bright_cyan()
        );
        println!(
            "{}",
            format!("║ {:<width$}║", self.config.version, width = inner_width).bright_cyan()
        );
        println!(
            "{}",
            format!("╚{}╝", "═".repeat(box_width - 2)).bright_cyan()
        );
        println!();
        println!(
            "{}",
            format!("BIOS Date: {}  S/N: {}", self.config.bios_date, bios_serial).dimmed()
        );
        println!(
            "{}",
            format!(
                "System Date: {}  Time: {}",
                now.format("%m/%d/%Y"),
                now.format("%H:%M:%S")
            )
            .dimmed()
        );
        println!("{}", format!("System Name: {}", sys_info.hostname).dimmed());
        thread::sleep(Duration::from_millis(self.config.header_delay));

        println!();
        println!(
            "{}",
            "Performing POST (Power-On Self Test)...".bright_white()
        );
        thread::sleep(Duration::from_millis(self.config.post_start_delay));

        let mut spinner = Spinner::new();

        spinner.animate(
            &format!("CPU: {}", sys_info.cpu_brand),
            self.config.cpu_detect_time,
            exit_check,
        )?;
        spinner.animate(
            &format!("CPU Cores: {} physical", sys_info.cpu_count),
            self.config.cpu_cores_time,
            exit_check,
        )?;

        if sys_info.cpu_freq > 0 {
            let freq_ghz = sys_info.cpu_freq as f64 / 1000.0;
            spinner.animate(
                &format!("CPU Speed: {:.2} GHz", freq_ghz),
                self.config.cpu_freq_time,
                exit_check,
            )?;
        }

        println!();
        let memory_mb = sys_info.total_memory_kb / 1024;
        let memory_gb = memory_mb as f64 / 1024.0;

        print!("{}", "Testing Memory: ".bright_white());
        io::stdout().flush()?;

        let mem_progress = ProgressBar::new(ProgressStyle::Hash);
        let steps = 40;
        let delay = self.config.memory_test_time / steps;

        for i in 0..=steps {
            if exit_check() {
                return Err(io::Error::new(io::ErrorKind::Interrupted, "User interrupt"));
            }
            let progress = i as f32 / steps as f32;
            let tested_kb = (sys_info.total_memory_kb as f32 * progress) as u64;
            print!(
                "\rTesting Memory: {} {}/{} KB",
                mem_progress.render(progress),
                tested_kb,
                sys_info.total_memory_kb
            );
            io::stdout().flush()?;
            thread::sleep(Duration::from_millis(delay));
        }
        println!(" {}", "OK".bright_green());

        spinner.animate(
            &format!(
                "Total System Memory: {:.2} GB ({} MB)",
                memory_gb, memory_mb
            ),
            self.config.memory_details_time,
            exit_check,
        )?;

        if rng.gen_bool(self.config.cmos_error_chance) {
            println!(
                "{}",
                "WARNING: CMOS checksum invalid, loading defaults".yellow()
            );
            thread::sleep(Duration::from_millis(self.config.cmos_warning_time));
        }

        println!();
        println!("{}", "Detecting IDE Devices...".bright_white());

        print!("  Primary Master   [0x1F0-0x1F7]: ");
        io::stdout().flush()?;
        thread::sleep(Duration::from_millis(self.config.ide_master_time));
        println!("{}", "WDC WD2000JB-00GVC0".bright_green());

        print!("  Primary Slave    [0x1F0-0x1F7]: ");
        io::stdout().flush()?;
        thread::sleep(Duration::from_millis(self.config.ide_slave_time));
        println!("{}", "None".dimmed());

        print!("  Secondary Master [0x170-0x177]: ");
        io::stdout().flush()?;
        thread::sleep(Duration::from_millis(self.config.ide_master_time));
        println!("{}", "ATAPI CD-ROM".bright_green());

        print!("  Secondary Slave  [0x170-0x177]: ");
        io::stdout().flush()?;
        thread::sleep(Duration::from_millis(self.config.ide_slave_time));
        println!("{}", "None".dimmed());

        println!();
        println!("{}", "Scanning PCI bus...".bright_white());

        let pci_progress = ProgressBar::new(ProgressStyle::Block);
        print!("  Probing 00:00.0 - 00:1F.7: ");
        io::stdout().flush()?;

        let steps = 30;
        let delay = self.config.pci_scan_time / steps;
        for i in 0..=steps {
            if exit_check() {
                return Err(io::Error::new(io::ErrorKind::Interrupted, "User interrupt"));
            }
            let progress = i as f32 / steps as f32;
            print!(
                "\r  Probing 00:00.0 - 00:1F.7: {}",
                pci_progress.render(progress)
            );
            io::stdout().flush()?;
            thread::sleep(Duration::from_millis(delay));
        }
        println!();

        thread::sleep(Duration::from_millis(200));
        let pci_addr1 = format!("00:{:02X}.0", rng.gen_range(0x02..0x10));
        let pci_addr2 = format!("00:{:02X}.0", rng.gen_range(0x10..0x1F));
        println!(
            "  Found {} - VGA Compatible Controller",
            pci_addr1.bright_cyan()
        );
        thread::sleep(Duration::from_millis(self.config.pci_device_time));
        println!("  Found {} - Ethernet Controller", pci_addr2.bright_cyan());
        thread::sleep(Duration::from_millis(self.config.pci_device_time));
        println!("  Found {} - SMBus Controller", "00:1F.3".bright_cyan());
        thread::sleep(Duration::from_millis(self.config.pci_device_time));

        println!();
        spinner.animate(
            &format!("Network Adapters: {} detected", sys_info.network_count),
            self.config.network_detect_time,
            exit_check,
        )?;
        spinner.animate(
            "USB Controller: UHCI/EHCI Compatible",
            self.config.usb_detect_time,
            exit_check,
        )?;
        spinner.animate(
            "USB Device(s): 0 connected",
            self.config.usb_detect_time,
            exit_check,
        )?;

        println!();
        spinner.animate(
            &format!("Host OS: {}", sys_info.os_name),
            self.config.system_info_time,
            exit_check,
        )?;
        spinner.animate(
            &format!("Storage Devices: {} disk(s) found", sys_info.disk_count),
            self.config.system_info_time,
            exit_check,
        )?;

        let system_uuid = format!(
            "{:08X}-{:04X}-{:04X}-{:04X}-{:012X}",
            rng.gen::<u32>(),
            rng.gen::<u16>(),
            rng.gen::<u16>(),
            rng.gen::<u16>(),
            rng.gen::<u64>() & 0xFFFFFFFFFFFF
        );
        spinner.animate(
            &format!("System UUID: {}", system_uuid),
            self.config.uuid_time,
            exit_check,
        )?;

        println!();
        spinner.animate(
            "Boot Device Priority:",
            self.config.boot_priority_time,
            exit_check,
        )?;
        println!("  1st: {}", "Hard Disk Drive".bright_green());
        println!("  2nd: {}", "CD-ROM Drive".dimmed());
        println!("  3rd: {}", "Network Boot".dimmed());
        thread::sleep(Duration::from_millis(self.config.boot_display_time));

        println!();
        let sep = "═".repeat(BIOS_BOX_WIDTH as usize);
        println!("{}", sep.bright_yellow());
        println!(
            "{}",
            "  CRITICAL: Firmware Update Sequence Initiated"
                .bright_yellow()
                .bold()
        );
        println!("{}", sep.bright_yellow());
        thread::sleep(Duration::from_millis(self.config.firmware_header_delay));

        spinner.animate(
            "Backing up current BIOS to NVRAM...",
            self.config.backup_time,
            exit_check,
        )?;
        spinner.animate(
            "Verifying backup integrity... CRC32 OK",
            self.config.verify_time,
            exit_check,
        )?;

        println!();
        println!(
            "{}",
            "  WARNING: Do NOT power off or restart during this process!"
                .yellow()
                .bold()
        );
        println!(
            "{}",
            "  System damage may occur if interrupted!".yellow().bold()
        );
        println!();
        thread::sleep(Duration::from_millis(self.config.warning_delay));

        let progress = ProgressBar::new(ProgressStyle::Block);
        progress.animate(
            "Erasing flash sectors:",
            rng.gen_range(self.config.erase_min..self.config.erase_max),
            exit_check,
        )?;

        progress.animate(
            "Writing new firmware:",
            rng.gen_range(self.config.write_min..self.config.write_max),
            exit_check,
        )?;

        progress.animate(
            "Verifying firmware:",
            rng.gen_range(self.config.verify_min..self.config.verify_max),
            exit_check,
        )?;

        println!();
        spinner.animate(
            "Firmware update complete!",
            self.config.complete_time,
            exit_check,
        )?;
        spinner.animate(
            "Updating ESCD (Extended System Configuration Data)...",
            self.config.escd_time,
            exit_check,
        )?;

        println!();
        println!(
            "{}",
            format!(
                "BIOS update successful - {} -> {}",
                self.config.version, self.config.new_version
            )
            .bright_green()
            .bold()
        );
        println!(
            "{}",
            "System will initialize with new firmware".bright_green()
        );
        thread::sleep(Duration::from_millis(self.config.success_delay));

        Ok(())
    }
}
