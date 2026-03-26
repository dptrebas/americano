use clap::{Parser, Subcommand};
use std::process::Command;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Start keeping the machine awake
    Start {
        /// Minutes to keep awake (0 = indefinite until Ctrl+C or stop)
        #[arg(short, long, default_value_t = 60)]
        minutes: u64,

        /// Also keep the display/screen awake
        #[arg(short, long)]
        display: bool,

        /// Reason shown in system tools
        #[arg(short, long, default_value = "Americano - NautilOS keep-awake")]
        reason: String,
    },
    /// Stop any running Americano keep-awake session
    Stop,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Shared flag for graceful shutdown
    let running = Arc::new(AtomicBool::new(true));
    let running_clone = Arc::clone(&running);

    // Set up Ctrl+C handler
    ctrlc::set_handler(move || {
        running_clone.store(false, Ordering::SeqCst);
        println!("\nReceived Ctrl+C, shutting down gracefully...");
    })?;

    let cli = Cli::parse();

    match cli.command {
        Commands::Start { minutes, display, reason } => {
            start_keep_awake(minutes, display, &reason, running)?;
        }
        Commands::Stop => {
            stop_keep_awake()?;
        }
    }

    Ok(())
}

fn start_keep_awake(
    minutes: u64,
    display: bool,
    reason: &str,
    running: Arc<AtomicBool>,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("☕ Americano v{} — NautilOS keep-awake agent", env!("CARGO_PKG_VERSION"));
    println!("Starting keep-awake for {} minutes...", minutes);
    if display {
        println!("→ Display will also stay awake");
    }
    println!("Reason: {}", reason);

    let _guard = platform_start_keep_awake(minutes, display, reason)?;

    if minutes == 0 {
        println!("\n✅ Machine is now caffeinated **indefinitely**.");
        println!("Press Ctrl+C to stop...");

        while running.load(Ordering::SeqCst) {
            std::thread::sleep(Duration::from_secs(1));
        }
    } else {
        let duration = Duration::from_secs(minutes * 60);
        println!("\n✅ Machine is caffeinated for {} minutes.", minutes);

        let start = std::time::Instant::now();
        while running.load(Ordering::SeqCst) && start.elapsed() < duration {
            std::thread::sleep(Duration::from_secs(1));
        }
    }

    println!("\nAmericano Iced. Normal sleep behavior restored.");
    Ok(())
}

fn stop_keep_awake() -> Result<(), Box<dyn std::error::Error>> {
    println!("☕ Americano — Stopping keep-awake session...");
    println!("If Americano is running in another terminal, press Ctrl+C there.");
    println!("Normal sleep behavior should resume shortly.");
    Ok(())
}

// ==================== Platform-specific start ====================

#[cfg(target_os = "macos")]
fn platform_start_keep_awake(minutes: u64, display: bool, _reason: &str) -> Result<impl Drop, Box<dyn std::error::Error>> {
    let mut cmd = Command::new("caffeinate");
    cmd.arg("-i");                    // prevent system idle sleep
    if display {
        cmd.arg("-d");                // also prevent display sleep
    }
    if minutes > 0 {
        cmd.arg("-t").arg((minutes * 60).to_string());
    }

    let child = cmd.spawn()?;
    println!("→ Using macOS caffeinate (native & reliable)");
    Ok(Guard { child })
}

#[cfg(target_os = "linux")]
fn platform_start_keep_awake(_minutes: u64, _display: bool, reason: &str) -> Result<impl Drop, Box<dyn std::error::Error>> {
    println!("→ Using Linux systemd-inhibit");
    let child = Command::new("systemd-inhibit")
        .arg("--what=sleep:idle")
        .arg("--who=Americano")
        .arg(format!("--why={}", reason))
        .arg("--mode=block")
        .arg("sleep")
        .arg("infinity")
        .spawn()?;
    Ok(Guard { child })
}

#[cfg(target_os = "windows")]
fn platform_start_keep_awake(minutes: u64, display: bool, _reason: &str) -> Result<impl Drop, Box<dyn std::error::Error>> {
    use windows::Win32::System::Power::*;
    unsafe {
        let mut flags = ES_CONTINUOUS | ES_SYSTEM_REQUIRED;
        if display {
            flags |= ES_DISPLAY_REQUIRED;
        }
        SetThreadExecutionState(flags);
    }
    println!("→ Using Windows SetThreadExecutionState");
    Ok(WindowsGuard)
}

// Guard for macOS + Linux
struct Guard {
    child: std::process::Child,
}

impl Drop for Guard {
    fn drop(&mut self) {
        let _ = self.child.kill();
    }
}

// Windows guard
#[cfg(windows)]
struct WindowsGuard;

#[cfg(windows)]
impl Drop for WindowsGuard {
    fn drop(&mut self) {
        use windows::Win32::System::Power::*;
        unsafe { SetThreadExecutionState(ES_CONTINUOUS); }
    }
}