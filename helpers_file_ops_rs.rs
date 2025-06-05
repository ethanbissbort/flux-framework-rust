use crate::error::{FluxError, Result};
use crate::helpers::logging::log_info;
use chrono::Local;
use fs_extra::dir::CopyOptions;
use std::fs::{self, File, OpenOptions};
use std::io::{Read, Write};
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};

/// Create a timestamped backup of a file
pub fn backup_file<P: AsRef<Path>>(file_path: P) -> Result<PathBuf> {
    let file_path = file_path.as_ref();
    
    if !file_path.exists() {
        return Err(FluxError::not_found(format!(
            "File does not exist: {}",
            file_path.display()
        )));
    }
    
    let timestamp = Local::now().format("%Y%m%d_%H%M%S");
    let backup_name = format!(
        "{}.backup_{}",
        file_path.file_name().unwrap().to_string_lossy(),
        timestamp
    );
    
    let backup_path = file_path.parent().unwrap().join(backup_name);
    
    fs::copy(file_path, &backup_path)
        .map_err(|e| FluxError::Io(e))?;
    
    log_info(format!("Backed up {} to {}", file_path.display(), backup_path.display()));
    
    Ok(backup_path)
}

/// Safely write content to a file with backup
pub fn safe_write_file<P: AsRef<Path>>(
    file_path: P,
    content: &str,
    backup: bool,
) -> Result<()> {
    let file_path = file_path.as_ref();
    
    // Backup existing file if requested
    if backup && file_path.exists() {
        backup_file(file_path)?;
    }
    
    // Write to temporary file first
    let temp_path = file_path.with_extension("tmp");
    
    let mut temp_file = File::create(&temp_path)
        .map_err(|e| FluxError::Io(e))?;
    
    temp_file.write_all(content.as_bytes())
        .map_err(|e| FluxError::Io(e))?;
    
    temp_file.sync_all()
        .map_err(|e| FluxError::Io(e))?;
    
    // Move temp file to final location
    fs::rename(&temp_path, file_path)
        .map_err(|e| FluxError::Io(e))?;
    
    log_info(format!("Successfully wrote to {}", file_path.display()));
    
    Ok(())
}

/// Safely append content to a file with backup
pub fn safe_append_file<P: AsRef<Path>>(
    file_path: P,
    content: &str,
    backup: bool,
) -> Result<()> {
    let file_path = file_path.as_ref();
    
    // Backup existing file if requested
    if backup && file_path.exists() {
        backup_file(file_path)?;
    }
    
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(file_path)
        .map_err(|e| FluxError::Io(e))?;
    
    file.write_all(content.as_bytes())
        .map_err(|e| FluxError::Io(e))?;
    
    log_info(format!("Successfully appended to {}", file_path.display()));
    
    Ok(())
}

/// Read file content as string
pub fn read_file_to_string<P: AsRef<Path>>(file_path: P) -> Result<String> {
    let file_path = file_path.as_ref();
    
    fs::read_to_string(file_path)
        .map_err(|e| FluxError::Io(e))
}

/// Check if file exists
pub fn file_exists<P: AsRef<Path>>(file_path: P) -> bool {
    file_path.as_ref().exists()
}

/// Create directory with parents
pub fn create_dir_all<P: AsRef<Path>>(dir_path: P) -> Result<()> {
    let dir_path = dir_path.as_ref();
    
    fs::create_dir_all(dir_path)
        .map_err(|e| FluxError::Io(e))?;
    
    Ok(())
}

/// Copy file with permissions
pub fn copy_file_with_perms<P: AsRef<Path>, Q: AsRef<Path>>(
    src: P,
    dst: Q,
) -> Result<()> {
    let src = src.as_ref();
    let dst = dst.as_ref();
    
    // Copy file
    fs::copy(src, dst)
        .map_err(|e| FluxError::Io(e))?;
    
    // Copy permissions
    let metadata = fs::metadata(src)
        .map_err(|e| FluxError::Io(e))?;
    
    fs::set_permissions(dst, metadata.permissions())
        .map_err(|e| FluxError::Io(e))?;
    
    Ok(())
}

