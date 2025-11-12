// std/flag module - Command-line flag parsing
// Provides a modern interface for parsing command-line arguments similar to Go's flag package

use crate::error::{BuluError, Result};
use crate::types::primitive::RuntimeValue;
use std::collections::HashMap;
use std::sync::Mutex;

// Global storage for flag definitions and parsed values
static FLAG_SET: Mutex<Option<FlagSet>> = Mutex::new(None);

/// Flag set containing all defined flags
#[derive(Debug, Clone)]
pub struct FlagSet {
    flags: HashMap<String, Flag>,
    parsed_values: HashMap<String, RuntimeValue>,
    args: Vec<String>,
    parsed: bool,
}

/// Individual flag definition
#[derive(Debug, Clone)]
pub struct Flag {
    name: String,
    short_name: Option<String>,
    description: String,
    default_value: RuntimeValue,
    flag_type: FlagType,
}

#[derive(Debug, Clone, PartialEq)]
pub enum FlagType {
    String,
    Int,
    Bool,
    Float,
}

impl FlagSet {
    fn new() -> Self {
        Self {
            flags: HashMap::new(),
            parsed_values: HashMap::new(),
            args: Vec::new(),
            parsed: false,
        }
    }

    fn define_flag(&mut self, name: String, short_name: Option<String>, description: String, default_value: RuntimeValue, flag_type: FlagType) {
        let flag = Flag {
            name: name.clone(),
            short_name,
            description,
            default_value: default_value.clone(),
            flag_type,
        };
        self.flags.insert(name.clone(), flag);
        self.parsed_values.insert(name, default_value);
    }

    fn parse(&mut self, args: Vec<String>) -> Result<()> {
        self.args = args.clone();
        let mut i = 0;
        let mut positional_args = Vec::new();

        while i < args.len() {
            let arg = &args[i];

            if arg.starts_with("--") {
                // Long flag: --name=value or --name value
                let flag_name = arg.trim_start_matches("--");
                
                if let Some(eq_pos) = flag_name.find('=') {
                    // --name=value format
                    let (name, value) = flag_name.split_at(eq_pos);
                    let value = &value[1..]; // Skip the '='
                    self.set_flag_value(name, value)?;
                } else {
                    // --name value format or --bool-flag
                    if let Some(flag) = self.flags.get(flag_name) {
                        if flag.flag_type == FlagType::Bool {
                            // Boolean flag without value means true
                            self.parsed_values.insert(flag_name.to_string(), RuntimeValue::Bool(true));
                        } else if i + 1 < args.len() {
                            i += 1;
                            self.set_flag_value(flag_name, &args[i])?;
                        } else {
                            return Err(BuluError::RuntimeError {
                                file: None,
                                message: format!("Flag --{} requires a value", flag_name),
                            });
                        }
                    } else {
                        return Err(BuluError::RuntimeError {
                            file: None,
                            message: format!("Unknown flag: --{}", flag_name),
                        });
                    }
                }
            } else if arg.starts_with('-') && arg.len() > 1 && !arg.starts_with("--") {
                // Short flag: -n value or -n=value
                let flag_char = &arg[1..2];
                
                // Find flag by short name
                let flag_name = self.flags.iter()
                    .find(|(_, f)| f.short_name.as_ref().map(|s| s.as_str()) == Some(flag_char))
                    .map(|(name, _)| name.clone());

                if let Some(name) = flag_name {
                    let flag = self.flags.get(&name).unwrap();
                    
                    if arg.len() > 2 && arg.chars().nth(2) == Some('=') {
                        // -n=value format
                        let value = &arg[3..];
                        self.set_flag_value(&name, value)?;
                    } else if flag.flag_type == FlagType::Bool {
                        // Boolean flag
                        self.parsed_values.insert(name, RuntimeValue::Bool(true));
                    } else if i + 1 < args.len() {
                        i += 1;
                        self.set_flag_value(&name, &args[i])?;
                    } else {
                        return Err(BuluError::RuntimeError {
                            file: None,
                            message: format!("Flag -{} requires a value", flag_char),
                        });
                    }
                } else {
                    return Err(BuluError::RuntimeError {
                        file: None,
                        message: format!("Unknown flag: -{}", flag_char),
                    });
                }
            } else {
                // Positional argument
                positional_args.push(arg.clone());
            }

            i += 1;
        }

        // Store positional args
        self.parsed_values.insert(
            "__positional__".to_string(),
            RuntimeValue::Array(positional_args.into_iter().map(RuntimeValue::String).collect()),
        );

        self.parsed = true;
        Ok(())
    }

