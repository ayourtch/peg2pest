# peg2pest
Convertor from https://github.com/pointlander/peg grammars into https://github.com/pest-parser/pest

## Usage

```
$ cargo run -- -i peg/peg.peg
Grammar =  ${ Spacing ~ "package" ~ MustSpacing ~ Identifier ~ Import* ~ "type" ~ MustSpacing ~ Identifier ~ "Peg" ~ Spacing ~ Action ~ Definition+ ~ EndOfFile }
Import =  ${ "import" ~ Spacing ~ ( MultiImport | SingleImport ) ~ Spacing }
SingleImport =  ${ ImportName }
MultiImport =  ${ "(" ~ Spacing ~ ( ImportName ~ "\n" ~ Spacing )* ~ Spacing ~ ")" }
ImportName =  ${ ( "\"") ~ ( '0'..'9' | 'a'..'z' | 'A'..'Z' | "_" | "/" | "." | "-")+ ~ ( "\"") }
Definition =  ${ Identifier ~ LeftArrow ~ Expression ~ & ( Identifier ~ LeftArrow | EOI ) }
Expression =  ${ ( Sequence ~ ( Slash ~ Sequence )* ~ ( Slash )? )? }
Sequence =  ${ Prefix ~ ( Prefix )* }
Prefix =  ${ And ~ Action | Not ~ Action | And ~ Suffix | Not ~ Suffix | Suffix }
Suffix =  ${ Primary ~ ( Question | Star | Plus )? }
Primary =  ${ Identifier ~ ! LeftArrow | Open ~ Expression ~ Close | Literal | Class | Dot | Action | Begin ~ Expression ~ End }
Identifier =  ${ IdentStart ~ IdentCont* ~ Spacing }
IdentStart =  ${ ( 'a'..'z' | 'A'..'Z' | "_") }
IdentCont =  ${ IdentStart | ( '0'..'9') }
Literal =  ${ ( "'") ~ ( ! ( "'") ~ Char )? ~ ( ! ( "'") ~ Char )* ~ ( "'") ~ Spacing | ( "\"") ~ ( ! ( "\"") ~ DoubleChar )? ~ ( ! ( "\"") ~ DoubleChar )* ~ ( "\"") ~ Spacing }
Class =  ${ ( "[[" ~ ( "^" ~ DoubleRanges | DoubleRanges )? ~ "]]" | "[" ~ ( "^" ~ Ranges | Ranges )? ~ "]" ) ~ Spacing }
Ranges =  ${ ! "]" ~ Range ~ ( ! "]" ~ Range )* }
DoubleRanges =  ${ ! "]]" ~ DoubleRange ~ ( ! "]]" ~ DoubleRange )* }
Range =  ${ Char ~ "-" ~ Char | Char }
DoubleRange =  ${ Char ~ "-" ~ Char | DoubleChar }
Char =  ${ Escape | ! "\\" ~ ANY }
DoubleChar =  ${ Escape | ( 'a'..'z' | 'A'..'Z') | ! "\\" ~ ANY }
Escape =  ${ ^"\\a" | ^"\\b" | ^"\\e" | ^"\\f" | ^"\\n" | ^"\\r" | ^"\\t" | ^"\\v" | ^"\\'" | "\\\"" | "\\[" | "\\]" | "\\-" | "\\" ~ ^"0x" ~ ( '0'..'9' | 'a'..'f' | 'A'..'F')+ | "\\" ~ ( '0'..'3') ~ ( '0'..'7') ~ ( '0'..'7') | "\\" ~ ( '0'..'7') ~ ( '0'..'7')? | "\\\\" }
LeftArrow =  ${ ( "<-" | "\0x2190" ) ~ Spacing }
Slash =  ${ "/" ~ Spacing }
And =  ${ "&" ~ Spacing }
Not =  ${ "!" ~ Spacing }
Question =  ${ "?" ~ Spacing }
Star =  ${ "*" ~ Spacing }
Plus =  ${ "+" ~ Spacing }
Open =  ${ "(" ~ Spacing }
Close =  ${ ")" ~ Spacing }
Dot =  ${ "." ~ Spacing }
SpaceComment =  ${ ( Space | Comment ) }
Spacing =  ${ SpaceComment* }
MustSpacing =  ${ SpaceComment+ }
Comment =  ${ ( "#" | "//" ) ~ ( ! EndOfLine ~ ANY )* ~ EndOfLine }
Space =  ${ " " | "\t" | EndOfLine }
EndOfLine =  ${ "\r\n" | "\n" | "\r" }
EndOfFile =  ${ EOI }
Action =  ${ "{" ~ ActionBody* ~ "}" ~ Spacing }
ActionBody =  ${ (!( "{" | "}" ) ~ ANY) | "{" ~ ActionBody* ~ "}" }
Begin =  ${ "<" ~ Spacing }
End =  ${ ">" ~ Spacing }
```
