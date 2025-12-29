use std::path::{Path, PathBuf};

pub const IGNORE: [&str; 5] = [
    ".git",
    ".nit",
    "playground",
    "examples",
    "target",
];

#[derive(Debug, PartialEq)]
pub struct Directory {
    files: Vec<PathBuf>,
    directories: Vec<Directory>,
}

impl Directory {
    pub fn from_path<T: AsRef<Path>>(path: T) -> Self {
        let dir = std::fs::read_dir(path).expect("Unable to read directory");

        let mut files = Vec::new();
        let mut directories = Vec::new();
        for path in dir {
            let path = path.unwrap().path();
            if IGNORE.iter().any(|i| path.ends_with(i)) {
                continue
            }

            if path.is_dir() {
                let sub_directory = Directory::from_path(path);
                directories.push(sub_directory);
            } else {
                files.push(path);
            }
        }
        Self { files, directories }
    }
}


pub fn get_all_files_flat<T: AsRef<Path>>(path: T) -> Vec<PathBuf> {
    let dir = std::fs::read_dir(path).expect("Unable to read directory");

    let mut files: Vec<PathBuf> = Vec::new();
    for path in dir {
        let path = path.unwrap().path();
        if IGNORE.iter().any(|i| path.ends_with(i)) {
            continue
        }

        if path.is_dir() {
            let sub_files = get_all_files_flat(path);
            files.extend_from_slice(&sub_files);
        } else {
            files.push(path);
        }
    }
    files
}



#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn read_file_in_dir() {
        let files: Vec<String> = get_all_files_flat("./examples")
            .iter()
            .filter_map(|p| p.clone().into_os_string().into_string().ok())
            .collect();

        let expected = vec![
            "./examples/commit",
            "./examples/tree",
            "./examples/commit_tree",
            "./examples/index_with_tree",
            "./examples/index",
            "./examples/blob.c",
        ];
        assert_eq!(files, expected);
    }

    #[test]
    fn read_dir_in_dir() {
        let files: Vec<String> = get_all_files_flat("./playground")
            .iter()
            .filter_map(|p| p.clone().into_os_string().into_string().ok())
            .collect();

        let expected = vec![
            "./playground/sub_dir/sub_file",
            "./playground/main.c"
        ];

        assert_eq!(files, expected);
    }

    #[test]
    fn get_files_as_tree_one_depth() {
        let files = Directory::from_path("./examples");

        let expected = Directory {
            files: vec![
                PathBuf::from("./examples/commit"),
                PathBuf::from("./examples/tree"),
                PathBuf::from("./examples/commit_tree"),
                PathBuf::from("./examples/index_with_tree"),
                PathBuf::from("./examples/index"),
                PathBuf::from("./examples/blob.c"),
            ],
            directories: Vec::new()
        };

        assert_eq!(files, expected);
    }

    #[test]
    fn get_files_as_tree_two_depth() {
        let files = Directory::from_path("./playground");

        let expected = Directory {
            files: vec![
                PathBuf::from("./playground/main.c"),
            ],
            directories: vec![
                Directory {
                    files: vec![
                        PathBuf::from("./playground/sub_dir/sub_file"),
                    ],
                    directories: Vec::new()
                }
            ]
        };

        assert_eq!(files, expected);
    }

    #[test]
    fn get_directory_name_from_entries() {
    }
}
