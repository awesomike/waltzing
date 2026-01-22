/// <reference types="tree-sitter-cli/dsl" />
// @ts-check

module.exports = grammar({
  name: "waltzing",

  extras: ($) => [/\s/],

  conflicts: ($) => [
    [$.expression, $.pattern],
    [$.rust_path, $.expression],
    [$.struct_pattern, $.expression],
    [$.html_element],
    [$.self_closing_function_tag, $.container_function_tag],
  ],

  rules: {
    // Root rule
    template: ($) => repeat($.template_element),

    template_element: ($) =>
      choice(
        $.use_statement,
        $.import_statement,
        $.struct_definition,
        $.enum_definition,
        $.function_definition,
        $.template_node,
      ),

    // Rust imports
    use_statement: ($) =>
      seq("@use", $.rust_path, optional(seq("as", $.identifier))),

    // Use token() to properly handle :: in paths
    rust_path: ($) =>
      token(
        seq(
          /[a-zA-Z_][a-zA-Z0-9_]*/,
          repeat(seq("::", /[a-zA-Z_][a-zA-Z0-9_]*/)),
        ),
      ),

    // Template imports
    import_statement: ($) =>
      seq("@import", $.string_literal, "as", $.identifier),

    // Struct definition
    struct_definition: ($) =>
      seq(
        "@struct",
        optional($.attribute_list),
        $.identifier,
        optional($.generic_params),
        "{",
        repeat($.struct_field),
        "}",
      ),

    struct_field: ($) =>
      seq(
        $.identifier,
        optional($.attribute_list),
        ":",
        $.type_expression,
        optional(","),
      ),

    // Enum definition
    enum_definition: ($) =>
      seq(
        "@enum",
        optional($.attribute_list),
        $.identifier,
        optional($.generic_params),
        "{",
        repeat($.enum_variant),
        "}",
      ),

    enum_variant: ($) =>
      seq(
        $.identifier,
        optional(seq("(", repeat(seq($.type_expression, optional(","))), ")")),
        optional(","),
      ),

    // Attribute list
    attribute_list: ($) =>
      seq("[", optional(seq($.attribute, repeat(seq(",", $.attribute)))), "]"),

    attribute: ($) =>
      seq($.identifier, optional(seq("(", $.attribute_content, ")"))),

    attribute_content: ($) => /[^)]*/,

    generic_params: ($) =>
      seq("<", $.identifier, repeat(seq(",", $.identifier)), ">"),

    type_expression: ($) => $.rust_type,

    // Function definition
    function_definition: ($) =>
      seq("@func", $.identifier, $.parameter_list, $.content_block),

    parameter_list: ($) =>
      seq(
        "(",
        optional(
          seq($.parameter, repeat(seq(",", $.parameter)), optional(",")),
        ),
        ")",
      ),

    parameter: ($) =>
      seq($.identifier, ":", $.rust_type, optional(seq("=", $.default_value))),

    default_value: ($) => $.expression,

    content_block: ($) => seq("{", repeat($.template_node), "}"),

    // Template nodes
    template_node: ($) =>
      choice(
        $.html_element,
        $.function_tag,
        $.template_expression,
        $.template_control_flow,
        $.comment,
        $.embedded_language,
        $.escape_at,
        $.text_content,
      ),

    // HTML elements
    html_element: ($) =>
      choice(
        // Self-closing tag
        seq("<", $.tag_name, repeat($.html_attribute), "/", ">"),
        // Void elements (no closing tag needed)
        seq("<", $.tag_name, repeat($.html_attribute), ">"),
        // Full element with content and closing tag
        seq(
          "<",
          $.tag_name,
          repeat($.html_attribute),
          ">",
          repeat($.template_node),
          "</",
          $.tag_name,
          ">",
        ),
      ),

    tag_name: ($) => /[a-zA-Z][a-zA-Z0-9-]*/,

    html_attribute: ($) =>
      seq($.attribute_name, optional(seq("=", $.attribute_value))),

    attribute_name: ($) => /[a-zA-Z_:@][a-zA-Z0-9_:.-]*/,

    attribute_value: ($) =>
      choice(
        $.string_literal,
        $.template_expression,
        seq("@", "{", $.expression, "}"),
      ),

    // Template expressions
    template_expression: ($) =>
      choice($.simple_expression, $.complex_expression, $.safe_expression),

    simple_expression: ($) => seq("@", $.expression_path),

    complex_expression: ($) => seq("@", "(", $.expression, ")"),

    safe_expression: ($) =>
      seq(
        "@",
        "safe",
        "(",
        $.expression,
        optional(seq(",", $.expression)),
        ")",
      ),

    expression_path: ($) =>
      seq(
        $.identifier,
        repeat(
          choice(
            seq(".", $.identifier),
            seq("[", $.expression, "]"),
            seq("(", optional($.argument_list), ")"),
          ),
        ),
      ),

    // Control flow
    template_control_flow: ($) =>
      choice(
        $.if_statement,
        $.for_loop,
        $.match_statement,
        $.break_statement,
        $.continue_statement,
      ),

    if_statement: ($) =>
      seq(
        "@if",
        optional(seq("let", $.pattern, "=")),
        $.expression,
        $.content_block,
        repeat($.else_if_branch),
        optional($.else_branch),
      ),

    else_if_branch: ($) =>
      seq(
        "else",
        "if",
        optional(seq("let", $.pattern, "=")),
        $.expression,
        $.content_block,
      ),

    else_branch: ($) => seq("else", $.content_block),

    for_loop: ($) =>
      seq(
        "@for",
        optional(seq($.identifier, ":")),
        $.pattern,
        "in",
        $.expression,
        $.content_block,
      ),

    match_statement: ($) =>
      seq("@match", $.expression, "{", repeat($.match_arm), "}"),

    match_arm: ($) =>
      seq(
        $.pattern,
        optional(seq("if", $.expression)),
        "=>",
        $.content_block,
        optional(","),
      ),

    break_statement: ($) =>
      seq("@break", optional(seq(":", $.identifier)), optional(";")),

    continue_statement: ($) =>
      seq("@continue", optional(seq(":", $.identifier)), optional(";")),

    // Function tags
    function_tag: ($) =>
      choice($.self_closing_function_tag, $.container_function_tag),

    self_closing_function_tag: ($) =>
      seq(
        "<@",
        $.function_path,
        repeat($.function_attribute),
        optional("/"),
        ">",
      ),

    container_function_tag: ($) =>
      seq(
        "<@",
        $.function_path,
        repeat($.function_attribute),
        ">",
        repeat($.template_node),
        "</@",
        $.function_path,
        ">",
      ),

    function_path: ($) =>
      token(
        seq(
          /[a-zA-Z_][a-zA-Z0-9_]*/,
          repeat(seq("::", /[a-zA-Z_][a-zA-Z0-9_]*/)),
        ),
      ),

    function_attribute: ($) =>
      choice(
        $.attribute_reference,
        $.named_function_attribute,
        $.boolean_attribute,
      ),

    attribute_reference: ($) => seq("@", optional("&"), $.identifier),

    named_function_attribute: ($) =>
      seq($.identifier, "=", $.function_attribute_value),

    boolean_attribute: ($) => $.identifier,

    function_attribute_value: ($) =>
      choice($.string_literal, seq("@", $.expression_path), $.unquoted_value),

    unquoted_value: ($) => /[^\s>=\/]+/,

    // Patterns
    pattern: ($) =>
      choice(
        $.wildcard_pattern,
        $.tuple_pattern,
        $.struct_pattern,
        $.identifier_pattern,
        $.literal,
      ),

    wildcard_pattern: ($) => "_",

    identifier_pattern: ($) => $.identifier,

    struct_pattern: ($) =>
      seq(
        $.rust_path,
        "{",
        repeat(seq($.field_pattern, optional(","))),
        optional(".."),
        "}",
      ),

    field_pattern: ($) => seq($.identifier, optional(seq(":", $.pattern))),

    tuple_pattern: ($) =>
      seq("(", $.pattern, repeat(seq(",", $.pattern)), optional(","), ")"),

    // Expressions
    expression: ($) =>
      choice($.binary_expression, $.unary_expression, $.primary_expression),

    binary_expression: ($) =>
      prec.left(1, seq($.expression, $.binary_operator, $.expression)),

    unary_expression: ($) => prec(2, seq($.unary_operator, $.expression)),

    primary_expression: ($) =>
      choice(
        $.literal,
        $.rust_path,
        $.method_call,
        $.field_access,
        $.index_access,
        $.parenthesized_expression,
        $.array_literal,
        $.closure_expression,
      ),

    method_call: ($) =>
      prec.left(
        3,
        seq(
          $.primary_expression,
          ".",
          $.identifier,
          "(",
          optional($.argument_list),
          ")",
        ),
      ),

    field_access: ($) =>
      prec.left(3, seq($.primary_expression, ".", $.identifier)),

    index_access: ($) =>
      prec.left(3, seq($.primary_expression, "[", $.expression, "]")),

    parenthesized_expression: ($) => seq("(", $.expression, ")"),

    array_literal: ($) =>
      seq(
        "[",
        optional(
          seq($.expression, repeat(seq(",", $.expression)), optional(",")),
        ),
        "]",
      ),

    closure_expression: ($) =>
      seq(
        "|",
        optional($.closure_params),
        "|",
        choice($.expression, $.content_block),
      ),

    closure_params: ($) => seq($.identifier, repeat(seq(",", $.identifier))),

    argument_list: ($) =>
      seq($.expression, repeat(seq(",", $.expression)), optional(",")),

    // Operators
    binary_operator: ($) =>
      choice(
        "+",
        "-",
        "*",
        "/",
        "%",
        "==",
        "!=",
        "<",
        ">",
        "<=",
        ">=",
        "&&",
        "||",
        "&",
        "|",
        "^",
        "<<",
        ">>",
      ),

    unary_operator: ($) => choice("!", "-", "*", "&"),

    // Literals
    literal: ($) =>
      choice(
        $.string_literal,
        $.char_literal,
        $.number_literal,
        $.boolean_literal,
      ),

    string_literal: ($) =>
      seq('"', repeat(choice(/[^"\\]/, $.escape_sequence)), '"'),

    char_literal: ($) => seq("'", choice(/[^'\\]/, $.escape_sequence), "'"),

    escape_sequence: ($) =>
      /\\[nrt\\'\"0]|\\x[0-9a-fA-F]{2}|\\u\{[0-9a-fA-F]+\}/,

    number_literal: ($) => choice($.integer_literal, $.float_literal),

    integer_literal: ($) =>
      choice(
        /[0-9][0-9_]*(i8|i16|i32|i64|i128|isize|u8|u16|u32|u64|u128|usize)?/,
        /0x[0-9a-fA-F][0-9a-fA-F_]*(i8|i16|i32|i64|i128|isize|u8|u16|u32|u64|u128|usize)?/,
        /0o[0-7][0-7_]*(i8|i16|i32|i64|i128|isize|u8|u16|u32|u64|u128|usize)?/,
        /0b[01][01_]*(i8|i16|i32|i64|i128|isize|u8|u16|u32|u64|u128|usize)?/,
      ),

    float_literal: ($) => /[0-9][0-9_]*\.[0-9_]*([eE][+-]?[0-9]+)?(f32|f64)?/,

    boolean_literal: ($) => choice("true", "false"),

    // Rust types
    rust_type: ($) =>
      choice(
        $.primitive_type,
        $.reference_type,
        $.generic_type,
        $.path_type,
        $.tuple_type,
        $.array_type,
        $.slice_type,
      ),

    primitive_type: ($) =>
      choice(
        "i8",
        "i16",
        "i32",
        "i64",
        "i128",
        "isize",
        "u8",
        "u16",
        "u32",
        "u64",
        "u128",
        "usize",
        "f32",
        "f64",
        "bool",
        "char",
        "str",
        "String",
      ),

    reference_type: ($) => seq("&", optional("mut"), $.rust_type),

    generic_type: ($) =>
      seq($.rust_path, "<", $.rust_type, repeat(seq(",", $.rust_type)), ">"),

    path_type: ($) => $.rust_path,

    tuple_type: ($) =>
      seq(
        "(",
        optional(
          seq($.rust_type, repeat(seq(",", $.rust_type)), optional(",")),
        ),
        ")",
      ),

    array_type: ($) => seq("[", $.rust_type, ";", $.expression, "]"),

    slice_type: ($) => seq("[", $.rust_type, "]"),

    // Comments
    comment: ($) => choice($.template_comment, $.html_comment),

    // Template comment: @* ... *@
    template_comment: ($) => seq("@*", optional($.comment_content), "*@"),

    comment_content: ($) => /([^*]|\*[^@])*/,

    // HTML comment: <!-- ... -->
    html_comment: ($) => seq("<!--", optional($.html_comment_content), "-->"),

    html_comment_content: ($) => /([^-]|-[^-]|--[^>])*/,

    // Embedded language blocks: @```lang ... ```@
    embedded_language: ($) =>
      seq("@```", $.language_name, optional($.embedded_content), "```@"),

    embedded_content: ($) => /([^`]|`[^`]|``[^`])*/,

    language_name: ($) =>
      choice("html", "css", "js", "javascript", "json", "alpine", "style"),

    // Escape sequence for literal @
    escape_at: ($) => "@@",

    // Text content - use negative precedence to prefer other rules
    // Exclude {} to avoid capturing control flow elements
    text_content: ($) => token(prec(-1, /[^<@{}\s][^<@{}]*/)),

    // Identifier
    identifier: ($) => /[a-zA-Z_][a-zA-Z0-9_]*/,
  },
});
