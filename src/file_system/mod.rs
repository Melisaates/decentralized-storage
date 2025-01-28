use sys_info;

pub mod file_operations; 
pub enum FileSystem {
    FAT32,
    ExFat,
    NTFS,
    Ext4,
    APFS,
}

impl FileSystem {
    pub fn max_file_size(&self) -> u128 {
        match self {
            FileSystem::FAT32 => 4 * 1024 * 1024 * 1024, // 4 GB
            FileSystem::ExFat => {
                // 16 EB (using u128 to avoid overflow)
                16 * 1024 * 1024 * 1024 * 1024 * 1024 * 1024 as u128
            }
            FileSystem::NTFS => {
                // 16 EB (using u128 to avoid overflow)
                16 * 1024 * 1024 * 1024 * 1024 * 1024 * 1024 as u128
            }
            FileSystem::Ext4 => 16 * 1024 * 1024 * 1024 * 1024, // 16 TB
            FileSystem::APFS => 8 * 1024 * 1024 * 1024 * 1024 * 1024 * 1024 as u128, // 8 EB
        }
    }

    pub fn detect_file_system() -> Option<FileSystem> {
        if let Ok(os_type) = sys_info::os_type() {
            match os_type.as_str() {
                "Windows" => Some(FileSystem::NTFS),
                "Linux" => Some(FileSystem::Ext4),
                "Darwin" => Some(FileSystem::APFS),
                _ => None,
            }
        } else {
            None
        }
    }
}
