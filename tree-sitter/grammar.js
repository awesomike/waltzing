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
// complete quickly on local hardware (last checked: 0.30s, <90 MB RSS).
//
// The previous grammar shape made `tree-sitter generate` explore an enormous
// LR/error-recovery state space. The important constraints that keep regen
// reliable are:
//   • All template-content bodies go through `_template_nodes`.
//   • HTML void elements are explicit; ordinary `<tag>` starts a full element.
//   • Function tags require `/>` for self-closing form.
//   • The bare `rust_expression` token stays at `prec(-1)` — it is a low-
//     priority fallback that every keyword, path, literal, and operator must
//     outrank. Bumping it to prec 0/1 makes it win greedily and blew the
//     corpus error count to ~1440. Richer Rust syntax is modeled with
//     *rules* built from existing tokens (`path_expression`, `binary_
//     expression`, `match_expression`, …), never by widening that token.
//   • Multi-depth template comments/raw blocks are not modeled as 22 token
//     variants; the 1–3 delimiter forms are tokenized for editor use.
//
// If any of those constraints are relaxed, run `tree-sitter generate` before
// committing and watch memory/time. A return to multi-GB RSS means the grammar
// has reintroduced exponential table construction.
//
// REAL-WORLD COVERAGE: `npm run corpus-check` parses the sibling `cli/` +
// `libraries/waltzing-ui/` `.wtz` trees and guards the ERROR-node count
// against regressions (current budget: 10 nodes across 65 files, 61 fully
// clean — down from 887/5). The remaining 10 are concentrated and hard:
//   • `@expr` function-tag attribute values that contain a quoted string
//     (`<@c x=@Some("a b")/>`): the `unquoted_value` token shadows the
//     expression branch. Excluding `@`/`"` from it regresses ~110 other
//     nodes, so it is left alone.
//   • Embedded JS that is a Rust string built inside a block expression
//     (string concat with `'…'` / `{` inside `"…"`), and JS regex literals
//     (`/^\d+$/`) — both want a real JS sub-grammar / scanner.
// Closing these needs careful, measure-driven work — run corpus-check after
// every change; many naive widenings regress the count badly.

