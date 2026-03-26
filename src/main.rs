use clap::Parser;
use std::process::Command;
use std::time::Duration;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Minutes to keep the machine awake (0 = indefinite until Ctrl+C)
    #[arg(short, long, default_value_t = 60)]
    minutes: u64,

    /// Also keep the display/screen awake
    #[arg(short, long)]
    display: bool,

    /// Reason shown in system tools (e.g. in `pmset -g assertions` on macOS)
    #[arg(short, long, default_value = "Americano - NautilOS keep-awake")]
    reason: String,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    println!("☕ Americano v{} — NautilOS keep-awake agent", env!("CARGO_PKG_VERSION"));
    println!("Keeping machine awake for {} minutes...", args.minutes);
    if args.display {
        println!("→ Display will also stay awake");
    }
    println!("Reason: {}", args.reason);

    // Start platform-specific keep-awake
    let _guard = start_keep_awake(args.minutes, args.display, &args.reason)?;

    if args.minutes == 0 {
        println!("\n✅ Machine is now caffeinated **indefinitely**.");
        println!("Press Ctrl+C to stop...");
        std::thread::park();
    } else {
        let duration = Duration::from_secs(args.minutes * 60);
        println!("\n✅ Machine is caffeinated for {} minutes.", args.minutes);
        std::thread::sleep(duration);
    }

    println!("\nAmericano Iced. Normal sleep behavior restored.");
    Ok(())
}

// ==================== Platform-specific implementations ====================

#[cfg(target_os = "macos")]
fn start_keep_awake(minutes: u64, display: bool, _reason: &str) -> Result<impl Drop, Box<dyn std::error::Error>> {
    let mut cmd = Command::new("caffeinate");

    // Always prevent system idle sleep
    cmd.arg("-i");

    // If --display is passed, also prevent display sleep
    if display {
        cmd.arg("-d");
    }

    // Set timeout only if not running indefinitely
    if minutes > 0 {
        cmd.arg("-t").arg((minutes * 60).to_string());
    }

    let child = cmd.spawn()?;
    println!("→ Using macOS caffeinate (native & reliable)");
    Ok(Guard { child })
}

#[cfg(target_os = "linux")]
fn start_keep_awake(_minutes: u64, _display: bool, reason: &str) -> Result<impl Drop, Box<dyn std::error::Error>> {
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
fn start_keep_awake(minutes: u64, display: bool, _reason: &str) -> Result<impl Drop, Box<dyn std::error::Error>> {
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

// Guard for cleaning up external processes (macOS + Linux)
struct Guard {
    child: std::process::Child,
}

impl Drop for Guard {
    fn drop(&mut self) {
        let _ = self.child.kill();
    }
}

// Windows guard (no child process)
#[cfg(windows)]
struct WindowsGuard;

#[cfg(windows)]
impl Drop for WindowsGuard {
    fn drop(&mut self) {
        use windows::Win32::System::Power::*;
        unsafe { SetThreadExecutionState(ES_CONTINUOUS); }
    }
}