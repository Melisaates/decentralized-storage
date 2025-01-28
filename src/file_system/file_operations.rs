use std::fs::{self, File};
use std::io::{self, Write};
use std::path::Path;
use std::process::Command;

pub fn change_file_permission(path: &str) -> io::Result<()> {
    #[cfg(target_os = "windows")]
    {
        let output = Command::new("icacls")
            .arg(path)
            .arg("/grant")
            .arg("Everyone:(F)") // Full access for everyone
            .output()?;

        if !output.status.success() {
            return Err(io::Error::new(io::ErrorKind::Other, "Permission change failed"));
        }
    }

    #[cfg(target_os = "unix")]
    {
        let output = Command::new("chmod")
            .arg("+w")
            .arg(path)
            .output()?;

        if !output.status.success() {
            return Err(io::Error::new(io::ErrorKind::Other, "Permission change failed"));
        }
    }

    Ok(())
}

pub fn control_permission(path: &str) -> bool {
    match fs::metadata(path) {
        Ok(meta) => {
            if meta.permissions().readonly() {
                println!("File is read-only. You can't write to it.");
                false
            } else {
                println!("File is writable. You can write to it.");
                true
            }
        }
        Err(e) => {
            println!("File access error: {}", e);
            false
        }
    }
}

pub fn can_write_to_path(path: &str) -> bool {
    let path = Path::new(path);

    if path.is_dir() {
        let test_file = format!("{}/.test_write_permission", path.display());
        match fs::File::create(&test_file) {
            Ok(_) => {
                // Test file created, remove it immediately
                let _ = fs::remove_file(test_file);
                true
            }
            Err(_) => false,
        }
    } else {
        match fs::OpenOptions::new().write(true).open(path) {
            Ok(_) => true,
            Err(_) => false,
        }
    }
}

