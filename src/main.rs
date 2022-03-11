use clap::Parser;
use lcid::LanguageId;
use rust_active_window_test::keyboard_utils;
use rust_active_window_test::window_utils;
use std::collections::BTreeMap;
use std::convert::TryInto;
use std::error::Error;
use std::path::PathBuf;
use std::time::Duration;

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Name of the person to greet
    #[clap()]
    path: PathBuf,
}

type Conf = BTreeMap<String, Vec<String>>;

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();
    let config_raw = std::fs::read_to_string(&args.path)?;
    let config: Conf = serde_yaml::from_str(&config_raw)?;
    let mut last_program: Option<PathBuf> = None;

    loop {
        match (
            window_utils::active_window_app_path(),
            keyboard_utils::current_langid(),
        ) {
            (Ok(app_path), Ok(cur_lang)) => {
                println!("{:?} {} {:?}", app_path, cur_lang.name, last_program);

                if path_changed(last_program.as_ref(), &app_path) {
                    if let Some(next_l) = find_app_lang(&app_path, &config) {
                        if cur_lang.lcid != next_l.lcid {
                            keyboard_utils::change_lang(&(next_l.lcid as u16))?;
                        }
                    }

                    last_program = Some(app_path.clone());
                }
            }
            _ => {}
        }

        std::thread::sleep(Duration::from_millis(1000));
    }
}

fn path_changed(prev: Option<&PathBuf>, curr: &PathBuf) -> bool {
    match (prev.and_then(|p| p.to_str()), curr.to_str()) {
        (Some(prev_app_path), Some(cur_app_path)) => prev_app_path != cur_app_path,
        (None, Some(_)) => true,
        _ => false,
    }
}

fn find_app_lang(app_name: &PathBuf, conf: &Conf) -> Option<&'static LanguageId> {
    app_name
        .file_name()
        .and_then(|name| name.to_str())
        .and_then(|cur_app| {
            for (lang_name, program_names) in conf {
                for program in program_names {
                    if program == cur_app {
                        let next_lang_res: Result<&LanguageId, lcid::NameLookupError> =
                            lang_name.as_str().try_into();
                        return next_lang_res.ok();
                    }
                }
            }
            None
        })
}
