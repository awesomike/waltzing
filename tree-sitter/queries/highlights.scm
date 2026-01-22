; Waltzing Template Highlights

; Keywords
"@use" @keyword.import
"@import" @keyword.import
"@struct" @keyword.type
"@enum" @keyword.type
"@func" @keyword.function
"@if" @keyword.conditional
"@for" @keyword.repeat
"@match" @keyword.conditional
"@break" @keyword.control
"@continue" @keyword.control
"@safe" @keyword.function
"else" @keyword.conditional
"if" @keyword.conditional
"let" @keyword
"in" @keyword
"as" @keyword
"mut" @keyword

; Operators
(binary_operator) @operator
(unary_operator) @operator
"=>" @operator
"=" @operator
"::" @punctuation.delimiter

; Punctuation
"{" @punctuation.bracket
"}" @punctuation.bracket
"(" @punctuation.bracket
")" @punctuation.bracket
"[" @punctuation.bracket
"]" @punctuation.bracket
"<" @punctuation.bracket
">" @punctuation.bracket
"," @punctuation.delimiter
":" @punctuation.delimiter
";" @punctuation.delimiter

; Literals
(string_literal) @string
(char_literal) @character
(number_literal) @number
(integer_literal) @number
(float_literal) @number
(boolean_literal) @boolean
(escape_sequence) @string.escape

; Types
(primitive_type) @type.builtin
(rust_type) @type
(generic_params (identifier) @type)
(generic_type (rust_path) @type)

; Struct/Enum definitions
(struct_definition (identifier) @type.definition)
(enum_definition (identifier) @type.definition)
(struct_field (identifier) @property)
(enum_variant (identifier) @constructor)

; Functions
(function_definition (identifier) @function.definition)
(parameter (identifier) @variable.parameter)

; Function tags (component calls)
(function_path) @function
(self_closing_function_tag) @tag
(container_function_tag) @tag
"<@" @tag.delimiter
"</@" @tag.delimiter

; HTML elements
(tag_name) @tag
(html_element) @tag
(attribute_name) @attribute
(attribute_value) @string

; Paths and identifiers
(rust_path) @module
(use_statement (rust_path) @module)
(import_statement (string_literal) @string.special)

; Expressions
(simple_expression "@" @punctuation.special)
(complex_expression "@" @punctuation.special)
(expression_path (identifier) @variable)
(field_access (identifier) @property)
(method_call (identifier) @function.method)

; Patterns
(wildcard_pattern) @variable.builtin
(identifier_pattern (identifier) @variable)
(struct_pattern (rust_path) @type)
(field_pattern (identifier) @property)

; Control flow
(if_statement) @conditional
(for_loop) @repeat
(match_statement) @conditional
(match_arm) @conditional

; Comments
(template_comment) @comment
(html_comment) @comment

; Attributes (derive, etc.)
(attribute_list) @attribute
(attribute (identifier) @attribute)

; Escape sequence
(escape_at) @string.escape

; Embedded languages
(embedded_language) @embedded
(language_name) @label

; Text content
(text_content) @text
