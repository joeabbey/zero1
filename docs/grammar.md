Below is a **complete, self‑contained grammar** for **Zero1 (Z1)** cells (`.z1c` compact and `.z1r` relaxed). It’s written as EBNF with a small set of disambiguation and precedence rules. Manifest/test DSLs are out of scope here; this is the language grammar for code cells only.

> **Key invariants**
>
> * One module per cell.
> * Compact and relaxed forms are both accepted by a **single grammar** via dual‑keyword tokens.
> * Statements are **semicolon‑terminated**; the final semicolon before `}` is optional. `return/ret` never requires `;`.
> * Shadow metadata lines (`//@z1:` / `//:key:`) are treated as comments and ignored by the parser.
> * `#sym { long: short, ... }` appears at top‑level (anywhere), and is ignored by semantic hashing.

---

## 1) Lexical grammar

```
File              ::= { WS | Comment | ShadowMeta } Module EOF

WS                ::= { " " | "\t" | Newline }
Newline           ::= "\r\n" | "\n" | "\r"

Comment           ::= "//" { !Newline . } Newline
ShadowMeta        ::= "//@z1:" { !Newline . } Newline
                    | "//:" Ident ":" { !Newline . } Newline

Ident             ::= IdentStart { IdentCont }
IdentStart        ::= "_" | Letter
IdentCont         ::= "_" | Letter | Digit
Letter            ::= /* Unicode XID_Start */
Digit             ::= "0"…"9"

String            ::= "\"" { StringChar } "\""
StringChar        ::= Esc | !("\"" | "\n" | "\r") .
Esc               ::= "\\" ( "\"" | "\\" | "n" | "r" | "t" | "u" Hex Hex Hex Hex )
Hex               ::= Digit | "a"…"f" | "A"…"F"

Int               ::= "0" | ( "1"…"9" { Digit } )
UIntSuffix        ::= "U16" | "U32" | "U64"     /* optional numeric suffix (see Expr) */

VersionLit        ::= Int { "." Int }           /* e.g., 1, 1.0, 1.2.3 */

Path              ::= Ident { "." Ident }       /* e.g., http.server or H.Res */

CapName           ::= Ident { "." Ident }       /* e.g., fs.ro, net, time */

/* Keywords are reserved; both compact and relaxed are recognized */
KW_MODULE         ::= "m" | "module"
KW_USE            ::= "u" | "use"
KW_TYPE           ::= "t" | "type"
KW_FN             ::= "f" | "fn"
KW_EXTERN         ::= "x" | "extern"
KW_RETURN         ::= "ret" | "return"
KW_EFF            ::= "eff"
KW_CTX            ::= "ctx"
KW_CAPS           ::= "caps"
KW_AS             ::= "as"
KW_ONLY           ::= "only"
KW_IF             ::= "if"
KW_ELSE           ::= "else"
KW_WHILE          ::= "while"
KW_MATCH          ::= "match"
KW_LET            ::= "let"
KW_MUT            ::= "mut"
KW_TASK           ::= "task"
KW_AWAIT          ::= "await"
KW_WHERE          ::= "where"

/* Primitive type names (reserved idents) */
KW_BOOL           ::= "Bool"
KW_STR            ::= "Str"
KW_UNIT           ::= "Unit"
KW_U16            ::= "U16"
KW_U32            ::= "U32"
KW_U64            ::= "U64"
```

**Tokenization rules**

* Longest‑match, then keyword precedence over identifiers.
* Whitespace and comments (including `ShadowMeta`) are skipped between tokens.
* Strings use C‑style escapes; no multiline strings.

---

## 2) Syntactic grammar

### 2.1 Module & header

