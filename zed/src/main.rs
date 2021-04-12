use fs::OpenOptions;
use gpui::PathPromptOptions;
use log::LevelFilter;
use simplelog::SimpleLogger;
use std::{fs, path::PathBuf};
use zed::{
    assets, editor, file_finder, menus,
    settings::{self, Settings},
    watch::Receiver,
    workspace::{self, OpenParams},
};

fn main() {
    init_logger();

    let app = gpui::App::new(assets::Assets).unwrap();
    let (_, settings_rx) = settings::channel(&app.font_cache()).unwrap();
    app.set_menus(menus::menus(settings_rx.clone()));
    app.on_menu_command(move |command, arg, ctx| match command {
        "app:open" => {
            if let Some(paths) = ctx.platform().prompt_for_paths(PathPromptOptions {
                files: true,
                directories: true,
                multiple: true,
            }) {
                ctx.dispatch_global_action(
                    "workspace:open_paths",
                    OpenParams {
                        paths,
                        settings: arg
                            .unwrap()
                            .downcast_ref::<Receiver<Settings>>()
                            .unwrap()
                            .clone(),
                    },
                );
            }
        }
        _ => ctx.dispatch_global_action(command, ()),
    })
    .run(move |ctx| {
        workspace::init(ctx);
        editor::init(ctx);
        file_finder::init(ctx);

        if stdout_is_a_pty() {
            ctx.platform().activate(true);
        }

        let paths = collect_path_args();
        if !paths.is_empty() {
            ctx.dispatch_global_action(
                "workspace:open_paths",
                OpenParams {
                    paths,
                    settings: settings_rx,
                },
            );
        }
    });
}

fn init_logger() {
    let level = LevelFilter::Info;

    if stdout_is_a_pty() {
        SimpleLogger::init(level, Default::default()).expect("could not initialize logger");
    } else {
        let log_dir_path = dirs::home_dir()
            .expect("could not locate home directory for logging")
            .join("Library/Logs/");
        let log_file_path = log_dir_path.join("Zed.log");
        fs::create_dir_all(&log_dir_path).expect("could not create log directory");
        let log_file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(log_file_path)
            .expect("could not open logfile");
        simplelog::WriteLogger::init(level, simplelog::Config::default(), log_file)
            .expect("could not initialize logger");
    }
}

fn stdout_is_a_pty() -> bool {
    unsafe { libc::isatty(libc::STDOUT_FILENO as i32) != 0 }
}

fn collect_path_args() -> Vec<PathBuf> {
    std::env::args()
        .skip(1)
        .filter_map(|arg| match fs::canonicalize(arg) {
            Ok(path) => Some(path),
            Err(error) => {
                log::error!("error parsing path argument: {}", error);
                None
            }
        })
        .collect::<Vec<_>>()
}
