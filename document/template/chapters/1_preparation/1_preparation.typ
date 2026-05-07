#import "../../../lib.typ": flex-caption, gls, glspl

= XSS Vector Analysis

== Sources

#gls("xss") exploit vectors were collected from two publicly available sources:

#table(
  columns: (auto, 1fr),
  stroke: 0.5pt,
  inset: 8pt,
  [*Source*], [*URL*],
  [xss.page], [#link("https://xss.page")],
  [SecLists XSS-Brutelogic],
  [#link("https://github.com/danielmiessler/SecLists/blob/master/Fuzzing/XSS/human-friendly/XSS-Brutelogic.txt")],
)

The xss.page list contains 150 payloads organised into categories including Basic, Event Handlers, SVG, HTML5,
JavaScript Context, Filter Bypasses, Encoding, Polyglots, #gls("dom"), #gls("waf") Bypass, Advanced, Legacy/IE, Modern,
Style/CSS, XML, Protocol Handlers, Obfuscation, Framework-Specific, Mobile, Data URIs, Comments, Meta, Interaction, and
CSS.

The Brutelogic list is a filter-bypass focused collection concentrating on minimal, evasion-oriented payloads that
exercise a wide variety of injection contexts.

== Clustering

Both lists were analysed using a custom clustering tool built into the translator program. Each payload line is
classified into one of eight structural groups by testing a series of string patterns in priority order:

#table(
  columns: (auto, 1fr, auto),
  stroke: 0.5pt,
  inset: 8pt,
  [*Cluster*], [*Matching criterion*], [*Maps to IPM*],
  [ScriptTag], [Line starts with `<script` (case-insensitive)], [IPM 1],
  [EventHandler],
  [Line contains `onerror=`, `onload=`, `onclick=`, `onmouseover=`, `onfocus=`, `onstart=`, or `ontoggle=`],
  [IPM 2],

  [AttributeBreakout], [Line starts with `">`, `'>`, `"><`, `'<`, or `><`], [IPM 3],
  [JSContext], [Line starts with `'-`, `"-`, `';`, `";`, `` `-` ``, or `];`], [IPM 4],
  [StyleCSS], [Line starts with `\<style` or contains `expression(`], [N/A],
  [DataURI], [Line contains `data:text/html` or `data:image/svg`], [N/A],
  [ProtocolHandler], [Line contains `javascript:` or `vbscript:`], [N/A],
  [Other], [No pattern matched], [N/A],
)

The clustering is run with:

```bash
xss_ca --cluster <payload_file1> <payload_file2> <payload_file3> ...
```

This writes one file per non-empty cluster under a `clusters/` subdirectory, named `<source_stem>_<ClusterName>.txt`.
The four clusters corresponding to structural injection contexts directly informed the four #glspl("ipm") described in
the next section.

= Input Parameter Models <ipms>

Four #glspl("ipm") were created, one for each identified structural injection pattern. All #glspl("ipm") use the #gls(
  "cagen",
) format: parameter values are symbolic names translated to concrete strings by the translator. The payload symbols
common to all four #glspl("ipm") resolve as follows:

#table(
  columns: (auto, 1fr),
  stroke: 0.5pt,
  inset: 8pt,
  [*Symbol*], [*Concrete value*],
  [`squote_payload`], [`console.log('rowhammer')`],
  [`dquote_payload`], [`console.log("rowhammer")`],
  [`eval_payload`], [`eval('console.log("rowhammer")')`],
  [`backtick_payload`], [``console.log(`rowhammer`)``],
)

== IPM 1: ScriptTag

_Template:_ `{open_tag}{whitespace}{payload}{close_tag}`

_Example:_ `<script>console.log('rowhammer')</script>`

_Situation:_ The injection point reflects user input directly into the HTML body without tag filtering, allowing a bare
`<script>` block.

```ipm
[System]
Name: ScriptTag

[Parameter]
open_tag   (enum): script_open, Script_open, SCRIPT_open
payload    (enum): squote_payload, dquote_payload, eval_payload
close_tag  (enum): script_close, Script_close, SCRIPT_close
encoding   (enum): none, url_encoded, html_entity
whitespace (enum): none, space, tab, newline

[Constraint]
-- html_entity breaks tag recognition for uppercase variant
encoding = "html_entity" => open_tag != "SCRIPT_open"
-- url encoding makes script tags unrecognizable without decoder
encoding = "url_encoded" => open_tag = "script_open"
-- eval payload wraps in eval(), html entity encoding breaks it
encoding = "html_entity" => payload != "eval_payload"
```

#table(
  columns: (auto, 1fr, auto),
  stroke: 0.5pt,
  inset: 8pt,
  [*Parameter*], [*Values*], [*Count*],
  [`open_tag`], [`script_open`, `Script_open`, `SCRIPT_open`], [3],
  [`payload`], [`squote_payload`, `dquote_payload`, `eval_payload`], [3],
  [`close_tag`], [`script_close`, `Script_close`, `SCRIPT_close`], [3],
  [`encoding`], [`none`, `url_encoded`, `html_entity`], [3],
  [`whitespace`], [`none`, `space`, `tab`, `newline`], [4],
  [*Total*], [], [*16*],
)

