use std::env;
use std::fs;
use std::io;
use std::path::PathBuf;
use yansi::Paint;

use crate::command::{Args, FolderNameEnum};
use crate::dir_helpers::{dir_size, get_paths_to_delete, DirInfo};

#[derive(Debug, PartialEq)]
pub struct WipeParams {
    pub wipe: bool,
    pub path: PathBuf,
    pub folder_name: FolderNameEnum,
}

pub fn get_params(args: &Args) -> io::Result<WipeParams> {
    let path = env::current_dir()?;

    Ok(WipeParams {
        folder_name: match args.folder_name {
            FolderNameEnum::Node | FolderNameEnum::NodeModules => FolderNameEnum::NodeModules,
            FolderNameEnum::Rust | FolderNameEnum::Target => FolderNameEnum::Target,
        },
        path,
        wipe: args.wipe,
    })
}

pub fn wipe_folders<W: io::Write>(mut stdout: &mut W, params: &WipeParams) -> io::Result<()> {
    write_header(&mut stdout, &params)?;
    let total = write_content(&mut stdout, &params)?;
    write_footer(&mut stdout, &params, &total)?;

    Ok(())
}

fn write_header<W: io::Write>(stdout: &mut W, params: &WipeParams) -> io::Result<()> {
    if params.wipe {
        write!(stdout, "{}", Paint::red("[WIPING]").bold())?;
    } else {
        write!(stdout, "{}", Paint::green("[DRY RUN]").bold())?;
    }

    writeln!(
        stdout,
        r#" Recursively searching for all "{}" folders in {} ..."#,
        Paint::yellow(&params.folder_name),
        Paint::yellow(params.path.display()),
    )?;

    writeln!(stdout)?;

    writeln!(
        stdout,
        r#"{:>18}{:>18}{:>9}{}"#,
        Paint::default("Files #").bold(),
        Paint::default("Size (MB)").bold(),
        "",
        Paint::default("Path").bold()
    )?;

    stdout.flush()?;

    Ok(())
}

fn write_content<W: io::Write>(stdout: &mut W, params: &WipeParams) -> io::Result<DirInfo> {
    let paths_to_delete = get_paths_to_delete(&params.path, &params.folder_name)?;

    let dir_count = &paths_to_delete.len();
    let mut file_count = 0_usize;
    let mut size = 0_usize;

    for path in paths_to_delete {
        if let Ok(path) = path {
            let dir_info = dir_size(&path);

            if let Ok(dir_info) = dir_info {
                write!(
                    stdout,
                    r#"{:>18}{:>18}{:>9}{}"#,
                    dir_info.file_count_formatted(),
                    dir_info.size_formatted_mb(),
                    "",
                    &path
                )?;

                file_count += dir_info.file_count;
                size += dir_info.size;
            } else {
                write!(stdout, r#"{:>18}{:>18}{:>9}{}"#, "?", "?", "", &path)?;
            }

            if params.wipe {
                let r = fs::remove_dir_all(path);

                if let Err(e) = r {
                    write!(stdout, " {}", Paint::red(e))?;
                }
            }

            writeln!(stdout)?;

            stdout.flush()?;
        }
    }

    Ok(DirInfo {
        dir_count: *dir_count,
        file_count,
        size,
    })
}

fn write_footer<W: io::Write>(
    stdout: &mut W,
    params: &WipeParams,
    total: &DirInfo,
) -> io::Result<()> {
    writeln!(stdout)?;
    writeln!(
        stdout,
        r#"{:>18}{:>18}"#,
        Paint::default("Total Files #").bold(),
        Paint::default("Total Size").bold()
    )?;
    writeln!(
        stdout,
        r#"{:>18}{:>18}"#,
        Paint::default(total.file_count_formatted()),
        Paint::default(total.size_formatted_flex())
    )?;

    stdout.flush()?;

    writeln!(stdout)?;
    if total.dir_count > 0 {
        if !params.wipe {
            writeln!(
                stdout,
                "Run {} to wipe all folders found. {}",
                Paint::red(format!("cargo wipe {} -w", params.folder_name)),
                Paint::red("USE WITH CAUTION!")
            )?;
        } else {
            writeln!(stdout, "{}", Paint::green("All clear!"))?
        }
    } else {
        writeln!(stdout, "{}", Paint::green("Nothing found!"))?
    }

    stdout.flush()?;

    Ok(())
}
