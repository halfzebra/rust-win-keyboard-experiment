use lcid::LanguageId;
use rust_active_window_test::keyboard_utils;
use rust_active_window_test::window_utils;
use std::convert::TryInto;
use std::error::Error;
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn Error>> {
    let mut last_program: Option<PathBuf> = None;

    loop {
        if let Ok(p) = window_utils::active_window_app_path() {
            match &last_program {
                Some(lp) => if *lp != *p {},
                None => {}
            }

            last_program = Some(p.clone());
            let cur_prog = &p.file_name().unwrap();
            println!("{:?}", cur_prog);
            let cur_lang: &LanguageId = keyboard_utils::current_langid()?;
            let next_l: &LanguageId = "en-US".try_into().unwrap();
            println!("cur_lang {}", cur_lang.name);
            if *cur_prog == "Code.exe" && cur_lang.name != "en-US" {
                keyboard_utils::change_lang(&(next_l.lcid as u16))?;
            }
        }

        std::thread::sleep_ms(1000);
    }

    Ok(())
}
