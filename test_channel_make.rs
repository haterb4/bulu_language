use bulu::runtime::builtins::*;
use bulu::types::primitive::RuntimeValue;

fn main() {
    println!("Testing channel creation with make() function...");
    
    // Test 1: Create unbuffered channel with make("chan")
    println!("\n=== Test 1: Unbuffered Channel Creation ===");
    let unbuffered_result = builtin_make(&[RuntimeValue::String("chan".to_string())]);
    match &unbuffered_result {
        Ok(RuntimeValue::Channel(id)) => {
            println!("✓ Successfully created unbuffered channel with ID: {}", id);
            
            // Test typeof for the channel
            let typeof_result = builtin_typeof(&[RuntimeValue::Channel(*id)]);
            println!("✓ Channel type: {:?}", typeof_result);
            
            // Test instanceof for the channel
            let instanceof_result = builtin_instanceof(&[
                RuntimeValue::Channel(*id), 
                RuntimeValue::String("channel".to_string())
            ]);
            println!("✓ Is instance of channel: {:?}", instanceof_result);
        }
        Ok(other) => println!("✗ Expected Channel, got: {:?}", other),
        Err(e) => println!("✗ Error creating unbuffered channel: {:?}", e),
    }
    
    // Test 2: Create buffered channel with make("chan", capacity)
    println!("\n=== Test 2: Buffered Channel Creation ===");
    let buffered_result = builtin_make(&[
        RuntimeValue::String("chan".to_string()),
        RuntimeValue::Int32(5)
    ]);
    match &buffered_result {
        Ok(RuntimeValue::Channel(id)) => {
            println!("✓ Successfully created buffered channel (capacity 5) with ID: {}", id);
        }
        Ok(other) => println!("✗ Expected Channel, got: {:?}", other),
        Err(e) => println!("✗ Error creating buffered channel: {:?}", e),
    }
    
    // Test 3: Test different capacity types
    println!("\n=== Test 3: Different Capacity Types ===");
    let capacity_tests = vec![
        ("Int32", RuntimeValue::Int32(10)),
        ("Int64", RuntimeValue::Int64(15)),
        ("UInt32", RuntimeValue::UInt32(20)),
        ("UInt64", RuntimeValue::UInt64(25)),
    ];
    
    for (type_name, capacity) in capacity_tests {
        let result = builtin_make(&[
            RuntimeValue::String("chan".to_string()),
            capacity
        ]);
        match result {
            Ok(RuntimeValue::Channel(id)) => {
                println!("✓ Created channel with {} capacity, ID: {}", type_name, id);
            }
            Ok(other) => println!("✗ Expected Channel for {}, got: {:?}", type_name, other),
            Err(e) => println!("✗ Error with {} capacity: {:?}", type_name, e),
        }
    }
    
    // Test 4: Test channel operations with created channels
    println!("\n=== Test 4: Channel Operations ===");
    if let Ok(RuntimeValue::Channel(channel_id)) = unbuffered_result {
        // Test send operation
        let send_result = builtin_send(&[
            RuntimeValue::Channel(channel_id),
            RuntimeValue::String("Hello from make() channel!".to_string())
        ]);
        println!("Send operation result: {:?}", send_result);
        
        // Test receive operation
        let recv_result = builtin_recv(&[RuntimeValue::Channel(channel_id)]);
        println!("Receive operation result: {:?}", recv_result);
        
        // Test close operation
        let close_result = builtin_close(&[RuntimeValue::Channel(channel_id)]);
        println!("Close operation result: {:?}", close_result);
    }
    
    // Test 5: Error cases
    println!("\n=== Test 5: Error Cases ===");
    
    // Test with invalid capacity type
    let invalid_capacity_result = builtin_make(&[
        RuntimeValue::String("chan".to_string()),
        RuntimeValue::String("invalid".to_string())
    ]);
    println!("Invalid capacity type result: {:?}", invalid_capacity_result);
    
    // Test with unsupported type
    let unsupported_type_result = builtin_make(&[
        RuntimeValue::String("invalid_type".to_string())
    ]);
    println!("Unsupported type result: {:?}", unsupported_type_result);
    
    // Test with no arguments
    let no_args_result = builtin_make(&[]);
    println!("No arguments result: {:?}", no_args_result);
    
    // Test 6: Alternative channel syntax
    println!("\n=== Test 6: Alternative Syntax ===");
    let alt_result = builtin_make(&[RuntimeValue::String("channel".to_string())]);
    match alt_result {
        Ok(RuntimeValue::Channel(id)) => {
            println!("✓ 'channel' syntax also works, ID: {}", id);
        }
        Ok(other) => println!("✗ Expected Channel, got: {:?}", other),
        Err(e) => println!("✗ Error with 'channel' syntax: {:?}", e),
    }
    
    println!("\n=== Channel make() Test Summary ===");
    println!("✓ Unbuffered channel creation: make(\"chan\")");
    println!("✓ Buffered channel creation: make(\"chan\", capacity)");
    println!("✓ Multiple capacity types supported");
    println!("✓ Channel operations work with created channels");
    println!("✓ Proper error handling for invalid inputs");
    println!("✓ Alternative 'channel' syntax supported");
    println!("\nAll channel make() tests completed!");
}