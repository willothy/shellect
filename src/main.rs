use std::{os::unix::process::CommandExt, path::PathBuf, process::Command};

pub struct RGB {
    r: u8,
    g: u8,
    b: u8,
}

pub struct Shell {
    name: String,
    path: PathBuf,
    args: Vec<String>,
    color: RGB,
}

fn main() {
    let shell = Shell {
        name: "bash".to_owned(),
        path: PathBuf::from("/bin/bash"),
        args: vec![],
        color: RGB {
            r: 255,
            g: 255,
            b: 255,
        },
    };

    let mut cmd = Command::new(&shell.path);
    cmd.args(&shell.args);
    cmd.exec();
}
