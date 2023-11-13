use std::{error::Error, fmt, process::Command};

use crate::{ConnectionTrait, Result, Window};

const SCRIPT: &str = r#"
set infoList to {}

-- List of processes
tell application "System Events"
    set procList to processes
end tell

-- Iterate processes
repeat with proc in procList
    try
        -- Windows
        set winList to windows of proc
        set winPid to id of proc

        -- Iterate windows
        repeat with win in winList
            set winName to name of win

            -- Add to list
            set end of infoList to "{" & winPid & "," & quoted form of winName & "}"
        end repeat
    on error errMsg
        -- do nothing
        log errMsg
    end try
end repeat

-- Output
copy infoList as string to stdout
"#;
const PERMISSION_ERROR: &str = "osascript is not allowed assistive access";

pub struct Connection;
impl ConnectionTrait for Connection {
    fn new() -> Result<Self> {
        Ok(Self)
    }
    fn window_titles(&self) -> Result<Vec<Window>> {
        let arguments = &["-ss", "-e", &format!("{}", SCRIPT)];
        let command = Command::new("osascript")
            .args(arguments)
            .output()
            .expect("failed to execute AppleScript command");

        let error = String::from_utf8_lossy(&command.stderr);

        match error.contains(PERMISSION_ERROR) {
            true => Err(WindowTitleError::NoAccessibilityPermission.into()),
            false => Ok(split(&String::from_utf8_lossy(&command.stdout))),
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum WindowTitleError {
    NoAccessibilityPermission,
}
impl fmt::Display for WindowTitleError {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        match self {
            WindowTitleError::NoAccessibilityPermission => write!(
                fmt,
                "Permission to use the accessibility API has not been granted"
            ),
        }
    }
}
impl Error for WindowTitleError {}

fn split(output: &str) -> Vec<Window> {
    let mut windows = Vec::new();
    
    // Output is in pattern: `"\{pid,'name'}{pid,'name'}"`
    for window in output.replace("\"", "").split("{") {
      let pid = window.split(",").next().unwrap().parse::<u32>().unwrap_or(0);
      let title = window.split("'").nth(1).unwrap_or("").to_string();

      windows.push(Window { title, pid });
    }

    windows
}
