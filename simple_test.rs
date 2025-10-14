use std::fs;

fn main() {
    println!("Testing complex Bulu example parsing...");
    
    // Read the main.bu file
    let main_path = "complex-bulu-example/src/main.bu";
    let source = match fs::read_to_string(main_path) {
        Ok(content) => content,
        Err(e) => {
            println!("Failed to read {}: {}", main_path, e);
            return;
        }
    };
    
    println!("Source code:");
    println!("{}", source);
    println!("\n{}\n", "=".repeat(50));
    
    // Let's analyze the import statements
    let lines: Vec<&str> = source.lines().collect();
    for (i, line) in lines.iter().enumerate() {
        let trimmed = line.trim();
        if trimmed.starts_with("import") {
            println!("Line {}: Import statement found: {}", i + 1, trimmed);
        }
    }
}