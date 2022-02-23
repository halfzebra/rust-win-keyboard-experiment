use lcid::LanguageId;
use std::convert::TryInto;
use std::error::Error;
use std::path::PathBuf;
use windows::Win32::Foundation::{HINSTANCE, HWND, LPARAM, MAX_PATH, PSTR, WPARAM};
use windows::Win32::UI::Input::KeyboardAndMouse::{
    GetKeyboardLayout, GetKeyboardLayoutList, GetKeyboardLayoutNameA, LoadKeyboardLayoutA,
    KLF_ACTIVATE,
};
use windows::Win32::UI::TextServices::HKL;
use windows::Win32::UI::WindowsAndMessaging::{
    GetAncestor, GetForegroundWindow, PostMessageA, GA_ROOTOWNER, KL_NAMELENGTH,
    WM_INPUTLANGCHANGEREQUEST,
};

use windows::Win32::System::ProcessStatus::K32GetModuleFileNameExA;
use windows::Win32::System::Threading::{OpenProcess, PROCESS_QUERY_INFORMATION, PROCESS_VM_READ};
use windows::Win32::UI::WindowsAndMessaging::GetWindowThreadProcessId;

// https://stackoverflow.com/questions/263276/change-keyboard-layout-for-other-process

fn keyboard_layout_name() -> Result<String, Box<dyn Error>> {
    let np: [u8; KL_NAMELENGTH as usize] = [0; KL_NAMELENGTH as usize];

    unsafe { GetKeyboardLayoutNameA(PSTR(np.as_ptr() as *const u8)) }
        .expect("Windows API call GetKeyboardLayoutNameA succeeded");

    return String::from_utf8(np.iter().map(|v| v.clone()).collect::<Vec<u8>>())
        .map_err(|e| e.into());
}

fn get_window() -> Result<HWND, Box<dyn Error>> {
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

fn recover_lcid(current_lid: &HKL) -> Result<u16, Box<dyn Error>> {
    unsafe {
        std::slice::from_raw_parts(
            (current_lid as *const HKL) as *const u16,
            std::mem::size_of::<isize>() / 4,
        )
    }
    .get(0)
    .map(|v| v.clone())
    .ok_or("No Language Identifier returned by GetKeyboardLayout".into())
}

fn current_lcid(idthread: u32) -> Result<u16, Box<dyn Error>> {
    let current_lid = unsafe { GetKeyboardLayout(idthread) };

    println!("{:?} {}", &current_lid, keyboard_layout_name()?);

    if current_lid.is_invalid() {
        println!("failed to load current language");
    }

    recover_lcid(&current_lid)
}

fn change_lang(lid: &u16) -> Result<(), Box<dyn Error>> {
    let kla = unsafe { LoadKeyboardLayoutA(format!("{:08X}", lid), KLF_ACTIVATE) };

    if kla.is_invalid() {
        return Err("Failed to load language using LoadKeyboardLayoutA".into());
    }

    unsafe {
        PostMessageA(
            get_window()?,
            WM_INPUTLANGCHANGEREQUEST,
            WPARAM(0),
            LPARAM(kla.0 as isize),
        )
    }
    .ok()
    .map_err(|e| e.into())
}

fn get_current_lcids() -> Result<Vec<HKL>, Box<dyn Error>> {
    let vec: [HKL; 100] = [HKL(0); 100];
    let p = vec.as_ptr() as *mut HKL;
    let count = unsafe { GetKeyboardLayoutList(100, p) };

    for lid in vec[0..count as usize].iter() {
        if lid.is_invalid() {
            return Err("One of the LGIDs is invalid".into());
        }
    }

    Ok(vec[0..count as usize].to_vec())
}

fn get_current_langs<'a>() -> Result<Vec<&'a LanguageId>, Box<dyn Error>> {
    let mut langs: Vec<&LanguageId> = vec![];
    for res_lcid in get_current_lcids()?.iter().map(recover_lcid) {
        match res_lcid.and_then(|lcid| -> Result<&LanguageId, _> {
            let lang: Result<&LanguageId, _> = (lcid as u32).try_into();
            lang.map_err(|e| e.into())
        }) {
            Ok(langp) => langs.push(langp),
            Err(e) => return Err(e),
        };
    }
    Ok(langs)
}

fn get_foreground_process_thread_id() -> Result<u32, Box<dyn Error>> {
    let fgw = unsafe { GetForegroundWindow() };

    if fgw.is_invalid() {
        return Err("Failed to GetForegroundWindow".into());
    }

    let lpdwprocessid: u32 = 0;

    Ok(unsafe { GetWindowThreadProcessId(fgw, (&lpdwprocessid as *const u32) as *mut u32) })
}

// https://stackoverflow.com/questions/32590428/how-can-i-get-the-process-name-of-the-current-active-window-in-windows-with-wina
fn active_window_app_path() -> Result<PathBuf, Box<dyn Error>> {
    let fgw = unsafe { GetForegroundWindow() };

    if fgw.is_invalid() {
        return Err("Failed to GetForegroundWindow".into());
    }

    let lpdwprocessid: u32 = 0;
    // https://docs.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-getwindowthreadprocessid
    let thread_id =
        unsafe { GetWindowThreadProcessId(fgw, (&lpdwprocessid as *const u32) as *mut u32) };

    current_lcid(thread_id)?;

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

fn current_langid<'a>() -> Result<&'a LanguageId, Box<dyn Error>> {
    get_foreground_process_thread_id()
        .and_then(|tid| current_lcid(tid))
        .and_then(|lcid| {
            (lcid as u32)
                .try_into()
                .map_err(|e: lcid::LcidLookupError| e.into())
        })
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut last_program: Option<PathBuf> = None;

    loop {
        if let Ok(p) = active_window_app_path() {
            match &last_program {
                Some(lp) => if *lp != *p {},
                None => {}
            }

            last_program = Some(p.clone());
            let cur_prog = &p.file_name().unwrap();
            println!("{:?}", cur_prog);
            let cur_lang: &LanguageId = current_langid()?;
            let next_l: &LanguageId = "en-US".try_into().unwrap();
            println!("cur_lang {}", cur_lang.name);
            if *cur_prog == "Code.exe" && cur_lang.name != "en-US" {
                change_lang(&(next_l.lcid as u16))?;
            }
        }

        std::thread::sleep_ms(1000);
    }

    Ok(())
}
