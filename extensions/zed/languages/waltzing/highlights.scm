; Waltzing Template Highlights

; Comments
(template_comment) @comment
(html_comment) @comment

; Raw blocks
(raw_block) @string.special

; Keywords - only in proper syntactic contexts
(use_statement "@use" @keyword)
(import_statement "@import" @keyword)
(struct_definition "@struct" @keyword)
(enum_definition "@enum" @keyword)
(function_definition "@fn" @keyword)
(let_statement "@let" @keyword)
(if_statement "@if" @keyword)
(for_loop "@for" @keyword)
(match_statement "@match" @keyword)
(break_statement "@break" @keyword)
(continue_statement "@continue" @keyword)
(attribute_if_statement "@if" @keyword)
(attribute_for_loop "@for" @keyword)
(else_if_branch "else" @keyword)
(else_if_branch "if" @keyword)
(else_branch "else" @keyword)
"in" @keyword
"as" @keyword
"mut" @keyword
"safe" @keyword

; Types
(primitive_type) @type
(struct_definition (identifier) @type)
(enum_definition (identifier) @type)
(generic_params (identifier) @type)
(generic_type (rust_path) @type)

; Functions
(function_definition (identifier) @fn)
(function_path) @fn
(method_call (identifier) @fn)

; Parameters
(parameter (identifier) @variable)

; Properties
(struct_field (identifier) @property)
(field_access (identifier) @property)
(field_pattern (identifier) @property)

; Constructors
(enum_variant (identifier) @constructor)

; Strings
(string_literal) @string
(char_literal) @string
(attribute_value) @string

; Numbers
(integer_literal) @number
(float_literal) @number

; Booleans
(boolean_literal) @boolean

; Operators
(binary_operator) @operator
(unary_operator) @operator
"=>" @operator
"=" @operator

; Punctuation
"{" @punctuation.bracket
"}" @punctuation.bracket
"(" @punctuation.bracket
")" @punctuation.bracket
"[" @punctuation.bracket
"]" @punctuation.bracket
"," @punctuation.delimiter
":" @punctuation.delimiter
";" @punctuation.delimiter

; Tags
(tag_name) @tag
"<@" @tag
"</@" @tag

; Attributes
(attribute_name) @attribute
(attribute_list) @attribute
(attribute (identifier) @attribute)

; Module paths
(use_statement (rust_path) @type)
(import_statement (import_path) @string)
(rust_path) @variable

; Escape
(escape_sequence) @escape
(escape_at) @escape

; Embedded language names
"json" @label
"js" @label
"javascript" @label
"css" @label
"style" @label
"html" @label

; Variables (fallback)
(identifier) @variable
