// ── Windows-only imports ─────────────────────────────────────────────────────
#[cfg(target_os = "windows")]
use std::ffi::OsString;
#[cfg(target_os = "windows")]
use std::os::windows::ffi::OsStringExt;

/// Detects the current user context based on the active foreground window.
#[derive(Clone)]
pub struct ContextDetector;

impl ContextDetector {
    pub fn new() -> Self {
        Self
    }

    /// Detect the current context by reading the active window title.
    /// Returns one of: "code", "comment", "chat", "email", "terminal", "general"
    pub fn detect_current_context(&self) -> String {
        let title = get_foreground_window_title().unwrap_or_default().to_lowercase();

        if title.is_empty() {
            return "general".to_string();
        }

        // Code editors / IDEs
        if title.contains("visual studio code")
            || title.contains("- vs code")
            || title.contains("intellij")
            || title.contains("webstorm")
            || title.contains("pycharm")
            || title.contains("rustrover")
            || title.contains("sublime text")
            || title.contains("neovim")
            || title.contains("vim")
            || title.contains("emacs")
            || title.contains("notepad++")
        {
            // Check for specific file extensions in the title
            if title.contains(".rs")
                || title.contains(".py")
                || title.contains(".js")
                || title.contains(".ts")
                || title.contains(".tsx")
                || title.contains(".jsx")
                || title.contains(".go")
                || title.contains(".java")
                || title.contains(".cpp")
                || title.contains(".c")
                || title.contains(".cs")
                || title.contains(".rb")
                || title.contains(".php")
                || title.contains(".swift")
                || title.contains(".kt")
            {
                return "code".to_string();
            }
            if title.contains(".md") || title.contains(".txt") || title.contains(".rst") {
                return "general".to_string();
            }
            return "code".to_string();
        }

        // Chat applications
        if title.contains("slack")
            || title.contains("discord")
            || title.contains("telegram")
            || title.contains("whatsapp")
            || title.contains("microsoft teams")
            || title.contains("signal")
        {
            return "chat".to_string();
        }

        // Email
        if title.contains("gmail")
            || title.contains("outlook")
            || title.contains("thunderbird")
            || title.contains("mail")
            || title.contains("inbox")
        {
            return "email".to_string();
        }

        // Terminal / CLI
        if title.contains("powershell")
            || title.contains("command prompt")
            || title.contains("cmd.exe")
            || title.contains("terminal")
            || title.contains("windows terminal")
            || title.contains("wezterm")
            || title.contains("alacritty")
            || title.contains("warp")
        {
            return "terminal".to_string();
        }

        "general".to_string()
    }
}

// ── Platform-specific foreground window title detection ───────────────────────

/// Windows implementation using WinAPI.
#[cfg(target_os = "windows")]
fn get_foreground_window_title() -> Option<String> {
    unsafe {
        let hwnd = winapi::um::winuser::GetForegroundWindow();
        if hwnd.is_null() {
            return None;
        }

        let mut title_buf: [u16; 512] = [0; 512];
        let len = winapi::um::winuser::GetWindowTextW(
            hwnd,
            title_buf.as_mut_ptr(),
            title_buf.len() as i32,
        );

        if len <= 0 {
            return None;
        }

        let os_string = OsString::from_wide(&title_buf[..len as usize]);
        os_string.into_string().ok()
    }
}

/// macOS implementation using `osascript` to query System Events.
/// Requires Accessibility permissions to be granted to the app.
#[cfg(target_os = "macos")]
fn get_foreground_window_title() -> Option<String> {
    let output = std::process::Command::new("osascript")
        .arg("-e")
        .arg(
            "tell application \"System Events\"\n\
             set frontApp to first application process whose frontmost is true\n\
             set appName to name of frontApp\n\
             try\n\
                 set windowTitle to name of front window of frontApp\n\
                 return appName & \" - \" & windowTitle\n\
             on error\n\
                 return appName\n\
             end try\n\
             end tell",
        )
        .output()
        .ok()?;

    if output.status.success() {
        String::from_utf8(output.stdout)
            .ok()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
    } else {
        None
    }
}

/// Fallback stub for all other platforms — returns None (context defaults to "general").
#[cfg(not(any(target_os = "windows", target_os = "macos")))]
fn get_foreground_window_title() -> Option<String> {
    None
}