    fn set_flag_value(&mut self, name: &str, value: &str) -> Result<()> {
        if let Some(flag) = self.flags.get(name) {
            let parsed_value = match flag.flag_type {
                FlagType::String => RuntimeValue::String(value.to_string()),
                FlagType::Int => {
                    let int_val = value.parse::<i64>().map_err(|_| BuluError::RuntimeError {
                        file: None,
                        message: format!("Invalid integer value for flag {}: {}", name, value),
                    })?;
                    RuntimeValue::Int64(int_val)
                }
                FlagType::Float => {
                    let float_val = value.parse::<f64>().map_err(|_| BuluError::RuntimeError {
                        file: None,
                        message: format!("Invalid float value for flag {}: {}", name, value),
                    })?;
                    RuntimeValue::Float64(float_val)
                }
                FlagType::Bool => {
                    let bool_val = match value.to_lowercase().as_str() {
                        "true" | "1" | "yes" | "y" => true,
                        "false" | "0" | "no" | "n" => false,
                        _ => return Err(BuluError::RuntimeError {
                            file: None,
                            message: format!("Invalid boolean value for flag {}: {}", name, value),
                        }),
                    };
                    RuntimeValue::Bool(bool_val)
                }
            };
            self.parsed_values.insert(name.to_string(), parsed_value);
            Ok(())
        } else {
            Err(BuluError::RuntimeError {
                file: None,
                message: format!("Unknown flag: {}", name),
            })
        }
    }

    fn get_value(&self, name: &str) -> Option<RuntimeValue> {
        self.parsed_values.get(name).cloned()
    }

    fn usage(&self) -> String {
        let mut usage = String::from("Usage:\n");
        
        let mut flags: Vec<_> = self.flags.values().collect();
        flags.sort_by_key(|f| &f.name);
        
        for flag in flags {
            let short = flag.short_name.as_ref().map(|s| format!("-{}, ", s)).unwrap_or_default();
            let type_hint = match flag.flag_type {
                FlagType::String => " <string>",
                FlagType::Int => " <int>",
                FlagType::Float => " <float>",
                FlagType::Bool => "",
            };
            let default = format!("{:?}", flag.default_value);
            usage.push_str(&format!("  {}--{}{}\n      {} (default: {})\n", 
                short, flag.name, type_hint, flag.description, default));
        }
        
        usage
    }
}

/// Initialize the flag set
fn get_or_init_flag_set() -> &'static Mutex<Option<FlagSet>> {
    &FLAG_SET
}

/// Define a string flag
pub fn string_flag(name: &str, short_name: Option<&str>, default_value: &str, description: &str) -> Result<RuntimeValue> {
    let mut flag_set = get_or_init_flag_set().lock().unwrap();
    if flag_set.is_none() {
        *flag_set = Some(FlagSet::new());
    }
    
    if let Some(fs) = flag_set.as_mut() {
        fs.define_flag(
            name.to_string(),
            short_name.map(|s| s.to_string()),
            description.to_string(),
            RuntimeValue::String(default_value.to_string()),
            FlagType::String,
        );
    }
    
    Ok(RuntimeValue::Null)
}

/// Define an Int8 flag
pub fn int8_flag(name: &str, short_name: Option<&str>, default_value: i8, description: &str) -> Result<RuntimeValue> {
    let mut flag_set = get_or_init_flag_set().lock().unwrap();
    if flag_set.is_none() {
        *flag_set = Some(FlagSet::new());
    }
    
    if let Some(fs) = flag_set.as_mut() {
        fs.define_flag(
            name.to_string(),
            short_name.map(|s| s.to_string()),
            description.to_string(),
            RuntimeValue::Int8(default_value),
            FlagType::Int,
        );
    }
    
    Ok(RuntimeValue::Null)
}

/// Define an Int16 flag
pub fn int16_flag(name: &str, short_name: Option<&str>, default_value: i16, description: &str) -> Result<RuntimeValue> {
    let mut flag_set = get_or_init_flag_set().lock().unwrap();
    if flag_set.is_none() {
        *flag_set = Some(FlagSet::new());
    }
    
    if let Some(fs) = flag_set.as_mut() {
        fs.define_flag(
            name.to_string(),
            short_name.map(|s| s.to_string()),
            description.to_string(),
            RuntimeValue::Int16(default_value),
            FlagType::Int,
        );
    }
    
    Ok(RuntimeValue::Null)
}

