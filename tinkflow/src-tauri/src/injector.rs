use enigo::{Enigo, Keyboard, Settings};
#[cfg(not(target_os = "windows"))]
use std::io::Write;
#[cfg(not(target_os = "windows"))]
use std::process::{Command, Stdio};
use std::thread;
use std::time::Duration;

#[derive(Debug, Clone)]
pub struct InjectionReport {
    pub mode_used: String,
    pub succeeded: bool,
    pub error: Option<String>,
}

pub struct TextInjector {
    enigo: Enigo,
}

impl TextInjector {
    pub fn new() -> Result<Self, String> {
        let enigo = Enigo::new(&Settings::default()).map_err(|e| e.to_string())?;
        Ok(Self { enigo })
    }

    pub fn inject(&mut self, text: &str) -> Result<(), String> {
        self.inject_direct(text)
    }

    pub fn inject_with_mode(&mut self, text: &str, mode: &str) -> InjectionReport {
        match mode {
            "direct" => Self::report("direct", self.inject_direct(text)),
            "clipboard" => Self::report("clipboard", inject_via_clipboard(text)),
            _ => match self.inject_direct(text) {
                Ok(()) => InjectionReport {
                    mode_used: "direct".to_string(),
                    succeeded: true,
                    error: None,
                },
                Err(direct_error) => match inject_via_clipboard(text) {
                    Ok(()) => InjectionReport {
                        mode_used: "clipboard-fallback".to_string(),
                        succeeded: true,
                        error: None,
                    },
                    Err(clipboard_error) => InjectionReport {
                        mode_used: "auto".to_string(),
                        succeeded: false,
                        error: Some(format!(
                            "Direct injection failed: {}; clipboard fallback failed: {}",
                            direct_error, clipboard_error
                        )),
                    },
                },
            },
        }
    }

    fn inject_direct(&mut self, text: &str) -> Result<(), String> {
        self.enigo.text(text).map_err(|e| e.to_string())?;
        Ok(())
    }

    fn report(mode: &str, result: Result<(), String>) -> InjectionReport {
        match result {
            Ok(()) => InjectionReport {
                mode_used: mode.to_string(),
                succeeded: true,
                error: None,
            },
            Err(e) => InjectionReport {
                mode_used: mode.to_string(),
                succeeded: false,
                error: Some(e),
            },
        }
    }
}

pub fn copy_text_to_clipboard(text: &str) -> Result<(), String> {
    set_clipboard_text(text)
}

fn inject_via_clipboard(text: &str) -> Result<(), String> {
    set_clipboard_text(text)?;
    thread::sleep(Duration::from_millis(80));
    paste_clipboard()
}

#[cfg(target_os = "windows")]
fn set_clipboard_text(text: &str) -> Result<(), String> {
    use std::ffi::OsStr;
    use std::os::windows::ffi::OsStrExt;
    use std::ptr;
    use winapi::um::winbase::{
        GlobalAlloc, GlobalFree, GlobalLock, GlobalUnlock, GMEM_MOVEABLE,
    };
    use winapi::um::winuser::{
        CloseClipboard, EmptyClipboard, OpenClipboard, SetClipboardData, CF_UNICODETEXT,
    };

    let wide_text: Vec<u16> = OsStr::new(text).encode_wide().chain(Some(0)).collect();

    let mut opened = false;
    for _ in 0..10 {
        if unsafe { OpenClipboard(ptr::null_mut()) } != 0 {
            opened = true;
            break;
        }
        thread::sleep(Duration::from_millis(10));
    }

    if !opened {
        return Err(format!(
            "Failed to open clipboard: {}",
            std::io::Error::last_os_error()
        ));
    }

    let result = unsafe {
        if EmptyClipboard() == 0 {
            Err(format!(
                "Failed to empty clipboard: {}",
                std::io::Error::last_os_error()
            ))
        } else {
            let bytes = wide_text.len() * std::mem::size_of::<u16>();
            let memory = GlobalAlloc(GMEM_MOVEABLE, bytes);
            if memory.is_null() {
                Err(format!(
                    "Failed to allocate clipboard memory: {}",
                    std::io::Error::last_os_error()
                ))
            } else {
                let destination = GlobalLock(memory) as *mut u16;
                if destination.is_null() {
                    GlobalFree(memory);
                    Err(format!(
                        "Failed to lock clipboard memory: {}",
                        std::io::Error::last_os_error()
                    ))
                } else {
                    ptr::copy_nonoverlapping(wide_text.as_ptr(), destination, wide_text.len());
                    GlobalUnlock(memory);

                    if SetClipboardData(CF_UNICODETEXT, memory).is_null() {
                        GlobalFree(memory);
                        Err(format!(
                            "Failed to set clipboard text: {}",
                            std::io::Error::last_os_error()
                        ))
                    } else {
                        Ok(())
                    }
                }
            }
        }
    };

    unsafe {
        CloseClipboard();
    }

    result
}

#[cfg(target_os = "macos")]
fn set_clipboard_text(text: &str) -> Result<(), String> {
    let mut child = Command::new("pbcopy")
        .stdin(Stdio::piped())
        .spawn()
        .map_err(|e| format!("Failed to start pbcopy: {}", e))?;
    if let Some(stdin) = child.stdin.as_mut() {
        stdin
            .write_all(text.as_bytes())
            .map_err(|e| e.to_string())?;
    }
    child.wait().map_err(|e| e.to_string()).and_then(|status| {
        if status.success() {
            Ok(())
        } else {
            Err(format!("pbcopy exited with status {}", status))
        }
    })
}

#[cfg(target_os = "linux")]
fn set_clipboard_text(text: &str) -> Result<(), String> {
    for command in ["wl-copy", "xclip"] {
        let mut child = match Command::new(command).stdin(Stdio::piped()).spawn() {
            Ok(child) => child,
            Err(_) => continue,
        };
        if let Some(stdin) = child.stdin.as_mut() {
            stdin
                .write_all(text.as_bytes())
                .map_err(|e| e.to_string())?;
        }
        let status = child.wait().map_err(|e| e.to_string())?;
        if status.success() {
            return Ok(());
        }
    }
    Err("No supported clipboard command found (wl-copy or xclip)".to_string())
}

#[cfg(target_os = "windows")]
fn paste_clipboard() -> Result<(), String> {
    use winapi::um::winuser::{keybd_event, KEYEVENTF_KEYUP, VK_CONTROL};
    unsafe {
        keybd_event(VK_CONTROL as u8, 0, 0, 0);
        keybd_event(0x56, 0, 0, 0);
        keybd_event(0x56, 0, KEYEVENTF_KEYUP, 0);
        keybd_event(VK_CONTROL as u8, 0, KEYEVENTF_KEYUP, 0);
    }
    Ok(())
}

#[cfg(target_os = "macos")]
fn paste_clipboard() -> Result<(), String> {
    Command::new("osascript")
        .args([
            "-e",
            "tell application \"System Events\" to keystroke \"v\" using command down",
        ])
        .status()
        .map_err(|e| e.to_string())
        .and_then(|status| {
            if status.success() {
                Ok(())
            } else {
                Err(format!("osascript exited with status {}", status))
            }
        })
}

#[cfg(target_os = "linux")]
fn paste_clipboard() -> Result<(), String> {
    for (command, args) in [
        ("wtype", vec!["-M", "ctrl", "v", "-m", "ctrl"]),
        ("xdotool", vec!["key", "ctrl+v"]),
    ] {
        if let Ok(status) = Command::new(command).args(args).status() {
            if status.success() {
                return Ok(());
            }
        }
    }
    Err("No supported paste command found (wtype or xdotool)".to_string())
}
