/// <reference types="tree-sitter-cli/dsl" />
// @ts-check

/**
 * Waltzing Template Engine - Tree-sitter Grammar
 *
 * VALIDATION RULES (enforced by the compiler, not this grammar):
 *
 * 1. Reserved variable names - Variables named `__wtz_target` or `out` declared
 *    via `@let` are not allowed. This includes:
 *    - Direct @let declarations: @let out = "value"
 *    - For loop iterators: @for out in items
 *    - Tuple patterns: @for (a, out) in items
 *    - If-let patterns: @if let Some(out) = opt
 *    - Match arm patterns: @match status { Some(out) => { ... } }
 *    Workaround: Use @(out) to reference a variable with that name.
 *
 * 2. Function name conflicts - If a function `foo` exists, you cannot also
 *    have `foo_to_stream` (and vice versa). This prevents conflicts with
 *    the auto-generated streaming functions.
 *
 * 3. Special keywords (NEW SYNTAX - recommended):
 *    - @Out - Output target type (includes &mut): &mut _WtzTarget
 *    - @out - Output reference (includes &mut): &mut __wtz_buffer or __wtz_target
 *    - @render(T1, T2, ...) - Render callback type: impl Fn(T1, T2, ..., &mut _WtzTarget)
 *
 * 4. Special keywords (DEPRECATED - still supported):
 *    - @Target - Compiler-injected type (resolves to _WtzTarget or _WtzWriter)
 *    - @target - Compiler-injected variable (resolves to __wtz_buffer or __wtz_target)
 *
 * These rules apply only to @let template variables and @fn template functions,
 * not to CSS or JavaScript code.
 */

// REGEN STATUS — read before touching this grammar.
//
// `src/parser.c`, `src/grammar.json`, and `src/node-types.json` are generated
// from this file with tree-sitter-cli v0.25.10. Regeneration is expected to
// complete quickly on local hardware (last checked: 0.60s, <90 MB RSS).
//
// The previous grammar shape made `tree-sitter generate` explore an enormous
// LR/error-recovery state space. The important constraints that keep regen
// reliable are:
//   • All template-content bodies go through `_template_nodes`.
//   • HTML void elements are explicit; ordinary `<tag>` starts a full element.
//   • Function tags require `/>` for self-closing form.
//   • Rust expressions are intentionally opaque (`rust_expression`) except for
//     literals, paths, inferred enum variants, template blocks, and render
//     closures. The Waltzing compiler validates Rust expression semantics.
//   • Multi-depth template comments/raw blocks are not modeled as 22 token
//     variants; the common one-delimiter forms are tokenized for editor use.
//
// If any of those constraints are relaxed, run `tree-sitter generate` before
// committing and watch memory/time. A return to multi-GB RSS means the grammar
// has reintroduced exponential table construction.

