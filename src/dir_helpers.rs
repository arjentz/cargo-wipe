use num_format::{Locale, ToFormattedString};
use number_prefix::NumberPrefix;
use std::path::PathBuf;
use std::{fs, io};

use crate::command::DirectoryEnum;

#[derive(Debug, Copy, Clone)]
pub struct DirInfo {
    pub dir_count: usize,
    pub file_count: usize,
    pub size: usize,
}

impl DirInfo {
    pub fn new(dir_count: usize, file_count: usize, size: usize) -> Self {
        DirInfo {
            dir_count,
            file_count,
            size,
        }
    }

    pub fn file_count_formatted(&self) -> String {
        self.file_count.to_formatted_string(&Locale::en)
    }

    pub fn size_formatted_mb(&self) -> String {
        let num = self.size / 1024_usize.pow(2);
        num.to_formatted_string(&Locale::en)
    }

    pub fn size_formatted_flex(&self) -> String {
        let np = NumberPrefix::binary(self.size as f64);

        match np {
            NumberPrefix::Prefixed(prefix, n) => format!("{n:.2} {prefix}B"),
            NumberPrefix::Standalone(bytes) => format!("{bytes} bytes"),
        }
    }
}

fn is_valid_target(path: PathBuf, directory: &DirectoryEnum) -> bool {
    if directory == &DirectoryEnum::Target {
        let file_path = path.join(".rustc_info.json");
        return file_path.exists();
    }

    true
}

pub type PathsResult = io::Result<Vec<Result<String, io::Error>>>;

pub fn get_paths_to_delete(path: impl Into<PathBuf>, directory: &DirectoryEnum) -> PathsResult {
    fn walk(dir: io::Result<fs::ReadDir>, directory: &DirectoryEnum) -> PathsResult {
        let mut dir = match dir {
            Ok(dir) => dir,
            Err(e) => {
                return Ok(vec![Err(e)]);
            }
        };

        dir.try_fold(
            Vec::new(),
            |mut acc: Vec<Result<String, io::Error>>, file| {
                let file = file?;

                let size = match file.metadata() {
                    Ok(data) if data.is_dir() => {
                        if file.file_name() == directory.to_string()[..] {
                            if is_valid_target(file.path(), directory) {
                                acc.push(Ok(file.path().display().to_string()));
                            }
                        } else {
                            acc.append(&mut walk(fs::read_dir(file.path()), directory)?);
                        }
                        acc
                    }
                    _ => acc,
                };

                Ok(size)
            },
        )
    }

    walk(fs::read_dir(path.into()), directory)
}

pub fn dir_size(path: impl Into<PathBuf>) -> io::Result<DirInfo> {
    fn walk(dir: io::Result<fs::ReadDir>) -> io::Result<DirInfo> {
        let mut dir = match dir {
            Ok(dir) => dir,
            Err(_) => {
                return Ok(DirInfo::new(0, 0, 0));
            }
        };

        dir.try_fold(DirInfo::new(0, 0, 0), |acc, file| {
            let file = file?;

            let size = match file.metadata() {
                Ok(data) if data.is_dir() => walk(fs::read_dir(file.path()))?,
                Ok(data) => DirInfo::new(1, 1, data.len() as usize),
                _ => DirInfo::new(0, 0, 0),
            };

            Ok(DirInfo::new(
                acc.dir_count + 1,
                acc.file_count + size.file_count,
                acc.size + size.size,
            ))
        })
    }

    walk(fs::read_dir(path.into()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use parameterized::parameterized;

    #[parameterized(
        size = { 0, 512, 1024, 1024_usize.pow(2), 1024_usize.pow(3), 1024_usize.pow(4) },
        output = { "0 bytes", "512 bytes", "1.00 KiB", "1.00 MiB", "1.00 GiB", "1.00 TiB" },
    )]
    fn size_formatted_flex(size: usize, output: &str) {
        let di = DirInfo {
            dir_count: 0,
            file_count: 0,
            size,
        };

        assert_eq!(di.size_formatted_flex(), output);
    }
}