/// Set file permissions (Unix mode)
pub fn set_permissions<P: AsRef<Path>>(file_path: P, mode: u32) -> Result<()> {
    let file_path = file_path.as_ref();
    
    let permissions = fs::Permissions::from_mode(mode);
    
    fs::set_permissions(file_path, permissions)
        .map_err(|e| FluxError::Io(e))?;
    
    Ok(())
}

/// Create temporary file
pub fn create_temp_file(prefix: &str) -> Result<(File, PathBuf)> {
    let temp_dir = std::env::temp_dir();
    let timestamp = Local::now().format("%Y%m%d_%H%M%S");
    let file_name = format!("{}_{}", prefix, timestamp);
    let temp_path = temp_dir.join(file_name);
    
    let file = File::create(&temp_path)
        .map_err(|e| FluxError::Io(e))?;
    
    Ok((file, temp_path))
}

/// Find files matching pattern
pub fn find_files(dir: &Path, pattern: &str) -> Result<Vec<PathBuf>> {
    let glob_pattern = dir.join(pattern).to_string_lossy().to_string();
    
    let mut files = Vec::new();
    
    for entry in glob::glob(&glob_pattern).map_err(|e| FluxError::parse(e.to_string()))? {
        match entry {
            Ok(path) => files.push(path),
            Err(e) => log_info(format!("Error reading file: {}", e)),
        }
    }
    
    Ok(files)
}

/// Calculate file checksum (SHA256)
pub fn file_checksum<P: AsRef<Path>>(file_path: P) -> Result<String> {
    use sha2::{Digest, Sha256};
    
    let mut file = File::open(file_path.as_ref())
        .map_err(|e| FluxError::Io(e))?;
    
    let mut hasher = Sha256::new();
    let mut buffer = [0; 8192];
    
    loop {
        let bytes_read = file.read(&mut buffer)
            .map_err(|e| FluxError::Io(e))?;
        
        if bytes_read == 0 {
            break;
        }
        
        hasher.update(&buffer[..bytes_read]);
    }
    
    Ok(format!("{:x}", hasher.finalize()))
}

/// Get file size
pub fn file_size<P: AsRef<Path>>(file_path: P) -> Result<u64> {
    let metadata = fs::metadata(file_path.as_ref())
        .map_err(|e| FluxError::Io(e))?;
    
    Ok(metadata.len())
}

/// Check if path is a directory
pub fn is_directory<P: AsRef<Path>>(path: P) -> bool {
    path.as_ref().is_dir()
}

/// Remove file or directory
pub fn remove_path<P: AsRef<Path>>(path: P) -> Result<()> {
    let path = path.as_ref();
    
    if path.is_dir() {
        fs::remove_dir_all(path)
            .map_err(|e| FluxError::Io(e))?;
    } else {
        fs::remove_file(path)
            .map_err(|e| FluxError::Io(e))?;
    }
    
    Ok(())
}

/// Copy directory recursively
pub fn copy_dir<P: AsRef<Path>, Q: AsRef<Path>>(src: P, dst: Q) -> Result<()> {
    let src = src.as_ref();
    let dst = dst.as_ref();
    
    let options = CopyOptions::new();
    
    fs_extra::dir::copy(src, dst, &options)
        .map_err(|e| FluxError::Io(std::io::Error::new(std::io::ErrorKind::Other, e)))?;
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    
    #[test]
    fn test_backup_file() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test.txt");
        
        // Create test file
        fs::write(&test_file, "test content").unwrap();
        
        // Backup file
        let backup_path = backup_file(&test_file).unwrap();
        
        // Check backup exists and has same content
        assert!(backup_path.exists());
        assert_eq!(
            fs::read_to_string(&backup_path).unwrap(),
            "test content"
        );
    }
    
    #[test]
    fn test_safe_write_file() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test.txt");
        
        // Write without backup
        safe_write_file(&test_file, "new content", false).unwrap();
        assert_eq!(fs::read_to_string(&test_file).unwrap(), "new content");
        
        // Write with backup
        safe_write_file(&test_file, "updated content", true).unwrap();
        assert_eq!(fs::read_to_string(&test_file).unwrap(), "updated content");
        
        // Check backup was created
        let backups: Vec<_> = fs::read_dir(temp_dir.path())
            .unwrap()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_name().to_string_lossy().contains("backup"))
            .collect();
        
        assert_eq!(backups.len(), 1);
    }
}