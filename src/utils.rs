use std;
use std::io::prelude::*;
use copy_dir;

lazy_static! {
    static ref VALID: std::collections::HashMap<String, bool> = {
       let m: std::collections::HashMap<String, bool> = [
           ("yes".to_owned(), true),
           ("y".to_owned(), true),
           ("no".to_owned(), false),
           ("n".to_owned(), false)
       ].iter().cloned().collect();
       m
    };
}

pub fn get_user_input() -> String {
    let stdin = std::io::stdin();
    for line in stdin.lock().lines() {
        return line.unwrap().clone();
    }
    String::new()
}

pub fn ask_user(prompt: String) -> bool {
    loop {
        println!("{}", prompt);
        let choice = get_user_input().to_lowercase();
        if VALID.contains_key(&choice) {
            return *VALID.get(&choice).unwrap();
        } else {
            errln!("Enter a correct choice.");
        }
    }
}

pub fn home_dir() -> String {
    String::from(std::env::var("HOME").unwrap_or_default())
}

pub fn expand_user(path: &std::path::Path) -> std::path::PathBuf {
    let mut path_str = String::from(path.to_str().unwrap_or_default());

    if path_str.len() > 0 && path_str.chars().nth(0).unwrap() == '~' {
        path_str = path_str.replacen("~", &home_dir(), 1);
    }

    std::path::PathBuf::from(&path_str)
}

pub fn is_symbol_link(path: &std::path::Path) -> bool {
    path.read_link().is_ok()
}

pub fn get_absolute_path(
    path: &std::path::Path,
) -> std::result::Result<std::path::PathBuf, String> {
    match path.canonicalize() {
        Ok(path) => Ok(path),
        Err(e) => Err(e.to_string()),
    }
}

pub fn create_empty_file(dry: bool, path: &std::path::Path) -> std::result::Result<(), String> {
    println!("Creating empty file {:?}", path);
    if dry {
        return Ok(());
    }

    match std::fs::OpenOptions::new().write(true).create(true).open(
        path,
    ) {
        Ok(_) => Ok(()),
        Err(e) => Err(e.to_string()),
    }
}

pub fn create_file(
    dry: bool,
    path: &std::path::Path,
    replace: bool,
) -> std::result::Result<(), String> {
    let path = expand_user(path);
    let path = path.as_path();
    let dir_path = path.parent().unwrap();

    if path.exists() {
        if replace || ask_user(format!("{:?} exist, delete it? [Y/n]", path)) {
            return remove_all(dry, path);
        }
    }

    match create_directory(dry, dir_path) {
        Ok(_) => create_empty_file(dry, path),
        Err(e) => Err(e.to_string()),
    }
}

pub fn create_directory(dry: bool, path: &std::path::Path) -> std::result::Result<(), String> {
    let path = expand_user(path);

    println!("Creating {:?}", path);
    if dry {
        return Ok(());
    }

    if !path.is_dir() {
        match std::fs::create_dir_all(path) {
            Ok(_) => Ok(()),
            Err(e) => Err(e.to_string()),
        }
    } else {
        println!("Skipping existing {:?}", path);
        Ok(())
    }
}

pub fn create_symlink(
    dry: bool,
    src: &std::path::Path,
    dest: &std::path::Path,
    replace: bool,
) -> std::result::Result<(), String> {
    let dest = expand_user(dest);
    let src = match get_absolute_path(expand_user(src).as_path()) {
        Ok(src) => src,
        Err(e) => return Err(e.to_string()),
    };

    println!("Linking {:?} -> {:?}", dest.as_os_str(), src.as_os_str());
    if dry {
        return Ok(());
    }

    if is_symbol_link(&dest) || dest.exists() {
        if is_symbol_link(&dest) && dest.read_link().unwrap() == src {
            println!("Skipping existing {:?} -> {:?}", dest, src);
            return Ok(());
        } else if replace || ask_user(format!("{:?} exists, delete it? [Y/n]", dest)) {
            if let Err(e) = remove_all(dry, &dest) {
                return Err(e.to_string());
            }
        } else {
            return Err(format!("Failed to create symbol link: {:?}", dest));
        }
    }

    match std::os::unix::fs::symlink(src, dest) {
        Ok(_) => Ok(()),
        Err(e) => Err(e.to_string()),
    }
}

pub fn remove_all(dry: bool, path: &std::path::Path) -> std::result::Result<(), String> {
    if dry {
        return Ok(());
    }
    if path.is_file() || is_symbol_link(path) {
        match std::fs::remove_file(path.clone()) {
            Ok(_) => Ok(()),
            Err(e) => Err(e.to_string()),
        }
    } else {
        match std::fs::remove_dir_all(path.clone()) {
            Ok(_) => Ok(()),
            Err(e) => Err(e.to_string()),
        }
    }
}

pub fn copy_path(
    dry: bool,
    src: &std::path::Path,
    dest: &std::path::Path,
    replace: bool,
) -> std::result::Result<(), String> {
    let src = match get_absolute_path(expand_user(src).as_path()) {
        Ok(src) => src,
        Err(e) => return Err(e.to_string()),
    };

    let dest = expand_user(dest);

    if dest.exists() {
        if replace || ask_user(format!("{:?} exists, delete it? [Y/n]", dest)) {
            println!("Removing {:?}", dest);
            if remove_all(dry, &dest).is_err() {
                errln!("Failed to remove {:?}", dest);
            }
        } else {
            return Ok(());
        }
    }

    println!("Copying {:?} -> {:?}", src, dest);
    if dry {
        return Ok(());
    }

    if src.is_file() {
        match std::fs::copy(src, dest) {
            Ok(_) => Ok(()),
            Err(e) => Err(e.to_string()),
        }
    } else {
        match copy_dir::copy_dir(src, dest) {
            Ok(_) => Ok(()),
            Err(errs) => Err(errs.to_string()),
        }
    }
}

pub fn run_command(
    command: &Vec<String>,
) -> std::result::Result<std::io::Result<std::process::Child>, String> {

    println!("Executing \"{}\"", command.join(" "));

    if command.is_empty() {
        return Err("No command".to_string());
    } else if command.len() == 1 {
        return Ok(std::process::Command::new(command[0].clone()).spawn());
    }

    match command.split_first() {
        Some((program, args)) => Ok(std::process::Command::new(program).args(args).spawn()),
        None => Err("Failed to execute".to_string()),
    }
}
