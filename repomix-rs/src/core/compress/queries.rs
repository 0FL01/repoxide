//! Tree-sitter queries for extracting code signatures from different languages.
//! Each query captures definitions (functions, classes, interfaces, etc.) that should
//! be included in compressed output.

/// Rust query - captures struct, enum, function, trait, impl, module, macro definitions
pub const QUERY_RUST: &str = r#"
(line_comment) @comment
(block_comment) @comment

; Import statements
(use_declaration) @definition.import

(extern_crate_declaration) @definition.import

; ADT definitions
(struct_item
    name: (type_identifier) @name.definition.class) @definition.class

(enum_item
    name: (type_identifier) @name.definition.class) @definition.class

(union_item
    name: (type_identifier) @name.definition.class) @definition.class

; type aliases
(type_item
    name: (type_identifier) @name.definition.class) @definition.class

; method definitions
(declaration_list
    (function_item
        name: (identifier) @name.definition.method)) @definition.method

; function definitions
(function_item
    name: (identifier) @name.definition.function) @definition.function

; trait definitions
(trait_item
    name: (type_identifier) @name.definition.interface) @definition.interface

; module definitions
(mod_item
    name: (identifier) @name.definition.module) @definition.module

; macro definitions
(macro_definition
    name: (identifier) @name.definition.macro) @definition.macro

; implementations
(impl_item
    trait: (type_identifier)? @name.reference.implementation
    type: (type_identifier) @name.reference.type) @reference.implementation
"#;

/// TypeScript query - captures function, class, interface, type, enum definitions
pub const QUERY_TYPESCRIPT: &str = r#"
(import_statement) @definition.import

(comment) @comment

(function_signature
  name: (identifier) @name.definition.function) @definition.function

(method_signature
  name: (property_identifier) @name.definition.method) @definition.method

(abstract_method_signature
  name: (property_identifier) @name.definition.method) @definition.method

(abstract_class_declaration
  name: (type_identifier) @name.definition.class) @definition.class

(module
  name: (identifier) @name.definition.module) @definition.module

(interface_declaration
  name: (type_identifier) @name.definition.interface) @definition.interface

(function_declaration
  name: (identifier) @name.definition.function) @definition.function

(method_definition
  name: (property_identifier) @name.definition.method) @definition.method

(class_declaration
  name: (type_identifier) @name.definition.class) @definition.class

(type_alias_declaration
  name: (type_identifier) @name.definition.type) @definition.type

(enum_declaration
  name: (identifier) @name.definition.enum) @definition.enum

(lexical_declaration
    (variable_declarator
      name: (identifier) @name.definition.function
      value: (arrow_function)
    )
  ) @definition.function

(variable_declaration
    (variable_declarator
      name: (identifier) @name.definition.function
      value: (arrow_function)
    )
) @definition.function
"#;

/// JavaScript query - captures function, class, method definitions
pub const QUERY_JAVASCRIPT: &str = r#"
(comment) @comment

(method_definition
  name: (property_identifier) @name.definition.method) @definition.method

(class
  name: (_) @name.definition.class) @definition.class

(class_declaration
  name: (_) @name.definition.class) @definition.class

(function_declaration
  name: (identifier) @name.definition.function) @definition.function

(generator_function
  name: (identifier) @name.definition.function) @definition.function

(generator_function_declaration
  name: (identifier) @name.definition.function) @definition.function

(lexical_declaration
  (variable_declarator
    name: (identifier) @name.definition.function
    value: (arrow_function)) @definition.function)

(variable_declaration
  (variable_declarator
    name: (identifier) @name.definition.function
    value: (arrow_function)) @definition.function)

(assignment_expression
  left: (identifier) @name.definition.function
  right: (arrow_function)) @definition.function

(pair
  key: (property_identifier) @name.definition.function
  value: (arrow_function)) @definition.function
"#;

/// Python query - captures class, function definitions
pub const QUERY_PYTHON: &str = r#"
(comment) @comment

(expression_statement
  (string) @comment) @docstring

; Import statements
(import_statement) @definition.import

(import_from_statement) @definition.import

(class_definition
  name: (identifier) @name.definition.class) @definition.class

(function_definition
  name: (identifier) @name.definition.function) @definition.function

(assignment
  left: (identifier) @name.definition.type_alias) @definition.type_alias
"#;

/// Go query - captures function, method, type, struct, interface definitions
pub const QUERY_GO: &str = r#"
(comment) @comment
(package_clause) @definition.package
(import_declaration) @definition.import
(var_declaration) @definition.variable
(const_declaration) @definition.constant

(function_declaration
  name: (identifier) @name) @definition.function

(method_declaration
  name: (field_identifier) @name) @definition.method

(type_spec
  name: (type_identifier) @name) @definition.type

