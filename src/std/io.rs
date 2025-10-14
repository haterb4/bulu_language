// std.io module - Input/Output operations
// Requirements: 7.1.1, 16.5.1-16.5.4

use std::io::{self, Write, BufRead, BufReader};
use std::fs::File;
use std::path::Path;

/// Print values to stdout without newline
pub fn print(args: &[String]) {
    let output = args.join(" ");
    print!("{}", output);
    io::stdout().flush().unwrap();
}

/// Print values to stdout with newline
pub fn println(args: &[String]) {
    let output = args.join(" ");
    println!("{}", output);
}

/// Print formatted string to stdout
pub fn printf(format: &str, args: &[String]) -> Result<(), std::fmt::Error> {
    // Simple printf implementation - would need more sophisticated formatting
    let mut result = format.to_string();
    
    for (i, arg) in args.iter().enumerate() {
        let placeholder = format!("{{{}}}", i);
        result = result.replace(&placeholder, arg);
    }
    
    print!("{}", result);
    io::stdout().flush().unwrap();
    Ok(())
}

/// Read a line from stdin with optional prompt
pub fn input(prompt: Option<&str>) -> Result<String, io::Error> {
    if let Some(p) = prompt {
        print!("{}", p);
        io::stdout().flush().unwrap();
    }
    
    let stdin = io::stdin();
    let mut line = String::new();
    stdin.read_line(&mut line)?;
    
    // Remove trailing newline
    if line.ends_with('\n') {
        line.pop();
        if line.ends_with('\r') {
            line.pop();
        }
    }
    
    Ok(line)
}

/// File operations
pub struct FileHandle {
    file: File,
}

impl FileHandle {
    /// Open a file for reading
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self, io::Error> {
        let file = File::open(path)?;
        Ok(FileHandle { file })
    }
    
    /// Create a file for writing
    pub fn create<P: AsRef<Path>>(path: P) -> Result<Self, io::Error> {
        let file = File::create(path)?;
        Ok(FileHandle { file })
    }
    
    /// Read entire file content as string
    pub fn read_to_string(&mut self) -> Result<String, io::Error> {
        let mut content = String::new();
        let mut reader = BufReader::new(&mut self.file);
        
        loop {
            let mut line = String::new();
            match reader.read_line(&mut line) {
                Ok(0) => break, // EOF
                Ok(_) => content.push_str(&line),
                Err(e) => return Err(e),
            }
        }
        
        Ok(content)
    }
    
    /// Read file line by line
    pub fn read_lines(&mut self) -> Result<Vec<String>, io::Error> {
        let reader = BufReader::new(&mut self.file);
        let mut lines = Vec::new();
        
        for line in reader.lines() {
            lines.push(line?);
        }
        
        Ok(lines)
    }
    
    /// Write string to file
    pub fn write_string(&mut self, content: &str) -> Result<(), io::Error> {
        use std::io::Write;
        self.file.write_all(content.as_bytes())?;
        self.file.flush()?;
        Ok(())
    }
    
    /// Write lines to file
    pub fn write_lines(&mut self, lines: &[String]) -> Result<(), io::Error> {
        for line in lines {
            self.write_string(line)?;
            self.write_string("\n")?;
        }
        Ok(())
    }
}

/// Directory operations
pub mod dir {
    use std::fs;
    use std::path::Path;
    
    /// List directory contents
    pub fn list<P: AsRef<Path>>(path: P) -> Result<Vec<String>, std::io::Error> {
        let entries = fs::read_dir(path)?;
        let mut files = Vec::new();
        
        for entry in entries {
            let entry = entry?;
            if let Some(name) = entry.file_name().to_str() {
                files.push(name.to_string());
            }
        }
        
        files.sort();
        Ok(files)
    }
    
    /// Create directory
    pub fn create<P: AsRef<Path>>(path: P) -> Result<(), std::io::Error> {
        fs::create_dir_all(path)
    }
    
    /// Remove directory
    pub fn remove<P: AsRef<Path>>(path: P) -> Result<(), std::io::Error> {
        fs::remove_dir_all(path)
    }
    
    /// Check if path exists
    pub fn exists<P: AsRef<Path>>(path: P) -> bool {
        path.as_ref().exists()
    }
    
    /// Check if path is directory
    pub fn is_dir<P: AsRef<Path>>(path: P) -> bool {
        path.as_ref().is_dir()
    }
    
    /// Check if path is file
    pub fn is_file<P: AsRef<Path>>(path: P) -> bool {
        path.as_ref().is_file()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::io::Write;
    
    #[test]
    fn test_file_operations() {
        let test_file = "test_io.txt";
        let test_content = "Hello, World!\nThis is a test file.";
        
        // Create and write to file
        {
            let mut file = FileHandle::create(test_file).unwrap();
            file.write_string(test_content).unwrap();
        }
        
        // Read from file
        {
            let mut file = FileHandle::open(test_file).unwrap();
            let content = file.read_to_string().unwrap();
            assert_eq!(content, test_content);
        }
        
        // Read lines
        {
            let mut file = FileHandle::open(test_file).unwrap();
            let lines = file.read_lines().unwrap();
            assert_eq!(lines.len(), 2);
            assert_eq!(lines[0], "Hello, World!");
            assert_eq!(lines[1], "This is a test file.");
        }
        
        // Cleanup
        fs::remove_file(test_file).unwrap();
    }
    
    #[test]
    fn test_directory_operations() {
        let test_dir = "test_dir";
        
        // Create directory
        dir::create(test_dir).unwrap();
        assert!(dir::exists(test_dir));
        assert!(dir::is_dir(test_dir));
        
        // Create test file in directory
        let test_file = format!("{}/test.txt", test_dir);
        fs::write(&test_file, "test content").unwrap();
        
        // List directory contents
        let contents = dir::list(test_dir).unwrap();
        assert_eq!(contents.len(), 1);
        assert_eq!(contents[0], "test.txt");
        
        // Cleanup
        dir::remove(test_dir).unwrap();
        assert!(!dir::exists(test_dir));
    }
}