== IPM 2: EventHandler

_Template:_ `<{tag} {src_attr} {event}={quote}{payload}{quote}>`

_Example:_ `<img src="x" onerror="console.log('rowhammer')">`

_Situation:_ User input is reflected inside or adjacent to an HTML element attribute, allowing injection of an event
handler.

```ipm
[System]
Name: EventHandler

[Parameter]
tag         (enum): img, svg, body, input, video
src_attr    (enum): empty, src_x, src_valid
event       (enum): onerror, onload, onfocus, onmouseover, onclick
payload     (enum): squote_payload, dquote_payload, eval_payload
quote_style (enum): dquote, squote, no_quote
tag_case    (enum): lower, upper, mixed

[Constraint]
-- onerror only fires when a src is present and fails to load
event = "onerror" => src_attr != "empty"
-- svg and body do not support src attributes
tag = "svg"  => src_attr = "empty"
tag = "body" => src_attr = "empty"
-- onfocus only makes sense on input-like elements
event = "onfocus" => tag = "input"
-- onload fires on resource-loading tags, not input
event = "onload" => tag != "input"
```

#table(
  columns: (auto, 1fr, auto),
  stroke: 0.5pt,
  inset: 8pt,
  [*Parameter*], [*Values*], [*Count*],
  [`tag`], [`img`, `svg`, `body`, `input`, `video`], [5],
  [`src_attr`], [`empty`, `src_x`, `src_valid`], [3],
  [`event`], [`onerror`, `onload`, `onfocus`, `onmouseover`, `onclick`], [5],
  [`payload`], [`squote_payload`, `dquote_payload`, `eval_payload`], [3],
  [`quote_style`], [`dquote`, `squote`, `no_quote`], [3],
  [`tag_case`], [`lower`, `upper`, `mixed`], [3],
  [*Total*], [], [*22*],
)

== IPM 3: AttributeBreakout

_Template:_ `{breakout}{prefix}<{tag} {event}={quote}{payload}{quote}{suffix}`

_Example:_ `"><img onerror="console.log('rowhammer')">`

_Situation:_ User input is reflected inside an HTML attribute value. The payload escapes the enclosing attribute context
and injects a new element with an event handler.

```ipm
[System]
Name: AttributeBreakout

[Parameter]
breakout (enum): dquote_close, squote_close, angle_close
prefix   (enum): empty, space, slash
tag      (enum): img, svg, script
event    (enum): onerror, onload, onmouseover
payload  (enum): squote_payload, dquote_payload
suffix   (enum): empty, comment_close, space_slash

[Constraint]
-- angle_close does not escape a quoted context so no prefix is needed
breakout = "angle_close" => prefix = "empty"
-- onerror needs img, not svg or script
event = "onerror" => tag = "img"
-- onload fits svg, not script which has no onload
event = "onload" => tag != "script"
-- script tag uses a different structure, restrict to safe event
tag = "script" => event = "onmouseover"
```

#table(
  columns: (auto, 1fr, auto),
  stroke: 0.5pt,
  inset: 8pt,
  [*Parameter*], [*Values*], [*Count*],
  [`breakout`], [`dquote_close`, `squote_close`, `angle_close`], [3],
  [`prefix`], [`empty`, `space`, `slash`], [3],
  [`tag`], [`img`, `svg`, `script`], [3],
  [`event`], [`onerror`, `onload`, `onmouseover`], [3],
  [`payload`], [`squote_payload`, `dquote_payload`], [2],
  [`suffix`], [`empty`, `comment_close`, `space_slash`], [3],
  [*Total*], [], [*17*],
)

== IPM 4: JSContext

_Template:_ `{breakout}{separator}{payload}{separator}{comment}`

_Example:_ `'; console.log('rowhammer'); //`

_Situation:_ User input is reflected inside an existing JavaScript string or expression. The payload breaks out of the
string, executes the payload as a statement, then comments out the remainder of the original expression.

```ipm
[System]
Name: JSContext

[Parameter]
breakout  (enum): squote_break, dquote_break, backtick_break, bracket_break
separator (enum): semicolon, newline, space
payload   (enum): squote_payload, dquote_payload, backtick_payload
comment   (enum): line_comment, block_comment, none
encoding  (enum): none, unicode_escape

[Constraint]
-- quote style of payload must not match breakout to avoid re-breaking
breakout = "squote_break"   => payload != "squote_payload"
breakout = "dquote_break"   => payload != "dquote_payload"
breakout = "backtick_break" => payload != "backtick_payload"
-- unicode escape obfuscates the payload, a trailing comment is redundant
encoding = "unicode_escape" => comment = "none"
```

