use std::error::Error;
use std::path::PathBuf;
use windows::Win32::Foundation::{HWND, MAX_PATH, HINSTANCE, PSTR};
use windows::Win32::UI::WindowsAndMessaging::{
    GetAncestor, GetForegroundWindow, GetWindowThreadProcessId, GA_ROOTOWNER,
};
use windows::Win32::System::Threading::{OpenProcess, PROCESS_QUERY_INFORMATION, PROCESS_VM_READ};
use windows::Win32::System::ProcessStatus::K32GetModuleFileNameExA;

pub fn get_window() -> Result<HWND, Box<dyn Error>> {
    let fgw = unsafe { GetForegroundWindow() };

    if fgw.is_invalid() {
        return Err("Failed to GetForegroundWindow".into());
    }

    let fgwa = unsafe { GetAncestor(fgw, GA_ROOTOWNER) };

    if fgwa.is_invalid() {
        return Err("Failed to GetAncestor".into());
    }

    Ok(fgwa)
}

pub fn get_foreground_process_thread_id() -> Result<u32, Box<dyn Error>> {
    let fgw = unsafe { GetForegroundWindow() };

    if fgw.is_invalid() {
        return Err("Failed to GetForegroundWindow".into());
    }

    let lpdwprocessid: u32 = 0;

    Ok(unsafe { GetWindowThreadProcessId(fgw, (&lpdwprocessid as *const u32) as *mut u32) })
}

// https://stackoverflow.com/questions/32590428/how-can-i-get-the-process-name-of-the-current-active-window-in-windows-with-wina
pub fn active_window_app_path() -> Result<PathBuf, Box<dyn Error>> {
    let fgw = unsafe { GetForegroundWindow() };

    if fgw.is_invalid() {
        return Err("Failed to GetForegroundWindow".into());
    }

    let lpdwprocessid: u32 = 0;
    // https://docs.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-getwindowthreadprocessid
    let _thread_id =
        unsafe { GetWindowThreadProcessId(fgw, (&lpdwprocessid as *const u32) as *mut u32) };

    let handle = unsafe {
        OpenProcess(
            PROCESS_QUERY_INFORMATION | PROCESS_VM_READ,
            false,
            lpdwprocessid,
        )
    };

    if handle.ok().is_ok() {
        let name: [u8; MAX_PATH as usize] = [0; MAX_PATH as usize];

        // https://docs.microsoft.com/en-us/windows/win32/api/psapi/nf-psapi-getmodulefilenameexa
        let name_len = unsafe {
            K32GetModuleFileNameExA(handle, HINSTANCE(0), PSTR(&name as *const u8), MAX_PATH)
        };

        return String::from_utf8(name[0..name_len as usize].to_vec())
            .map(|s| s.into())
            .map_err(|e| e.into());
    }

    Err("Counldn't recieve process handle using OpenProcess".into())
}
