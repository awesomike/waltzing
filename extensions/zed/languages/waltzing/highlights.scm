; Waltzing Template Highlights

; Comments
(template_comment) @comment
(html_comment) @comment

; Raw blocks
(raw_block) @string.special

; Keywords - only in proper syntactic contexts
(use_statement "@" @keyword "use" @keyword)
(import_statement "@" @keyword "import" @keyword)
(struct_definition "@" @keyword "struct" @keyword)
(enum_definition "@" @keyword "enum" @keyword)
(function_definition "@" @keyword "fn" @keyword)
(let_statement "@" @keyword "let" @keyword)
(if_statement "@" @keyword "if" @keyword)
(for_loop "@" @keyword "for" @keyword)
(match_statement "@" @keyword "match" @keyword)
(break_statement "@" @keyword "break" @keyword)
(continue_statement "@" @keyword "continue" @keyword)
(attribute_if_statement "@" @keyword "if" @keyword)
(attribute_for_loop "@" @keyword "for" @keyword)
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

; Parameters
(parameter (identifier) @variable)

; Properties
(struct_field (identifier) @property)
(field_pattern (identifier) @property)
(struct_field_init (identifier) @property)

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
(rust_path) @variable

; Escape
(escape_sequence) @escape
(escape_at) @escape

; Embedded language names
"json" @label
"alpine" @label
"js" @label
"javascript" @label
"css" @label
"style" @label
"html" @label

; Variables (fallback)
(identifier) @variable