#table(
  columns: (auto, 1fr, auto),
  stroke: 0.5pt,
  inset: 8pt,
  [*Parameter*], [*Values*], [*Count*],
  [`breakout`], [`squote_break`, `dquote_break`, `backtick_break`, `bracket_break`], [4],
  [`separator`], [`semicolon`, `newline`, `space`], [3],
  [`payload`], [`squote_payload`, `dquote_payload`, `backtick_payload`], [3],
  [`comment`], [`line_comment`, `block_comment`, `none`], [3],
  [`encoding`], [`none`, `unicode_escape`], [2],
  [*Total*], [], [*15*],
)

== Summary

#table(
  columns: (auto, auto, auto, auto),
  stroke: 0.5pt,
  inset: 8pt,
  [*IPM*], [*Parameters*], [*Values*], [*Constraints*],
  [ScriptTag], [5], [16], [3],
  [EventHandler], [6], [22], [5],
  [AttributeBreakout], [6], [17], [4],
  [JSContext], [5], [15], [4],
  [*Total*], [*22*], [*70*], [*16*],
)


= Covering Array Generation

#glspl("ca") were generated using the #gls("cagen") at strength t = 2 for all four #glspl("ipm"), and additionally at
strength t = 3 for IPM 1 (ScriptTag). Before exporting, "Randomize Don't-Care Values" was applied in #gls("cagen") to
ensure all cells contain concrete symbolic values rather than wildcard entries.

#table(
  columns: (auto, auto, auto),
  stroke: 0.5pt,
  inset: 8pt,
  [*File*], [*IPM*], [*Strength*],
  [`ca_scripttag_t2.csv`], [ScriptTag], [t = 2],
  [`ca_scripttag_t3.csv`], [ScriptTag], [t = 3],
  [`ca_eventhandler_t2.csv`], [EventHandler], [t = 2],
  [`ca_attributebreakout_t2.csv`], [AttributeBreakout], [t = 2],
  [`ca_jscontext_t2.csv`], [JSContext], [t = 2],
)

= Translation <translation>

== Overview

Translation is performed by a custom tool written in Rust. It accepts either a #gls("ca") CSV file or a raw payload list
as input:

```bash
xss_ca --translate <ca_file.csv> <ca_file2.csv> ...
xss_ca --cluster <payload_file> <payload_file2> ...
```

The tool auto-detects the target #gls("ipm") from the CSV column headers and writes one exploit string per row to a
`translated_exploits/` subdirectory next to the input file.

== Module Structure

#table(
  columns: (auto, 1fr),
  stroke: 0.5pt,
  inset: 8pt,
  [*Module*], [*Responsibility*],
  [`main.rs`], [#gls("cli") argument parsing and orchestration],
  [`encoding.rs`], [`apply_encoding`, `apply_case`, random mixed-case generation],
  [`symbols.rs`], [Symbol table construction and dynamic value generators],
  [`translate.rs`], [IPM detection and the four `translate_*` functions],
  [`cluster.rs`], [Payload clustering by structural pattern],
  [`error.rs`], [Shared `AppError` type],
)

== Dynamic Generation

Three parameters are generated dynamically at startup rather than mapped to fixed strings:

#table(
  columns: (auto, auto, 1fr),
  stroke: 0.5pt,
  inset: 8pt,
  [*Parameter*], [*Symbol*], [*Generation and validation*],
  [src attribute],
  [`src_valid`],
  [Random lowercase alphanumeric domain (4 to 10 chars) and path (3 to 8 chars). Validated to contain no characters that
    break an HTML attribute value (`'`, `>`, space).],

  [Whitespace],
  [`space`],
  [Random sequence of 1 to 4 space or tab characters. Validated to be non-empty and contain only whitespace
    characters.],

  [Tag casing],
  [`mixed`],
  [Per-character random upper/lower decision applied at translate time. Validated to preserve the original tag string
    length.],
)

== Quote Safety

The event handler and attribute breakout translators include quote safety logic. When the requested quote style would
produce a conflict with a quote character inside the payload, the translator either switches to the opposite quote style
or HTML-entity encodes the conflicting characters. This is safe for event handler attributes because browsers decode
HTML entities in attribute values before passing them to the JavaScript engine.

== Produced Test Sets

#table(
  columns: (auto, auto, auto),
  stroke: 0.5pt,
  inset: 8pt,
  [*File*], [*IPM*], [*Strength*],
  [`exploits_scripttag_t2.txt`], [ScriptTag], [t = 2],
  [`exploits_scripttag_t3.txt`], [ScriptTag], [t = 3],
  [`exploits_eventhandler_t2.txt`], [EventHandler], [t = 2],
  [`exploits_attributebreakout_t2.txt`], [AttributeBreakout], [t = 2],
  [`exploits_jscontext_t2.txt`], [JSContext], [t = 2],
)

Each file contains one #gls("xss") exploit per line. All exploits call `console.log('rowhammer')` (or a variant using
`eval` or backtick syntax) when executed, confirming successful JavaScript injection without blocking the browser UI.
