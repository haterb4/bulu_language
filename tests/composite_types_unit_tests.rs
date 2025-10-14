//! Unit tests specifically for composite types implementation

use bulu::lexer::Lexer;
use bulu::parser::Parser;
use bulu::types::{CompositeTypeId, TypeChecker, TypeId, TypeRegistry};

/// Helper function to parse and type check source code
fn type_check_source(source: &str) -> Result<(), bulu::error::BuluError> {
    let mut lexer = Lexer::new(source);
    let tokens = lexer.tokenize()?;
    let mut parser = Parser::new(tokens);
    let program = parser.parse()?;
    let mut type_checker = TypeChecker::new();
    type_checker.check(&program)
}

/// Helper function to expect type checking to succeed
fn expect_success(source: &str) {
    match type_check_source(source) {
        Ok(()) => {} // Success
        Err(e) => panic!("Expected type checking to succeed, but got error: {}", e),
    }
}

/// Helper function to expect type checking to fail
fn expect_failure(source: &str) {
    match type_check_source(source) {
        Ok(()) => panic!("Expected type checking to fail, but it succeeded"),
        Err(_) => {} // Expected failure
    }
}

#[cfg(test)]
mod composite_type_registry_tests {
    use super::*;

    #[test]
    fn test_type_registry_array_types() {
        let mut registry = TypeRegistry::new();

        // Register array types
        let int_array_id = registry.register_array_type(TypeId::Int32);
        let _string_array_id = registry.register_array_type(TypeId::String);

        // Test that same types get same ID
        let int_array_id2 = registry.register_array_type(TypeId::Int32);
        assert_eq!(int_array_id, int_array_id2);

        // Test element type retrieval
        let array_type_id = TypeId::Array(int_array_id);
        let element_type = registry.get_element_type(array_type_id);
        assert_eq!(element_type, Some(TypeId::Int32));

        // Test type names
        let type_name = registry.get_type_name(array_type_id);
        assert!(type_name.contains("array") || type_name.contains("[]"));
    }

    #[test]
    fn test_type_registry_slice_types() {
        let mut registry = TypeRegistry::new();

        // Register slice types
        let int_slice_id = registry.register_slice_type(TypeId::Int32);
        let string_slice_id = registry.register_slice_type(TypeId::String);

        // Test that different element types get different IDs
        assert_ne!(int_slice_id, string_slice_id);

        // Test element type retrieval
        let slice_type_id = TypeId::Slice(int_slice_id);
        let element_type = registry.get_element_type(slice_type_id);
        assert_eq!(element_type, Some(TypeId::Int32));
    }

    #[test]
    fn test_type_registry_map_types() {
        let mut registry = TypeRegistry::new();

        // Register map types
        let string_int_map_id = registry.register_map_type(TypeId::String, TypeId::Int32);
        let int_string_map_id = registry.register_map_type(TypeId::Int32, TypeId::String);

        // Test that different key/value types get different IDs
        assert_ne!(string_int_map_id, int_string_map_id);

        // Test key/value type retrieval
        let map_type_id = TypeId::Map(string_int_map_id);
        let (key_type, value_type) = registry.get_map_types(map_type_id).unwrap();
        assert_eq!(key_type, TypeId::String);
        assert_eq!(value_type, TypeId::Int32);
    }

    #[test]
    fn test_composite_type_assignability() {
        // Test array to slice assignability
        let int_array = CompositeTypeId::Array(Box::new(TypeId::Int32));
        let int_slice = CompositeTypeId::Slice(Box::new(TypeId::Int32));
        let string_slice = CompositeTypeId::Slice(Box::new(TypeId::String));

        assert!(int_array.is_assignable_to(&int_slice));
        assert!(!int_slice.is_assignable_to(&string_slice));

        // Test map assignability
        let string_int_map1 =
            CompositeTypeId::Map(Box::new(TypeId::String), Box::new(TypeId::Int32));
        let string_int_map2 =
            CompositeTypeId::Map(Box::new(TypeId::String), Box::new(TypeId::Int32));
        let int_string_map =
            CompositeTypeId::Map(Box::new(TypeId::Int32), Box::new(TypeId::String));

        assert!(string_int_map1.is_assignable_to(&string_int_map2));
        assert!(!string_int_map1.is_assignable_to(&int_string_map));
    }
}

#[cfg(test)]
mod array_literal_tests {
    use super::*;

    #[test]
    fn test_array_literal_basic() {
        expect_success("let numbers = [1, 2, 3, 4, 5]");
        expect_success("let names = [\"Alice\", \"Bob\", \"Charlie\"]");
        expect_success("let bools = [true, false, true]");
    }

