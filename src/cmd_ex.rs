use std::process::Command;

pub trait CommandExt {
    fn with_no_window(&mut self) -> &mut Self;
}

impl CommandExt for Command {
    /// On Windows, CLI applications that aren't the window's subsystem will
    /// create and show a console window that pops up next to the main
    /// application window when run. We disable this behavior by setting the
    /// `CREATE_NO_WINDOW` flag.
    /// see https://learn.microsoft.com/en-us/windows/win32/procthread/process-creation-flags
    #[inline]
    fn with_no_window(&mut self) -> &mut Self {
        #[cfg(windows)]
        {
            use std::os::windows::process::CommandExt;
            self.creation_flags(0x0800_0000)
        }

        #[cfg(not(windows))]
        {
            self
        }
    }
}