```
Module            ::= ModuleHeader { TopItem }

ModuleHeader      ::= KW_MODULE Path VersionOpt CtxOpt CapsOpt SymHeaderOpt
VersionOpt        ::= [ ":" VersionLit ]
CtxOpt            ::= [ KW_CTX "=" Int ]
CapsOpt           ::= [ KW_CAPS "=" "[" CapListOpt "]" ]
CapListOpt        ::= [ CapName { "," CapName } ]
SymHeaderOpt      ::= [ SymHeader ]
SymHeader         ::= "#sym" "{" SymPair { "," SymPair } "}"
SymPair           ::= Ident ":" Ident       /* long : short */
```

> **Note:** `SymHeader` may also appear as a `TopItem`. If multiple appear, later pairs override earlier ones for formatting only.

### 2.2 Top‑level items

```
TopItem           ::= { WS | Comment | ShadowMeta }
                      ( Import
                      | TypeDecl
                      | ExternDecl
                      | FnDecl
                      | SymHeader
                      )

Import            ::= KW_USE String AliasOpt OnlyOpt
AliasOpt          ::= [ KW_AS Ident ]
OnlyOpt           ::= [ KW_ONLY "[" UseListOpt "]" ]
UseListOpt        ::= [ Ident { "," Ident } ]
```

### 2.3 Types

```
TypeDecl          ::= KW_TYPE Ident TypeParamsOpt "=" TypeExpr

TypeParamsOpt     ::= [ "<" TypeParam { "," TypeParam } ">" WhereClauseOpt ]
TypeParam         ::= Ident
WhereClauseOpt    ::= [ KW_WHERE TypeBound { "," TypeBound } ]
TypeBound         ::= Ident ":" TraitList
TraitList         ::= Trait { "+" Trait }
Trait             ::= Ident                     /* e.g., Copy, Send (names only) */

TypeExpr          ::= TypeSum

TypeSum           ::= TypeProd { "|" TypeProd }             /* sum/union */
TypeProd          ::= TypePostfix                           /* (reserved for future) */
TypePostfix       ::= TypePrimary { GenericSuffix }
GenericSuffix     ::= "<" TypeExpr { "," TypeExpr } ">"

TypePrimary       ::= KW_BOOL | KW_STR | KW_UNIT
                    | KW_U16  | KW_U32 | KW_U64
                    | Path                                  /* named/aliased type */
                    | "{" FieldTypeListOpt "}"              /* structural record */
                    | "(" TypeExpr ")"                      /* parenthesized */

FieldTypeListOpt  ::= [ FieldType { "," FieldType } [ "," ] ]
FieldType         ::= Ident ":" TypeExpr
```

### 2.4 Functions (extern + defined)

```
ExternDecl        ::= KW_EXTERN KW_FN Ident TypeParamsFnOpt "(" ParamListOpt ")"
                      "->" TypeExpr EffAnnOpt ";"

FnDecl            ::= KW_FN Ident TypeParamsFnOpt "(" ParamListOpt ")"
                      "->" TypeExpr EffAnnOpt Block

TypeParamsFnOpt   ::= [ "<" TypeParam { "," TypeParam } ">" WhereClauseOpt ]
ParamListOpt      ::= [ Param { "," Param } ]
Param             ::= Ident ":" TypeExpr

EffAnnOpt         ::= [ KW_EFF "[" EffListOpt "]" ]
EffListOpt        ::= [ Effect { "," Effect } ]
Effect            ::= "pure" | "fs" | "net" | "time" | "crypto" | "env" | "async" | "unsafe"
```

### 2.5 Statements & blocks