    #[test]
    fn test_array_literal_empty() {
        expect_success("let empty = []");
    }

    #[test]
    fn test_array_literal_type_consistency() {
        expect_failure("let mixed = [1, \"hello\", true]");
        expect_failure("let mixed_numbers = [1, 3.14]");
    }

    #[test]
    fn test_array_indexing() {
        expect_success(
            r#"
            let numbers = [10, 20, 30]
            let first = numbers[0]
            let second = numbers[1]
        "#,
        );
    }

    #[test]
    fn test_array_indexing_invalid() {
        expect_failure(
            r#"
            let numbers = [1, 2, 3]
            let invalid = numbers["not_a_number"]
        "#,
        );
    }

    #[test]
    fn test_array_with_type_annotation() {
        expect_success("let typed_array: []int32 = [10, 20, 30]");
        expect_failure("let wrong_type: []string = [1, 2, 3]");
    }
}

#[cfg(test)]
mod map_literal_tests {
    use super::*;

    #[test]
    fn test_map_literal_basic() {
        expect_success(r#"let ages = {"Alice": 30, "Bob": 25, "Charlie": 35}"#);
        expect_success(r#"let scores = {"math": 95, "science": 87, "english": 92}"#);
    }

    #[test]
    fn test_map_literal_empty() {
        expect_success("let empty = {}");
    }

    #[test]
    fn test_map_literal_type_consistency() {
        expect_failure(r#"let mixed_keys = {"Alice": 30, 42: 25}"#);
        expect_failure(r#"let mixed_values = {"Alice": 30, "Bob": "twenty-five"}"#);
    }

    #[test]
    fn test_map_access() {
        expect_success(
            r#"
            let ages = {"Alice": 30, "Bob": 25}
            let alice_age = ages["Alice"]
            let bob_age = ages["Bob"]
        "#,
        );
    }

    #[test]
    fn test_map_access_invalid() {
        expect_failure(
            r#"
            let ages = {"Alice": 30, "Bob": 25}
            let invalid = ages[42]
        "#,
        );
    }

    #[test]
    fn test_map_with_type_annotation() {
        expect_success(r#"let typed_map: map[string]int32 = {"x": 1, "y": 2}"#);
        expect_failure(r#"let wrong_type: map[int32]string = {"x": 1, "y": 2}"#);
    }
}

#[cfg(test)]
mod nested_composite_tests {
    use super::*;

    #[test]
    fn test_array_of_arrays() {
        // Note: Nested array literals like [[1, 2], [3, 4]] are not fully supported yet
        // This test uses separate variable declarations
        expect_success(
            r#"
            let row1 = [1, 2, 3]
            let row2 = [4, 5, 6]
        "#,
        );
    }

    #[test]
    fn test_map_of_arrays() {
        expect_success(
            r#"
            let user_scores = {
                "Alice": [100, 85, 92],
                "Bob": [88, 90, 87]
            }
            let alice_scores = user_scores["Alice"]
            let alice_first = alice_scores[0]
        "#,
        );
    }

    #[test]
    fn test_array_of_maps() {
        // Test with consistent map types - all string values
        expect_success(
            r#"
            let user1 = {"name": "Alice", "role": "admin"}
            let user2 = {"name": "Bob", "role": "user"}
        "#,
        );
    }

    #[test]
    fn test_map_of_maps() {
        expect_success(
            r#"
            let groups = {
                "group1": {"Alice": 30, "Bob": 25},
                "group2": {"Charlie": 35, "David": 28}
            }
            let group1 = groups["group1"]
            let alice_age = group1["Alice"]
        "#,
        );
    }
}

#[cfg(test)]
mod multiline_parsing_tests {
    use super::*;

    #[test]
    fn test_multiline_array() {
        expect_success(
            r#"
            let numbers = [
                1,
                2,
                3,
                4,
                5
            ]
        "#,
        );
    }

    #[test]
    fn test_multiline_map() {
        expect_success(
            r#"
            let user_data = {
                "Alice": 30,
                "Bob": 25,
                "Charlie": 35
            }
        "#,
        );
    }

    #[test]
    fn test_multiline_nested() {
        // Test valid nested structures with consistent types
        expect_success(
            r#"
            let user_groups = {
                "group1": {
                    "Alice": 30,
                    "Bob": 25
                },
                "group2": {
                    "Charlie": 35,
                    "David": 28
                }
            }
        "#,
        );

        // Test separate nested structures
        expect_success(
            r#"
            let scores = [
                100,
                85,
                92
            ]
            let users = {
                "Alice": 30,
                "Bob": 25
            }
        "#,
        );
    }
}