/// Define an Int32 flag
pub fn int32_flag(name: &str, short_name: Option<&str>, default_value: i32, description: &str) -> Result<RuntimeValue> {
    let mut flag_set = get_or_init_flag_set().lock().unwrap();
    if flag_set.is_none() {
        *flag_set = Some(FlagSet::new());
    }
    
    if let Some(fs) = flag_set.as_mut() {
        fs.define_flag(
            name.to_string(),
            short_name.map(|s| s.to_string()),
            description.to_string(),
            RuntimeValue::Int32(default_value),
            FlagType::Int,
        );
    }
    
    Ok(RuntimeValue::Null)
}

/// Define an Int64 flag
pub fn int64_flag(name: &str, short_name: Option<&str>, default_value: i64, description: &str) -> Result<RuntimeValue> {
    let mut flag_set = get_or_init_flag_set().lock().unwrap();
    if flag_set.is_none() {
        *flag_set = Some(FlagSet::new());
    }
    
    if let Some(fs) = flag_set.as_mut() {
        fs.define_flag(
            name.to_string(),
            short_name.map(|s| s.to_string()),
            description.to_string(),
            RuntimeValue::Int64(default_value),
            FlagType::Int,
        );
    }
    
    Ok(RuntimeValue::Null)
}

/// Define a UInt8 flag
pub fn uint8_flag(name: &str, short_name: Option<&str>, default_value: u8, description: &str) -> Result<RuntimeValue> {
    let mut flag_set = get_or_init_flag_set().lock().unwrap();
    if flag_set.is_none() {
        *flag_set = Some(FlagSet::new());
    }
    
    if let Some(fs) = flag_set.as_mut() {
        fs.define_flag(
            name.to_string(),
            short_name.map(|s| s.to_string()),
            description.to_string(),
            RuntimeValue::UInt8(default_value),
            FlagType::Int,
        );
    }
    
    Ok(RuntimeValue::Null)
}

/// Define a UInt16 flag
pub fn uint16_flag(name: &str, short_name: Option<&str>, default_value: u16, description: &str) -> Result<RuntimeValue> {
    let mut flag_set = get_or_init_flag_set().lock().unwrap();
    if flag_set.is_none() {
        *flag_set = Some(FlagSet::new());
    }
    
    if let Some(fs) = flag_set.as_mut() {
        fs.define_flag(
            name.to_string(),
            short_name.map(|s| s.to_string()),
            description.to_string(),
            RuntimeValue::UInt16(default_value),
            FlagType::Int,
        );
    }
    
    Ok(RuntimeValue::Null)
}

/// Define a UInt32 flag
pub fn uint32_flag(name: &str, short_name: Option<&str>, default_value: u32, description: &str) -> Result<RuntimeValue> {
    let mut flag_set = get_or_init_flag_set().lock().unwrap();
    if flag_set.is_none() {
        *flag_set = Some(FlagSet::new());
    }
    
    if let Some(fs) = flag_set.as_mut() {
        fs.define_flag(
            name.to_string(),
            short_name.map(|s| s.to_string()),
            description.to_string(),
            RuntimeValue::UInt32(default_value),
            FlagType::Int,
        );
    }
    
    Ok(RuntimeValue::Null)
}

/// Define a UInt64 flag
pub fn uint64_flag(name: &str, short_name: Option<&str>, default_value: u64, description: &str) -> Result<RuntimeValue> {
    let mut flag_set = get_or_init_flag_set().lock().unwrap();
    if flag_set.is_none() {
        *flag_set = Some(FlagSet::new());
    }
    
    if let Some(fs) = flag_set.as_mut() {
        fs.define_flag(
            name.to_string(),
            short_name.map(|s| s.to_string()),
            description.to_string(),
            RuntimeValue::UInt64(default_value),
            FlagType::Int,
        );
    }
    
    Ok(RuntimeValue::Null)
}

/// Define a Byte flag (alias for UInt8)
pub fn byte_flag(name: &str, short_name: Option<&str>, default_value: u8, description: &str) -> Result<RuntimeValue> {
    uint8_flag(name, short_name, default_value, description)
}

