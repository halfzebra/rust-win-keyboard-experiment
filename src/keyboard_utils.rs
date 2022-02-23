use crate::window_utils;
use lcid::LanguageId;
use std::convert::TryInto;
use std::error::Error;
use windows::Win32::Foundation::{LPARAM, WPARAM};
use windows::Win32::UI::Input::KeyboardAndMouse::{
    GetKeyboardLayout, GetKeyboardLayoutList, LoadKeyboardLayoutA, KLF_ACTIVATE,
};
use windows::Win32::UI::TextServices::HKL;
use windows::Win32::UI::WindowsAndMessaging::{PostMessageA, WM_INPUTLANGCHANGEREQUEST};

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

pub fn get_current_langs<'a>() -> Result<Vec<&'a LanguageId>, Box<dyn Error>> {
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

    if current_lid.is_invalid() {
        println!("failed to load current language");
    }

    recover_lcid(&current_lid)
}

// https://stackoverflow.com/questions/263276/change-keyboard-layout-for-other-process
pub fn change_lang(lid: &u16) -> Result<(), Box<dyn Error>> {
    let kla = unsafe { LoadKeyboardLayoutA(format!("{:08X}", lid), KLF_ACTIVATE) };

    if kla.is_invalid() {
        return Err("Failed to load language using LoadKeyboardLayoutA".into());
    }

    unsafe {
        PostMessageA(
            window_utils::get_window()?,
            WM_INPUTLANGCHANGEREQUEST,
            WPARAM(0),
            LPARAM(kla.0 as isize),
        )
    }
    .ok()
    .map_err(|e| e.into())
}

pub fn current_langid<'a>() -> Result<&'a LanguageId, Box<dyn Error>> {
    window_utils::get_foreground_process_thread_id()
        .and_then(|tid| current_lcid(tid))
        .and_then(|lcid| {
            (lcid as u32)
                .try_into()
                .map_err(|e: lcid::LcidLookupError| e.into())
        })
}