module.exports = grammar({
  name: "waltzing",

  extras: ($) => [/\s/],

  conflicts: ($) => [
    // `|x|` — could be closure that returns `|x|` (no return type) or that's
    // about to declare a `-> T` return type. cli 0.25 needs both parses.
    [$.closure_type],
    // `expr as Path` — `Path` could still take `<T>` (generic_type) or stop
    // (path_type). Exposed by `cast_expression` putting a `rust_type` after
    // `as`; resolved with a GLR split.
    [$.generic_type, $.path_type],
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

    // Template import path. Covers global (`//x`), alias-relative (`/x`),
    // and bare sibling (`templates/base.wtz`) forms — the leading `/` is
    // optional, so a non-quoted path need not start with it.
    import_path: ($) => /[^\s"][^\s"]*/,

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
      seq(
        seq("@", "fn"),
        $.identifier,
        $.parameter_list,
        // Either a template body, or `: ReturnType = { rust }` for a function
        // that computes and returns a plain Rust value (e.g. class strings).
        choice(
          $.content_block,
          seq(":", $.rust_type, "=", $.rust_block),
        ),
      ),

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
        $.raw_text_element,
        $.doctype,
        $.function_tag,
        $.template_control_flow,
        $.template_expression,
        $.comment,
        $.raw_block,
        $.embedded_language,
        $.escape_at,
        $.text_content,
      ),

    // HTML doctype declaration, e.g. `<!DOCTYPE html>`. The `<![a-zA-Z]`
    // prefix never collides with `<!--` HTML comments.
    doctype: ($) => token(/<![a-zA-Z][^>]*>/),

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

    // `<style>` / `<script>` hold raw text (CSS / JS), not template content —
    // their bodies must not be parsed as `_template_nodes` or every `{`, `}`,
    // `;` inside the stylesheet errors. `raw_text_tag_name` outranks the
    // generic `tag_name` token so these tags always take this branch.
    raw_text_element: ($) =>
      seq(
        "<",
        alias($.raw_text_tag_name, $.tag_name),
        repeat($.attribute_or_control),
        ">",
        optional($.raw_text),
        "</",
        alias($.raw_text_tag_name, $.tag_name),
        ">",
      ),

    raw_text_tag_name: ($) => token(prec(2, choice("style", "script"))),

    // Anything up to the closing `</`. A `<` is fine as long as it is not the
    // start of that closing tag.
    raw_text: ($) => token(prec(-1, /([^<]|<[^/])+/)),

    // Allow HTML attributes, control flow (@if/@for), or comments in
    // attribute position — `@* … *@` is common between attributes.
    attribute_or_control: ($) =>
      choice(
        $.attribute_control_flow,
        $.html_attribute,
        $.comment,
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
        optional(seq("let", $.pattern, "=")),
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
        // `@@click` — an escaped `@` attribute name (Alpine `@click` shorthand);
        // a lone `@` would otherwise enter expression mode.
        seq("@@", /[a-zA-Z_][a-zA-Z0-9_:.-]*/),
      ),

    attribute_value: ($) =>
      choice(
        $.string_literal,
        $.template_expression,
        $.embedded_language,
        $.raw_block,
        seq("@", "{", $.expression, "}"),
        // `@"…"` — literal string value: lets a `@` appear inside (a bare
        // `@` would otherwise enter expression mode), e.g. `src=@"…/@x/…"`.
        seq("@", $.string_literal),
        // A bare `@if` / `@match` expression as the value, e.g.
        // `x-show=@if cond { "a" } else { "b" }`.
        seq("@", $.if_expression),
        seq("@", $.match_expression),
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
        seq("@", "&", $.expression_path),  // Borrowed: @&user.display_name
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
      prec(
        3,
        seq(
          seq("@", "let"),
          $.simple_pattern,
          optional(seq(":", $.rust_type)),
          "=",
          $.expression,
          optional(";"),
        ),
      ),

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
        // Range iterator, e.g. `@for i in 0..n` / `@for p in start..=end`.
        optional(seq(choice("..", "..="), $.expression)),
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

    // An unquoted attribute value excludes `"` so a quoted value always
    // lexes as `string_literal` — otherwise this token swallows the opening
    // `"` and truncates `attr="a b c"` at the first space.
    unquoted_value: ($) => /[^\s>=\/"]+/,

    // Patterns - full patterns used in match arms
    pattern: ($) =>
      choice(
        $.wildcard_pattern,
        $.tuple_variant_pattern,
        $.tuple_pattern,
        $.struct_pattern,
        $.literal,
        // A bare `rust_path` covers both single-segment bindings (`v`) and
        // unit-variant / path patterns (`None`, `Color::Red`). The lexer
        // produces a `rust_path` token for a lone identifier in pattern
        // position, so `identifier_pattern` (which wants an `identifier`
        // token) is only reachable via a `ref` / `mut` prefix — keep both.
        $.rust_path,
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

    // Bare identifier binding, with optional `ref` / `mut` binding mode
    // (e.g. `Some(ref h)`, `Some(mut x)`).
    identifier_pattern: ($) =>
      seq(optional(choice("ref", "mut")), $.identifier),

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
        $.array_literal,
        $.reference_expression,
        $.unary_expression,
        $.binary_expression,
        $.cast_expression,
        $.paren_expression,
        $.closure_expression,
        $.path_expression,
        $.match_expression,
        $.if_expression,
        $.rust_block,
        $.out_ref,
        $.target_ref,
        $.rust_path,
        $.rust_expression,
      ),

    rust_expression: ($) =>
      token(prec(-1, /[^{}<@(\[,;)\]\s][^{}<@(\[,;)\]]*/)),

    // `match` / `if` used as Rust *expressions* (e.g. `@let cls = match v { … }`
    // or `@cn([base, if cond { "a" } else { "b" }])`). Distinct from the
    // `@match` / `@if` template control flow: no `@`, and the arm / branch
    // bodies are opaque Rust blocks (`rust_block`), not template content.
    match_expression: ($) =>
      prec(
        2,
        seq("match", $.expression, "{", repeat($.match_expression_arm), "}"),
      ),

    match_expression_arm: ($) =>
      seq(
        $.pattern,
        optional(seq("if", $.expression)),
        "=>",
        // `expression` already reaches `rust_block` via `primary_expression`,
        // so a braced arm body and a bare-expression arm body share one path.
        $.expression,
        optional(","),
      ),

    if_expression: ($) =>
      prec.right(
        2,
        seq(
          "if",
          optional(seq("let", $.pattern, "=")),
          $.expression,
          $.rust_block,
          optional(seq("else", choice($.if_expression, $.rust_block))),
        ),
      ),

    // Opaque braced Rust — the body of a `match` / `if` expression arm. Spans
    // balanced `{}` `()` `[]`; recursion is bounded since each level needs a
    // literal opening delimiter. Not template content — no `<…>` markup here.
    rust_block: ($) => seq("{", repeat($._rust_token), "}"),

    _rust_token: ($) =>
      choice(
        $.rust_expression,
        $.literal,
        $.rust_block,
        // Inside an opaque Rust block `<` is never an HTML tag opener — it's
        // a comparison or a generic-args bracket (`Vec<&str>`). The bare
        // `rust_expression` token excludes `<` for HTML safety, so admit it
        // explicitly here.
        "<",
        seq("(", repeat($._rust_token), ")"),
        seq("[", repeat($._rust_token), "]"),
        ",",
        ";",
      ),

    // Array / slice literal in expression position, e.g. the `["a", cls]` in
    // `@cn(["a", cls])`. The `[` / `]` are literal tokens so recursion is
    // bounded; the bare `rust_expression` token can't span the interior `,`
    // so this rule is needed for multi-element arrays. A `&[…]` slice ref is
    // `reference_expression` wrapping this.
    array_literal: ($) =>
      seq("[", optional($.argument_list), "]"),

    // `&expr` borrow in expression position, e.g. the RHS of
    // `@if let Some(v) = &min_str`. The leading `&` is otherwise a bare token
    // (from `array_literal` / attribute refs) that shadows `rust_expression`.
    reference_expression: ($) =>
      prec(2, seq("&", $.primary_expression)),

    // `!expr` / `-expr` unary prefix, e.g. `@if !unit_name.is_empty()`.
    unary_expression: ($) =>
      prec(3, seq(choice("!", "-"), $.primary_expression)),

    // Parenthesised expression, e.g. `(selected == Some(v))` — the `(` / `)`
    // are literal tokens, so recursion is bounded.
    paren_expression: ($) =>
      seq("(", $.expression, ")"),

    // Rust closure value, e.g. `|v| v.to_string()` in `input.map(|v| …)`.
    // Distinct from `closure_type` (a type in parameter position) and
    // `render_closure` (`@(…)`). A bare `|` here can't be confused with the
    // `||` binary operator since that is a single two-char token.
    closure_expression: ($) =>
      prec(
        1,
        seq(
          "|",
          optional(seq($.simple_pattern, repeat(seq(",", $.simple_pattern)))),
          "|",
          $.expression,
        ),
      ),

    // Binary operator expression, e.g. `step > 0.0`, `a == b && c`. Operands
    // are `primary_expression` (not the full `expression`) to keep the LR
    // table tractable. Bare `<` is deliberately excluded — it's ambiguous
    // with HTML tag openers — so `a < b` comparisons stay unparsed.
    binary_expression: ($) =>
      prec.left(
        1,
        seq($.primary_expression, $.binary_operator, $.primary_expression),
      ),

    binary_operator: ($) =>
      choice(
        "==", "!=", ">=", "<=", ">", "&&", "||",
        "+", "-", "*", "/", "%",
        // `<` only as a comparison when followed by whitespace — `<` directly
        // before a name (`<div`) or `@` (`<@comp`) is always an HTML / function
        // tag open, never a comparison, so this never shadows markup.
        $.lt_operator,
      ),

    lt_operator: ($) => token(seq("<", /[ \t]/)),

    // `expr as Type` cast, e.g. `value_min.map(|v| v as f64)`.
    cast_expression: ($) =>
      prec.left(1, seq($.primary_expression, "as", $.rust_type)),

    // Method / field / index chains off a path, e.g. `data.is_some()` or
    // `value.map(...)`. Built from `rust_path` tokens (not the opaque
    // `rust_expression`) so `rust_path` can't truncate the chain at the first
    // `.`. `repeat1` + `token.immediate` keep a lone `foo` a plain `rust_path`.
    path_expression: ($) =>
      prec(
        2,
        seq(
          // A string literal can also be the base of a method chain, e.g.
          // `"…".to_string()` or `"open = false".replace(…)`.
          choice($.rust_path, $.string_literal),
          repeat1(
            choice(
              seq(token.immediate("."), $.rust_path),
              seq(token.immediate("("), optional($.argument_list), ")"),
              seq(token.immediate("["), optional($.argument_list), "]"),
            ),
          ),
        ),
      ),

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
        $.raw_string,
        $.string_literal,
        $.char_literal,
        $.number_literal,
        $.boolean_literal,
      ),

    string_literal: ($) =>
      seq('"', repeat(choice(/[^"\\]/, $.escape_sequence)), '"'),

    // Rust raw string: `r"…"`, `r#"…"#`, `r##"…"##`. Tokenised so embedded
    // `"` / `{` / `@` don't terminate it. One- and two-hash forms cover the
    // corpus; deeper nesting is rare enough to leave unhandled.
    raw_string: ($) =>
      token(
        choice(
          /r"[^"]*"/,
          /r#"([^"]|"[^#])*"#/,
          /r##"([^#]|#[^#]|"#?[^#])*"##/,
        ),
      ),

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

    // The fractional part requires at least one digit so a bare `0.` can't
    // swallow the first dot of a `0..n` range literal.
    float_literal: ($) => /[0-9][0-9_]*\.[0-9][0-9_]*([eE][+-]?[0-9]+)?(f32|f64)?/,

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
      // The name may lex as `rust_path` (the `identifier` / `rust_path` token
      // collision): in closure-param position both a bare name and a path
      // type are valid, so accept either for the name.
      seq(optional(seq(choice($.identifier, $.rust_path), ":")), $.rust_type),

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

    // Template comments: @* ... *@, @** ... **@, @*** ... ***@.
    //
    // The compiler supports arbitrary delimiter depth, but each depth is a
    // separate token; wiring up too many at once bloats the error-recovery
    // tables. The 1–3 asterisk forms cover real-world templates — deeper
    // comments fall back to text/error recovery until a scanner is added.
    template_comment: ($) =>
      choice(
        $.template_comment_1,
        $.template_comment_2,
        $.template_comment_3,
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