```
Block             ::= "{" StmtListOpt "}"
StmtListOpt       ::= [ Stmt { ";" Stmt } [ ";" ] ]          /* final semicolon optional */

Stmt              ::= ReturnStmt
                    | LetStmt
                    | AssignStmt
                    | WhileStmt
                    | IfStmt
                    | MatchStmt
                    | ExprStmt

ReturnStmt        ::= KW_RETURN Expr

LetStmt           ::= KW_LET MutOpt Ident TypeAnnOpt "=" Expr
MutOpt            ::= [ KW_MUT ]
TypeAnnOpt        ::= [ ":" TypeExpr ]

AssignStmt        ::= LValue "=" Expr
LValue            ::= Path                                /* var or qualified field */
                     | LValue "." Ident                   /* nested field */
                     /* (indexing left for future) */

WhileStmt         ::= KW_WHILE Expr Block

IfStmt            ::= KW_IF Expr Block ElseOpt
ElseOpt           ::= [ KW_ELSE ( Block | IfStmt ) ]      /* else-if chain */

MatchStmt         ::= KW_MATCH Expr "{" MatchArms "}"
MatchArms         ::= MatchArm { "," MatchArm } [ "," ]
MatchArm          ::= Pattern "->" ( Expr | Block )

ExprStmt          ::= Expr                                 /* must not be a bare block */
```

### 2.6 Expressions

**Precedence (low → high)**

1. `||`
2. `&&`
3. `==` `!=`
4. `<` `<=` `>` `>=`
5. `+` `-`
6. `*` `/` `%`
7. unary: `-` `!` `KW_AWAIT`
8. postfix: call, field, record/variant init

All operators are left‑associative except unary.

```
Expr              ::= OrExpr

OrExpr            ::= AndExpr { "||" AndExpr }
AndExpr           ::= EqExpr  { "&&" EqExpr }
EqExpr            ::= CmpExpr { ( "==" | "!=" ) CmpExpr }
CmpExpr           ::= AddExpr { ( "<" | "<=" | ">" | ">=" ) AddExpr }
AddExpr           ::= MulExpr { ( "+" | "-" ) MulExpr }
MulExpr           ::= UnaryExpr { ( "*" | "/" | "%" ) UnaryExpr }

UnaryExpr         ::= ( "-" | "!" | KW_AWAIT ) UnaryExpr
                    | PostfixExpr

PostfixExpr       ::= PrimaryExpr { PostfixSuffix }
PostfixSuffix     ::= CallSuffix
                    | FieldSuffix
                    | InitSuffix

CallSuffix        ::= "(" ArgListOpt ")"
ArgListOpt        ::= [ Expr { "," Expr } [ "," ] ]

FieldSuffix       ::= "." Ident

/* Disambiguation rule for InitSuffix:
   - If the first token after "{" is Ident ":" → RecordInit.
   - Else → VariantInit (single payload expression, optional). */
InitSuffix        ::= "{" InitBodyOpt "}"
InitBodyOpt       ::= [ /* RecordInit */ FieldInitList
                      | /* VariantInit */ Expr
                      ]
FieldInitList     ::= FieldInit { "," FieldInit } [ "," ]
FieldInit         ::= Ident ":" Expr

PrimaryExpr       ::= Literal
                    | Path                               /* names and constructors */
                    | "(" Expr ")"
                    | KW_TASK Block                      /* spawn a fiber; yields task handle */

Literal           ::= String
                    | Int [ UIntSuffix ]                 /* e.g., 42U16 */
                    | BoolLit
BoolLit           ::= "true" | "false"
```

### 2.7 Patterns (for `match`)

```
Pattern           ::= "_"
                    | Literal
                    | Ident                               /* bind variable */
                    | Path                                /* match by qualified name (e.g., Ok) */
                      VariantPayloadOpt
                    | "{" PatFieldListOpt "}"             /* structural record pattern */

VariantPayloadOpt ::= [ "{" Pattern "}" ]                 /* Ok{ x } */

PatFieldListOpt   ::= [ PatField { "," PatField } [ "," ] ]
PatField          ::= Ident ":" Pattern
```

---

## 3) Additional rules & clarifications

**3.1 Statement termination**

* Inside `Block`, statements are separated by `;`. The final `;` before `}` is optional.
* `ReturnStmt` does **not** require `;` (and must be the last token of the statement).
* An `ExprStmt` consisting solely of `{ ... }` is **not** permitted; standalone blocks must be preceded by a keyword (e.g., `if`, `while`, `task`).

