//! This module contains the commands that can be used for windows systems.

use super::{IAlternateScreenCommand, IEnableAnsiCommand, TerminalOutput};

use kernel::windows_kernel::{ansi_support, csbi, handle, kernel};
use winapi::shared::minwindef::DWORD;
use winapi::um::wincon;
use winapi::um::wincon::ENABLE_VIRTUAL_TERMINAL_PROCESSING;

use std::io::{Error, ErrorKind, Result};

/// This command is used for enabling and disabling ANSI code support for windows systems,
/// For more info check: https://docs.microsoft.com/en-us/windows/console/console-virtual-terminal-sequences.
#[derive(Clone, Copy)]
pub struct EnableAnsiCommand {
    mask: DWORD,
}

impl EnableAnsiCommand {
    pub fn new() -> EnableAnsiCommand {
        let command = EnableAnsiCommand {
            mask: ENABLE_VIRTUAL_TERMINAL_PROCESSING,
        };
        command
    }
}

impl IEnableAnsiCommand for EnableAnsiCommand {
    fn enable(&self) -> bool {
        // we need to check whether we tried to enable ansi before. If we have we can just return if that had succeeded.
        if ansi_support::has_been_tried_to_enable_ansi() && ansi_support::ansi_enabled() {
            return ansi_support::windows_supportable();
        } else {
            let output_handle = handle::get_output_handle().unwrap();

            let mut dw_mode: DWORD = 0;
            if !kernel::get_console_mode(&output_handle, &mut dw_mode) {
                return false;
            }

            dw_mode |= self.mask;
            if !kernel::set_console_mode(&output_handle, dw_mode) {
                return false;
            }
            return true;
        }
    }

    fn disable(&self) -> bool {
        if ansi_support::ansi_enabled() {
            let output_handle = handle::get_output_handle().unwrap();

            let mut dw_mode: DWORD = 0;
            if !kernel::get_console_mode(&output_handle, &mut dw_mode) {
                return false;
            }

            dw_mode &= !self.mask;
            if !kernel::set_console_mode(&output_handle, dw_mode) {
                return false;
            }

            ansi_support::set_ansi_enabled(false);
        }
        return true;
    }
}

/// This command is used for enabling and disabling raw mode for windows systems.
/// For more info check: https://docs.microsoft.com/en-us/windows/console/high-level-console-modes.
#[derive(Clone, Copy)]
pub struct RawModeCommand {
    mask: DWORD,
}
use self::wincon::{ENABLE_LINE_INPUT, ENABLE_WRAP_AT_EOL_OUTPUT};
impl RawModeCommand {
    pub fn new() -> Self {
        RawModeCommand {
            mask: ENABLE_WRAP_AT_EOL_OUTPUT | ENABLE_LINE_INPUT,
        }
    }
}

impl RawModeCommand {
    /// Enables raw mode.
    pub fn enable(&mut self) -> Result<()> {
        let mut dw_mode: DWORD = 0;
        let stdout = handle::get_output_handle().unwrap();

        if !kernel::get_console_mode(&stdout, &mut dw_mode) {
            return Err(Error::new(
                ErrorKind::Other,
                "Could not get console mode when enabling raw mode",
            ));
        }

        let new_mode = dw_mode & !self.mask;

        if !kernel::set_console_mode(&stdout, new_mode) {
            return Err(Error::new(
                ErrorKind::Other,
                "Could not set console mode when enabling raw mode",
            ));
        }

        Ok(())
    }

    /// Disables raw mode.
    pub fn disable(&self) -> Result<()> {
        let stdout = handle::get_output_handle().unwrap();

        let mut dw_mode: DWORD = 0;
        if !kernel::get_console_mode(&stdout, &mut dw_mode) {
            return Err(Error::new(
                ErrorKind::Other,
                "Could not get console mode when disabling raw mode",
            ));
        }

        let new_mode = dw_mode | self.mask;

        if !kernel::set_console_mode(&stdout, new_mode) {
            return Err(Error::new(
                ErrorKind::Other,
                "Could not set console mode when disabling raw mode",
            ));
        }

        return Ok(());
    }
}

/// This command is used for switching to alternate screen and back to main screen.
/// check https://docs.microsoft.com/en-us/windows/console/reading-and-writing-blocks-of-characters-and-attributes for more info
pub struct ToAlternateScreenCommand;

impl ToAlternateScreenCommand {
    pub fn new() -> ToAlternateScreenCommand {
        return ToAlternateScreenCommand {};
    }
}

impl IAlternateScreenCommand for ToAlternateScreenCommand {
    fn enable(&self, _stdout: &mut TerminalOutput) -> Result<()> {
        let _handle = handle::get_output_handle()?;

        // create a new screen buffer to copy to.
        let new_handle = csbi::create_console_screen_buffer();

        // Make the new screen buffer the active screen buffer.
        csbi::set_active_screen_buffer(new_handle)?;

        Ok(())
    }

    fn disable(&self, _stdout: &TerminalOutput) -> Result<()> {
        let handle = handle::get_output_handle()?;
        csbi::set_active_screen_buffer(handle);

        Ok(())
    }
}
