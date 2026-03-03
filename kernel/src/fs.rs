// Basic Virtual File System layer
//
// Provides a simple in-memory file system for m5rOS

use core::fmt;

/// Maximum number of files
pub const MAX_FILES: usize = 64;

/// Maximum file name length
pub const MAX_FILENAME: usize = 32;

/// Maximum file size (32KB per file)
pub const MAX_FILE_SIZE: usize = 32 * 1024;

/// File entry
#[derive(Clone, Copy)]
pub struct FileEntry {
    pub name: [u8; MAX_FILENAME],
    pub name_len: usize,
    pub size: usize,
    pub is_directory: bool,
    pub in_use: bool,
    pub data_offset: usize,
}

impl FileEntry {
    pub const fn new() -> Self {
        FileEntry {
            name: [0; MAX_FILENAME],
            name_len: 0,
            size: 0,
            is_directory: false,
            in_use: false,
            data_offset: 0,
        }
    }

    pub fn name_as_str(&self) -> &str {
        // SAFETY: We only store valid UTF-8
        unsafe { core::str::from_utf8_unchecked(&self.name[..self.name_len]) }
    }

    pub fn set_name(&mut self, name: &str) {
        self.name_len = 0;
        for &b in name.as_bytes() {
            if self.name_len < MAX_FILENAME {
                self.name[self.name_len] = b;
                self.name_len += 1;
            }
        }
    }
}

/// Simple in-memory filesystem
pub struct SimpleFS {
    files: [FileEntry; MAX_FILES],
    file_count: usize,
    data: [u8; MAX_FILES * MAX_FILE_SIZE],
    data_used: usize,
}

impl SimpleFS {
    pub const fn new() -> Self {
        SimpleFS {
            files: [FileEntry::new(); MAX_FILES],
            file_count: 0,
            data: [0; MAX_FILES * MAX_FILE_SIZE],
            data_used: 0,
        }
    }

    /// Initialize filesystem with root directory
    pub fn init(&mut self) {
        // Create root directory
        let mut root = FileEntry::new();
        root.set_name("/");
        root.is_directory = true;
        root.in_use = true;
        self.files[0] = root;
        self.file_count = 1;
    }

    /// Create a new file
    pub fn create_file(&mut self, name: &str) -> Result<usize, &'static str> {
        if self.file_count >= MAX_FILES {
            return Err("Maximum file limit reached");
        }

        // Check if file already exists
        if self.find_file(name).is_some() {
            return Err("File already exists");
        }

        let idx = self.file_count;
        let mut file = FileEntry::new();
        file.set_name(name);
        file.in_use = true;
        file.is_directory = false;
        file.data_offset = self.data_used;
        file.size = 0;

        self.files[idx] = file;
        self.file_count += 1;

        Ok(idx)
    }

    /// Create a new directory
    pub fn create_dir(&mut self, name: &str) -> Result<usize, &'static str> {
        if self.file_count >= MAX_FILES {
            return Err("Maximum file limit reached");
        }

        // Check if directory already exists
        if self.find_file(name).is_some() {
            return Err("Directory already exists");
        }

        let idx = self.file_count;
        let mut dir = FileEntry::new();
        dir.set_name(name);
        dir.in_use = true;
        dir.is_directory = true;
        dir.data_offset = 0;
        dir.size = 0;

        self.files[idx] = dir;
        self.file_count += 1;

        Ok(idx)
    }

    /// Find a file by name
    pub fn find_file(&self, name: &str) -> Option<usize> {
        for i in 0..self.file_count {
            if self.files[i].in_use && self.files[i].name_as_str() == name {
                return Some(i);
            }
        }
        None
    }

    /// Read file contents
    pub fn read_file(&self, idx: usize) -> Option<&[u8]> {
        if idx >= self.file_count || !self.files[idx].in_use {
            return None;
        }

        let file = &self.files[idx];
        if file.is_directory {
            return None;
        }

        let start = file.data_offset;
        let end = start + file.size;

        Some(&self.data[start..end])
    }

    /// Write to a file
    pub fn write_file(&mut self, idx: usize, data: &[u8]) -> Result<(), &'static str> {
        if idx >= self.file_count || !self.files[idx].in_use {
            return Err("Invalid file index");
        }

        let file = &mut self.files[idx];
        if file.is_directory {
            return Err("Cannot write to directory");
        }

        if data.len() > MAX_FILE_SIZE {
            return Err("File too large");
        }

        // Check if we have space
        let new_data_used = file.data_offset + data.len();
        if new_data_used > self.data.len() {
            return Err("Out of storage space");
        }

        // Copy data
        let start = file.data_offset;
        self.data[start..start + data.len()].copy_from_slice(data);
        file.size = data.len();

        if new_data_used > self.data_used {
            self.data_used = new_data_used;
        }

        Ok(())
    }

    /// List all files
    pub fn list_files(&self) -> &[FileEntry] {
        &self.files[..self.file_count]
    }

    /// Delete a file
    pub fn delete_file(&mut self, idx: usize) -> Result<(), &'static str> {
        if idx >= self.file_count || !self.files[idx].in_use || idx == 0 {
            return Err("Invalid file index or cannot delete root");
        }

        self.files[idx].in_use = false;
        Ok(())
    }
}

/// Global filesystem instance
static mut FILESYSTEM: SimpleFS = SimpleFS::new();

/// Initialize the global filesystem
pub fn init() {
    unsafe {
        FILESYSTEM.init();
    }
}

/// Get a reference to the global filesystem
pub fn get_fs() -> &'static mut SimpleFS {
    unsafe { &mut FILESYSTEM }
}
