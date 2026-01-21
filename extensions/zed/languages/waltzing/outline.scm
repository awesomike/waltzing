; Outline queries for Waltzing templates
; Provides structure navigation in the editor

; Struct definitions
(struct_definition
  (identifier) @name) @item

; Enum definitions
(enum_definition
  (identifier) @name) @item

; Function definitions
(function_definition
  (identifier) @name) @item

; Imports for navigation
(use_statement
  (rust_path) @name) @item

(import_statement
  (string_literal) @name) @item