module.exports = grammar({
  name: "waltzing",

  extras: ($) => [/\s/],

  conflicts: ($) => [
    // `|x|` — could be closure that returns `|x|` (no return type) or that's
    // about to declare a `-> T` return type. cli 0.25 needs both parses.
    [$.closure_type],
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

    content_block: ($) => seq("{", optional($._template_nodes), "}"),

    _template_nodes: ($) => repeat1($.template_node),

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
        seq("<", alias($.void_tag_name, $.tag_name), repeat($.attribute_or_control), optional("/"), ">"),
        // Full element with content and closing tag
        seq(
          "<",
          $.tag_name,
          repeat($.attribute_or_control),
          ">",
          optional($._template_nodes),
          "</",
          $.tag_name,
          ">",
        ),
      ),

    // Allow either HTML attributes or control flow (@if/@for) in attribute position
    attribute_or_control: ($) =>
      choice(
        $.attribute_control_flow,
        $.html_attribute,
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

    void_tag_name: ($) =>
      token(prec(1, choice(
        "area",
        "base",
        "br",
        "col",
        "embed",
        "hr",
        "img",
        "input",
        "link",
        "meta",
        "param",
        "source",
        "track",
        "wbr",
      ))),

    html_attribute: ($) =>
      seq($.attribute_name, optional(seq("=", $.attribute_value))),

    // Attribute names: allow @ directives like @click, while keeping @if/@for
    // visible as `@` + keyword tokens for attribute_control_flow.
    attribute_name: ($) =>
      choice(
        /[a-zA-Z_:][a-zA-Z0-9_:.-]*/,
        seq("@", /[a-zA-Z_][a-zA-Z0-9_:.-]*/),
      ),

    attribute_value: ($) =>
      choice(
        $.string_literal,
        $.template_expression,
        seq("@", "{", $.expression, "}"),
      ),

    // Template expressions
    template_expression: ($) =>
      choice(
        $.simple_expression,
        $.complex_expression,
        $.safe_expression,
        $.format_expression,
        $.matches_expression,
        $.out_ref,
        $.target_ref,
      ),

    // Compiler-injected output reference - resolves to &mut __wtz_buffer or __wtz_target
    out_ref: ($) => seq("@", "out"),

    // Compiler-injected target reference - DEPRECATED, use out_ref
    target_ref: ($) => seq("@", "target"),

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

    // Format expression: @format("template {}", arg1, arg2, ...)
    // Compiles to Rust's format!() macro.
    format_expression: ($) =>
      seq(
        "@",
        "format",
        "(",
        $.string_literal,
        repeat(seq(",", $.expression)),
        ")",
      ),

    // Matches expression: @matches(value, pattern) or @matches(value, pattern if guard)
    // Compiles to Rust's matches!() macro.
    matches_expression: ($) =>
      seq(
        "@",
        "matches",
        "(",
        $.expression,
        ",",
        $.pattern,
        optional(seq("if", $.expression)),
        ")",
      ),

    // Inferred enum path: ::Variant — the enum type is resolved from the
    // surrounding context (function parameter type). Tokenized so `::` and
    // the variant name parse as one unit.
    inferred_enum_path: ($) =>
      token(seq("::", /[a-zA-Z_][a-zA-Z0-9_]*/)),

    // Inferred enum tuple variant: ::Variant(arg1, arg2, ...)
    inferred_enum_call: ($) =>
      prec(4, seq($.inferred_enum_path, "(", optional($.argument_list), ")")),

    // Inferred enum struct variant: ::Variant { field: value, ... }
    inferred_enum_struct: ($) =>
      prec(
        4,
        seq(
          $.inferred_enum_path,
          "{",
          optional(seq(
            $.struct_field_init,
            repeat(seq(",", $.struct_field_init)),
            optional(","),
          )),
          "}",
        ),
      ),

    struct_field_init: ($) => seq($.identifier, ":", $.expression),

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
    // Use simple_pattern to avoid confusion with if/match expressions
    // containing { }. The `prec(3)` outranks `simple_expression`'s prec(2)
    // macro form so that `@let n` commits to let_statement rather than
    // parsing `@let` as `simple_expression` (= `@` + identifier `let`) and
    // leaving the `= 42` orphaned.
    let_statement: ($) =>
      prec(3, seq(seq("@", "let"), $.simple_pattern, "=", $.expression, optional(";"))),

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
        "/",
        ">",
      ),

    container_function_tag: ($) =>
      seq(
        "<@",
        $.function_path,
        repeat($.function_attribute),
        ">",
        optional($._template_nodes),
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
      choice($.string_literal, seq("@", $.expression_path), $.render_closure, $.unquoted_value),

    unquoted_value: ($) => /[^\s>=\/]+/,

    // Patterns - full patterns used in match arms
    pattern: ($) =>
      choice(
        $.wildcard_pattern,
        $.tuple_variant_pattern,
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

    tuple_variant_pattern: ($) =>
      seq(
        $.rust_path,
        "(",
        optional(seq($.pattern, repeat(seq(",", $.pattern)), optional(","))),
        ")",
      ),

    tuple_pattern: ($) =>
      seq("(", $.pattern, repeat(seq(",", $.pattern)), optional(","), ")"),

    // Expressions
    //
    // Rust expressions are intentionally kept mostly opaque here. Earlier
    // versions modeled Rust expressions recursively (`binary_expression`,
    // `method_call`, `statement_block`, etc.), but that gave the LR generator
    // too many paths across every Waltzing template-content boundary. The
    // compiler remains the authority for Rust expression validity; this grammar
    // only needs a stable editor tree around the template syntax.
    expression: ($) =>
      choice(
        $.template_block,
        $.render_closure,
        $.primary_expression,
      ),

    primary_expression: ($) =>
      choice(
        $.literal,
        $.inferred_enum_call,
        $.inferred_enum_struct,
        $.inferred_enum_path,
        $.rust_path,
        $.rust_expression,
      ),

    rust_expression: ($) =>
      token(prec(-1, /[^{}<@,;)\]\s][^{}<@,;)\]]*/)),

    // Template block - explicit template content in expression position
    // Use @{ ... } to create template content as an expression
    // Example: @let html = @{ <div>Hello</div> }
    template_block: ($) => seq("@", "{", optional($._template_nodes), "}"),

    // Render closure - inline closure that writes to output target
    // Generates: |params..., __wtz_target: &mut _WtzTarget| { template_content }
    // Auto-threading: calling a @() variable automatically appends @out
    render_closure: ($) =>
      seq(
        "@",
        "(",
        optional(seq($.parameter, repeat(seq(",", $.parameter)))),
        ")",
        $.content_block,
      ),
    argument_list: ($) =>
      seq($.expression, repeat(seq(",", $.expression)), optional(",")),

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
        $.closure_type,
        $.render_type,
        $.out_type,
        $.target_type,
      ),

    // Closure type: |[name: ]Type, ...| [-> ReturnType]
    closure_type: ($) =>
      seq(
        "|",
        optional(seq($.closure_param, repeat(seq(",", $.closure_param)))),
        "|",
        optional(seq("->", $.rust_type)),
      ),

    closure_param: ($) =>
      seq(optional(seq($.identifier, ":")), $.rust_type),

    // Compiler-injected render callback type - generates impl Fn(T1, T2, ..., &mut _WtzTarget)
    // Full syntax: @render(T1, T2, ...)
    // Shorthand: @() or @(T1, T2) - equivalent to @render()/@render(T1, T2)
    render_type: ($) =>
      choice(
        seq("@", "render", "(", optional(seq($.rust_type, repeat(seq(",", $.rust_type)))), ")"),
        seq("@", "(", optional(seq($.rust_type, repeat(seq(",", $.rust_type)))), ")"),
      ),

    // Compiler-injected output type - generates &mut _WtzTarget
    out_type: ($) => seq("@", "Out"),

    // Compiler-injected target type - DEPRECATED, use out_type
    target_type: ($) => seq("@", "Target"),

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

    // Template comments: @* ... *@.
    //
    // The compiler supports arbitrary delimiter depth (`@** ... **@`, etc.),
    // but modeling each depth as a separate token made error-recovery table
    // construction explode. Keep the common editor token here; deeper comments
    // still parse as ordinary text/error recovery until a scanner is added.
    template_comment: ($) => $.template_comment_1,
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

    // Raw blocks: @# ... #@.
    //
    // See template_comment: deeper delimiter variants are compiler-supported,
    // but keeping 22 separate tokens prevents reliable parser regeneration.
    raw_block: ($) => $.raw_block_1,
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
