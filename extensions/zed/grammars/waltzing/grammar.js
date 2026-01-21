/// <reference types="tree-sitter-cli/dsl" />
// @ts-check

module.exports = grammar({
  name: "waltzing",

  extras: ($) => [/\s/],

  conflicts: ($) => [
    [$.expression, $.pattern],
    [$.rust_path, $.expression],
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
      seq(seq("@", "use"), $.rust_path, optional(seq("as", $.identifier))),

    // Use token() to properly handle :: in paths
    rust_path: ($) =>
      token(
        seq(
          /[a-zA-Z_][a-zA-Z0-9_]*/,
          repeat(seq("::", /[a-zA-Z_][a-zA-Z0-9_]*/)),
        ),
      ),

    // Template imports - supports both quoted "path" and unquoted /path
    // The "as alias" part is optional
    import_statement: ($) =>
      seq(seq("@", "import"), choice($.string_literal, $.import_path), optional(seq("as", $.identifier))),

    import_path: ($) => /\/[^\s]+/,

    // Struct definition
    struct_definition: ($) =>
      seq(
        seq("@", "struct"),
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
        seq("@", "enum"),
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
      seq(seq("@", "fn"), $.identifier, $.parameter_list, $.content_block),

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
    // Note: template_control_flow must come before template_expression
    // to avoid @for/@if/@let being parsed as variable expressions
    template_node: ($) =>
      choice(
        $.html_element,
        $.function_tag,
        $.template_control_flow,
        $.template_expression,
        $.comment,
        $.raw_block,
        $.embedded_language,
        $.escape_at,
        $.text_content,
      ),

    // HTML elements
    // Note: attribute_or_control allows @if/@for in attribute position
    html_element: ($) =>
      choice(
        // Self-closing tag
        seq("<", $.tag_name, repeat($.attribute_or_control), "/", ">"),
        // Void elements (no closing tag needed)
        seq("<", $.tag_name, repeat($.attribute_or_control), ">"),
        // Full element with content and closing tag
        seq(
          "<",
          $.tag_name,
          repeat($.attribute_or_control),
          ">",
          repeat($.template_node),
          "</",
          $.tag_name,
          ">",
        ),
      ),

    // Allow either HTML attributes or control flow (@if/@for) in attribute position
    attribute_or_control: ($) =>
      choice(
        $.html_attribute,
        $.attribute_control_flow,
      ),

    // Control flow in attribute context - produces attributes conditionally
    attribute_control_flow: ($) =>
      choice(
        $.attribute_if_statement,
        $.attribute_for_loop,
      ),

    attribute_if_statement: ($) =>
      seq(
        seq("@", "if"),
        $.expression,
        "{",
        repeat($.attribute_or_control),
        "}",
        optional(seq("else", "{", repeat($.attribute_or_control), "}")),
      ),

    attribute_for_loop: ($) =>
      seq(
        seq("@", "for"),
        $.simple_pattern,
        "in",
        $.expression,
        "{",
        repeat($.attribute_or_control),
        "}",
      ),

    tag_name: ($) => /[a-zA-Z][a-zA-Z0-9-]*/,

    html_attribute: ($) =>
      seq($.attribute_name, optional(seq("=", $.attribute_value))),

    // Attribute names: allow @ for directives like @click, but not @if/@for/@let
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

    // High-precedence token to match @identifier before @for/@if etc keywords
    simple_expression: ($) =>
      choice(
        token(prec(2, /@[a-zA-Z_][a-zA-Z0-9_]*!/)),  // Macro calls: @format!
        seq("@", $.expression_path),  // Regular expressions: @foo.bar
      ),

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
        optional("!"),  // Rust macro call
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
        $.let_statement,
        $.if_statement,
        $.for_loop,
        $.match_statement,
        $.break_statement,
        $.continue_statement,
      ),

    // Let binding: @let name = expression
    // Use simple_pattern to avoid confusion with if/match expressions containing { }
    let_statement: ($) =>
      seq(seq("@", "let"), $.simple_pattern, "=", $.expression),

    if_statement: ($) =>
      seq(
        seq("@", "if"),
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
        seq("@", "for"),
        optional(seq($.identifier, ":")),
        $.simple_pattern,
        "in",
        $.expression,
        $.content_block,
      ),

    match_statement: ($) =>
      seq(seq("@", "match"), $.expression, "{", repeat($.match_arm), "}"),

    match_arm: ($) =>
      seq(
        $.pattern,
        optional(seq("if", $.expression)),
        "=>",
        $.content_block,
        optional(","),
      ),

    break_statement: ($) =>
      seq(seq("@", "break"), optional(seq(":", $.identifier)), optional(";")),

    continue_statement: ($) =>
      seq(seq("@", "continue"), optional(seq(":", $.identifier)), optional(";")),

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

    // Patterns - full patterns used in match arms
    pattern: ($) =>
      choice(
        $.wildcard_pattern,
        $.tuple_pattern,
        $.struct_pattern,
        $.literal,
        $.identifier_pattern,
      ),

    // Simple pattern for for loops - no struct patterns to avoid ambiguity with content_block
    simple_pattern: ($) =>
      choice(
        $.wildcard_pattern,
        $.tuple_pattern,
        $.literal,
        $.identifier_pattern,
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
        // if/match expressions must come first to match the keywords before rust_path
        $.if_expression,
        $.match_expression,
        $.literal,
        $.macro_call,
        $.method_call,
        $.field_access,
        $.index_access,
        $.parenthesized_expression,
        $.array_literal,
        $.closure_expression,
        // rust_path last as fallback for identifiers
        $.rust_path,
      ),

    // If expression (returns a value): if cond { expr } else { expr }
    if_expression: ($) =>
      prec.right(
        10,
        seq(
          "if",
          $.expression,
          "{",
          $.expression,
          "}",
          "else",
          choice(
            $.if_expression,
            seq("{", $.expression, "}"),
          ),
        ),
      ),

    // Match expression (returns a value): match expr { pat => expr, ... }
    match_expression: ($) =>
      prec.right(
        10,
        seq(
          "match",
          $.expression,
          "{",
          repeat($.expression_match_arm),
          "}",
        ),
      ),

    expression_match_arm: ($) =>
      seq(
        $.pattern,
        optional(seq("if", $.expression)),
        "=>",
        $.expression,
        optional(","),
      ),

    // Rust macro calls: format!(), vec![], println!(), etc.
    macro_call: ($) =>
      prec(
        5,
        seq(
          $.identifier,
          "!",
          choice(
            seq("(", optional($.macro_args), ")"),
            seq("[", optional($.macro_args), "]"),
            seq("{", optional($.macro_args), "}"),
          ),
        ),
      ),

    // Macro arguments
    macro_args: ($) =>
      seq(
        $.macro_arg,
        repeat(seq(",", $.macro_arg)),
        optional(","),
      ),

    // Macro argument - just use expression
    macro_arg: ($) => $.expression,

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

    // Template comments: @* ... *@ with varying asterisk counts
    template_comment: ($) => choice(
      ...Array.from({ length: 22 }, (_, i) => $[`template_comment_${i + 1}`])
    ),
    template_comment_1: ($) => /@\*([^*]|\*[^@])*\*@/,
    template_comment_2: ($) => /@\*\*([^*]|\*[^*]|\*\*[^@])*\*\*@/,
    template_comment_3: ($) => /@\*\*\*([^*]|\*[^*]|\*\*[^*]|\*\*\*[^@])*\*\*\*@/,
    template_comment_4: ($) => /@\*{4}([^*]|\*{1,3}[^*]|\*{4}[^@])*\*{4}@/,
    template_comment_5: ($) => /@\*{5}([^*]|\*{1,4}[^*]|\*{5}[^@])*\*{5}@/,
    template_comment_6: ($) => /@\*{6}([^*]|\*{1,5}[^*]|\*{6}[^@])*\*{6}@/,
    template_comment_7: ($) => /@\*{7}([^*]|\*{1,6}[^*]|\*{7}[^@])*\*{7}@/,
    template_comment_8: ($) => /@\*{8}([^*]|\*{1,7}[^*]|\*{8}[^@])*\*{8}@/,
    template_comment_9: ($) => /@\*{9}([^*]|\*{1,8}[^*]|\*{9}[^@])*\*{9}@/,
    template_comment_10: ($) => /@\*{10}([^*]|\*{1,9}[^*]|\*{10}[^@])*\*{10}@/,
    template_comment_11: ($) => /@\*{11}([^*]|\*{1,10}[^*]|\*{11}[^@])*\*{11}@/,
    template_comment_12: ($) => /@\*{12}([^*]|\*{1,11}[^*]|\*{12}[^@])*\*{12}@/,
    template_comment_13: ($) => /@\*{13}([^*]|\*{1,12}[^*]|\*{13}[^@])*\*{13}@/,
    template_comment_14: ($) => /@\*{14}([^*]|\*{1,13}[^*]|\*{14}[^@])*\*{14}@/,
    template_comment_15: ($) => /@\*{15}([^*]|\*{1,14}[^*]|\*{15}[^@])*\*{15}@/,
    template_comment_16: ($) => /@\*{16}([^*]|\*{1,15}[^*]|\*{16}[^@])*\*{16}@/,
    template_comment_17: ($) => /@\*{17}([^*]|\*{1,16}[^*]|\*{17}[^@])*\*{17}@/,
    template_comment_18: ($) => /@\*{18}([^*]|\*{1,17}[^*]|\*{18}[^@])*\*{18}@/,
    template_comment_19: ($) => /@\*{19}([^*]|\*{1,18}[^*]|\*{19}[^@])*\*{19}@/,
    template_comment_20: ($) => /@\*{20}([^*]|\*{1,19}[^*]|\*{20}[^@])*\*{20}@/,
    template_comment_21: ($) => /@\*{21}([^*]|\*{1,20}[^*]|\*{21}[^@])*\*{21}@/,
    template_comment_22: ($) => /@\*{22}([^*]|\*{1,21}[^*]|\*{22}[^@])*\*{22}@/,

    // HTML comment: <!-- ... -->
    html_comment: ($) => /<!--([^-]|-[^-]|--[^>])*-->/,

    // Raw blocks: @# ... #@ with varying hash counts
    raw_block: ($) => choice(
      ...Array.from({ length: 22 }, (_, i) => $[`raw_block_${i + 1}`])
    ),
    raw_block_1: ($) => /@#([^#]|#[^@])*#@/,
    raw_block_2: ($) => /@##([^#]|#[^#]|##[^@])*##@/,
    raw_block_3: ($) => /@###([^#]|#{1,2}[^#]|###[^@])*###@/,
    raw_block_4: ($) => /@#{4}([^#]|#{1,3}[^#]|#{4}[^@])*#{4}@/,
    raw_block_5: ($) => /@#{5}([^#]|#{1,4}[^#]|#{5}[^@])*#{5}@/,
    raw_block_6: ($) => /@#{6}([^#]|#{1,5}[^#]|#{6}[^@])*#{6}@/,
    raw_block_7: ($) => /@#{7}([^#]|#{1,6}[^#]|#{7}[^@])*#{7}@/,
    raw_block_8: ($) => /@#{8}([^#]|#{1,7}[^#]|#{8}[^@])*#{8}@/,
    raw_block_9: ($) => /@#{9}([^#]|#{1,8}[^#]|#{9}[^@])*#{9}@/,
    raw_block_10: ($) => /@#{10}([^#]|#{1,9}[^#]|#{10}[^@])*#{10}@/,
    raw_block_11: ($) => /@#{11}([^#]|#{1,10}[^#]|#{11}[^@])*#{11}@/,
    raw_block_12: ($) => /@#{12}([^#]|#{1,11}[^#]|#{12}[^@])*#{12}@/,
    raw_block_13: ($) => /@#{13}([^#]|#{1,12}[^#]|#{13}[^@])*#{13}@/,
    raw_block_14: ($) => /@#{14}([^#]|#{1,13}[^#]|#{14}[^@])*#{14}@/,
    raw_block_15: ($) => /@#{15}([^#]|#{1,14}[^#]|#{15}[^@])*#{15}@/,
    raw_block_16: ($) => /@#{16}([^#]|#{1,15}[^#]|#{16}[^@])*#{16}@/,
    raw_block_17: ($) => /@#{17}([^#]|#{1,16}[^#]|#{17}[^@])*#{17}@/,
    raw_block_18: ($) => /@#{18}([^#]|#{1,17}[^#]|#{18}[^@])*#{18}@/,
    raw_block_19: ($) => /@#{19}([^#]|#{1,18}[^#]|#{19}[^@])*#{19}@/,
    raw_block_20: ($) => /@#{20}([^#]|#{1,19}[^#]|#{20}[^@])*#{20}@/,
    raw_block_21: ($) => /@#{21}([^#]|#{1,20}[^#]|#{21}[^@])*#{21}@/,
    raw_block_22: ($) => /@#{22}([^#]|#{1,21}[^#]|#{22}[^@])*#{22}@/,

    // Embedded language blocks: @```lang ... ```@
    embedded_language: ($) =>
      seq("@```", $.language_name, /([^`]|`[^`]|``[^`]|```[^@])*/, "```@"),

    language_name: ($) =>
      choice("html", "css", "js", "javascript", "json", "style"),

    // Escape sequence for literal @
    escape_at: ($) => "@@",

    // Text content - use negative precedence to prefer other rules
    // Exclude {} to avoid capturing control flow elements
    text_content: ($) => token(prec(-1, /[^<@{}\s][^<@{}]*/)),

    // Identifier
    identifier: ($) => /[a-zA-Z_][a-zA-Z0-9_]*/,
  },
});