(type_declaration (type_spec name: (type_identifier) @name type: (interface_type))) @definition.interface

(type_declaration (type_spec name: (type_identifier) @name type: (struct_type))) @definition.struct
"#;

/// Java query - captures class, method, interface definitions
pub const QUERY_JAVA: &str = r#"
(line_comment) @comment
(block_comment) @comment

(import_declaration) @definition.import

(package_declaration) @definition.import

(class_declaration
  name: (identifier) @name.definition.class) @definition.class

(method_declaration
  name: (identifier) @name.definition.method) @definition.method

(interface_declaration
  name: (identifier) @name.definition.interface) @definition.interface

(enum_declaration
  name: (identifier) @name.definition.enum) @definition.enum
"#;

/// C query - captures struct, function, typedef definitions
pub const QUERY_C: &str = r#"
(comment) @comment

(struct_specifier name: (type_identifier) @name.definition.class body:(_)) @definition.class

(declaration type: (union_specifier name: (type_identifier) @name.definition.class)) @definition.class

(function_declarator declarator: (identifier) @name.definition.function) @definition.function

(type_definition declarator: (type_identifier) @name.definition.type) @definition.type

(enum_specifier name: (type_identifier) @name.definition.type) @definition.type

(preproc_include) @definition.import
"#;

/// C++ query - captures class, struct, function, method definitions
pub const QUERY_CPP: &str = r#"
(comment) @comment

(struct_specifier name: (type_identifier) @name.definition.class body:(_)) @definition.class

(declaration type: (union_specifier name: (type_identifier) @name.definition.class)) @definition.class

(function_declarator declarator: (identifier) @name.definition.function) @definition.function

(function_declarator declarator: (field_identifier) @name.definition.function) @definition.function

(type_definition declarator: (type_identifier) @name.definition.type) @definition.type

(enum_specifier name: (type_identifier) @name.definition.type) @definition.type

(class_specifier name: (type_identifier) @name.definition.class) @definition.class

(preproc_include) @definition.import
"#;

/// C# query - captures class, method, interface, namespace definitions
pub const QUERY_CSHARP: &str = r#"
(comment) @comment

(class_declaration
  name: (identifier) @name.definition.class
) @definition.class

(interface_declaration
  name: (identifier) @name.definition.interface
) @definition.interface

(method_declaration
  name: (identifier) @name.definition.method
) @definition.method

(namespace_declaration
  name: (identifier) @name.definition.module
) @definition.module

(struct_declaration
  name: (identifier) @name.definition.class
) @definition.class

(enum_declaration
  name: (identifier) @name.definition.enum
) @definition.enum

(using_directive) @definition.import
"#;

/// Ruby query - captures class, module, method definitions
pub const QUERY_RUBY: &str = r#"
(comment) @comment

; Method definitions
(method
  name: (_) @name.definition.method) @definition.method

(singleton_method
  name: (_) @name.definition.method) @definition.method

(alias
  name: (_) @name.definition.method) @definition.method

; Class definitions
(class
  name: (constant) @name.definition.class) @definition.class

(singleton_class
  value: (constant) @name.definition.class) @definition.class

; Module definitions
(module
  name: (constant) @name.definition.module) @definition.module

; Require statements
(call
  method: (identifier) @method
  (#match? @method "^require")) @definition.import
"#;

/// PHP query - captures class, function, method, interface, trait definitions
pub const QUERY_PHP: &str = r#"
(comment) @comment
(namespace_use_clause) @definition.import
(enum_declaration name: (name) @name) @definition.enum

(namespace_definition
  name: (namespace_name) @name) @definition.module

(interface_declaration
  name: (name) @name) @definition.interface

(trait_declaration
  name: (name) @name) @definition.interface

(class_declaration
  name: (name) @name) @definition.class

(function_definition
  name: (name) @name) @definition.function

(method_declaration
  name: (name) @name) @definition.function
"#;

/// CSS query - captures rule_set and at_rule definitions
pub const QUERY_CSS: &str = r#"
(comment) @comment

(rule_set
  (selectors) @name.definition.selector
) @definition.selector

(at_rule) @definition.at_rule
"#;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_queries_not_empty() {
        assert!(!QUERY_RUST.is_empty());
        assert!(!QUERY_TYPESCRIPT.is_empty());
        assert!(!QUERY_JAVASCRIPT.is_empty());
        assert!(!QUERY_PYTHON.is_empty());
        assert!(!QUERY_GO.is_empty());
        assert!(!QUERY_JAVA.is_empty());
        assert!(!QUERY_C.is_empty());
        assert!(!QUERY_CPP.is_empty());
        assert!(!QUERY_CSHARP.is_empty());
        assert!(!QUERY_RUBY.is_empty());
        assert!(!QUERY_PHP.is_empty());
        assert!(!QUERY_CSS.is_empty());
    }
}