**3.2 Compact vs relaxed**

* Dual keywords (e.g., `m|module`, `f|fn`, `ret|return`) are interchangeable.
* Parsers should normalize to the canonical long form in the AST, with SymbolMap used only for formatting.

**3.3 Symbol map header**

* Multiple `#sym` headers are allowed; later entries shadow earlier ones for printing only.
* Grammar accepts them at top‑level anywhere; tools typically place one near the header.

**3.4 Imports**

* `only` restricts which identifiers from the import are introduced into the cell’s symbol table; it does not affect the ability to reference items with fully qualified `Path`.
* `as` introduces an alias used in `Path`.

**3.5 Type sums & variants**

* In types, `A | B | C` forms a sum (union) by labels.
* Variant constructors in **expressions** are written as `Label{ payload }`. Records use `TypeOrAlias{ field: expr, ... }`.
* **Disambiguation:** After `Path "{"`, if the next token is `Ident ":"`, parse as a **record literal**; otherwise as a **variant payload**.

**3.6 Numeric literals**

* Plain `Int` is an unsuffixed integer (type‑inferred).
* Suffixes `U16|U32|U64` set the type directly.

**3.7 Effects & capabilities**

* `EffAnnOpt` may appear on `fn` and `extern fn`.
* Capability names in the header follow `CapName` (dotted identifiers).

**3.8 Tasks & await**

* `task { ... }` is an expression producing a task handle (type is implementation‑defined; typically `Task<T>`).
* `await e` is a unary expression that waits on `e` and yields its result.

**3.9 Precedence of init vs call**

* Postfix ordering is **left‑to‑right**. `Path{...}(...)` is parsed as `(Path{...}) ( ... )` which is **invalid** unless the constructor yields a callable. Typical usage is either `Path{...}` or `Path(...)` (function call), not both.

**3.10 Reserved identifiers**

* All keywords (both compact and relaxed forms) and primitive type names are reserved and cannot be used as identifiers.

---

## 4) Minimal valid examples under the grammar

**Compact**

```z1c
m http.server:1 ctx=128 caps=[net]
u "std/http" as H only [listen, Req, Res]
#sym { serve: sv, handler: h, Health: Hl }

t Hl = { ok: Bool, msg: Str }

f h(q:H.Req)->H.Res eff [pure] {
  ret H.Res{ status:200, body:"ok" }
}

f sv(p:U16)->Unit eff [net] {
  H.listen(p, h);
}
```

**Relaxed**

```z1r
module http.server : 1
  ctx = 128
  caps = [net]

use "std/http" as http only [listen, Req, Res]

// SymbolMap: { serve ↔ sv, handler ↔ h, Health ↔ Hl }

type Health = { ok: Bool, msg: Str }

fn handler(q: http.Req) -> http.Res
  eff [pure]
{
  return http.Res{ status: 200, body: "ok" };
}

fn serve(port: U16) -> Unit
  eff [net]
{
  http.listen(port, handler);
}
```

---

## 5) PEG variant (optional)

If you prefer PEG/packrat, the same grammar can be rendered almost verbatim; the only PEG‑specific tweaks are (1) left‑recursion removal (handled by precedence climbing in the parser) and (2) the lookahead for `InitSuffix`:

```
InitSuffix <- "{" ( &Ident ":" FieldInitList / Expr ) "}"
```

---

### Done

This grammar is complete for the specified Z1 MVP (compact + relaxed), covering modules, imports, symbol maps, types (records, sums, generics with where‑bounds), functions (extern & defined), effects, statements, expressions (incl. task/await), patterns/match, and all lexical elements. If you want, I can also generate an ANTLR4 grammar file or a ready‑to‑compile Rust `winnow`/`chumsky` parser that implements exactly this spec.