/// Define a boolean flag
pub fn bool_flag(name: &str, short_name: Option<&str>, default_value: bool, description: &str) -> Result<RuntimeValue> {
    let mut flag_set = get_or_init_flag_set().lock().unwrap();
    if flag_set.is_none() {
        *flag_set = Some(FlagSet::new());
    }
    
    if let Some(fs) = flag_set.as_mut() {
        fs.define_flag(
            name.to_string(),
            short_name.map(|s| s.to_string()),
            description.to_string(),
            RuntimeValue::Bool(default_value),
            FlagType::Bool,
        );
    }
    
    Ok(RuntimeValue::Null)
}

/// Define a Float32 flag
pub fn float32_flag(name: &str, short_name: Option<&str>, default_value: f32, description: &str) -> Result<RuntimeValue> {
    let mut flag_set = get_or_init_flag_set().lock().unwrap();
    if flag_set.is_none() {
        *flag_set = Some(FlagSet::new());
    }
    
    if let Some(fs) = flag_set.as_mut() {
        fs.define_flag(
            name.to_string(),
            short_name.map(|s| s.to_string()),
            description.to_string(),
            RuntimeValue::Float32(default_value),
            FlagType::Float,
        );
    }
    
    Ok(RuntimeValue::Null)
}

/// Define a Float64 flag
pub fn float64_flag(name: &str, short_name: Option<&str>, default_value: f64, description: &str) -> Result<RuntimeValue> {
    let mut flag_set = get_or_init_flag_set().lock().unwrap();
    if flag_set.is_none() {
        *flag_set = Some(FlagSet::new());
    }
    
    if let Some(fs) = flag_set.as_mut() {
        fs.define_flag(
            name.to_string(),
            short_name.map(|s| s.to_string()),
            description.to_string(),
            RuntimeValue::Float64(default_value),
            FlagType::Float,
        );
    }
    
    Ok(RuntimeValue::Null)
}

/// Parse command-line arguments
pub fn parse(args: Vec<String>) -> Result<RuntimeValue> {
    let mut flag_set = get_or_init_flag_set().lock().unwrap();
    if flag_set.is_none() {
        *flag_set = Some(FlagSet::new());
    }
    
    if let Some(fs) = flag_set.as_mut() {
        fs.parse(args)?;
    }
    
    Ok(RuntimeValue::Null)
}

/// Get the value of a flag
pub fn get(name: &str) -> Result<RuntimeValue> {
    let flag_set = get_or_init_flag_set().lock().unwrap();
    
    if let Some(fs) = flag_set.as_ref() {
        if !fs.parsed {
            return Err(BuluError::RuntimeError {
                file: None,
                message: "Flags have not been parsed yet. Call flag.parse() first.".to_string(),
            });
        }
        
        fs.get_value(name).ok_or_else(|| BuluError::RuntimeError {
            file: None,
            message: format!("Flag '{}' not defined", name),
        })
    } else {
        Err(BuluError::RuntimeError {
            file: None,
            message: "Flag set not initialized".to_string(),
        })
    }
}

/// Get positional arguments (non-flag arguments)
pub fn args() -> Result<RuntimeValue> {
    let flag_set = get_or_init_flag_set().lock().unwrap();
    
    if let Some(fs) = flag_set.as_ref() {
        if !fs.parsed {
            return Err(BuluError::RuntimeError {
                file: None,
                message: "Flags have not been parsed yet. Call flag.parse() first.".to_string(),
            });
        }
        
        fs.get_value("__positional__").ok_or_else(|| BuluError::RuntimeError {
            file: None,
            message: "No positional arguments found".to_string(),
        })
    } else {
        Ok(RuntimeValue::Array(Vec::new()))
    }
}

/// Print usage information
pub fn usage() -> Result<RuntimeValue> {
    let flag_set = get_or_init_flag_set().lock().unwrap();
    
    if let Some(fs) = flag_set.as_ref() {
        Ok(RuntimeValue::String(fs.usage()))
    } else {
        Ok(RuntimeValue::String("No flags defined\n".to_string()))
    }
}

/// Reset the flag set (useful for testing)
pub fn reset() -> Result<RuntimeValue> {
    let mut flag_set = get_or_init_flag_set().lock().unwrap();
    *flag_set = Some(FlagSet::new());
    Ok(RuntimeValue::Null)
}
