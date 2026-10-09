# Lumina - Developer Documentation

## Overview

Lumina is a multidimensional spreadsheet engine written in Rust. It provides a flexible, high-performance backend for spreadsheet applications with support for N-dimensional grids, formula parsing and execution, and a rich set of built-in functions.

## Architecture

### Core Modules

```
src/
├── lib.rs              # Public API exports, core types (Cell, Content, PositionN, Color, Properties)
├── container/          # Container trait and implementations
│   ├── mod.rs          # Container trait, AnyPosition enum, ContainerError
│   └── mapcontainer.rs # MapContainer - HashMap-based implementation
├── formula/            # Formula parsing and execution
│   ├── mod.rs          # FormulaError, FormulaAst, vec_sub utility
│   ├── parser.rs       # FormulaParser using chumsky parser combinators
│   ├── exec.rs         # FormulaExecutor for AST evaluation
│   └── builtins/       # Built-in spreadsheet functions
│       ├── mod.rs      # Function registry, comparison utilities
│       └── *.rs        # Individual function implementations
└── io/                 # Import/export functionality
    └── mod.rs          # Serialization support
```

### Key Types

#### Sparse Representation & Empty Cells

A fundamental design concept in Lumina is the **sparse container model**. The spreadsheet is conceptually infinite (or very large), but only non-empty cells are actually stored. This enables "almost-infinite" spreadsheets without memory overhead.

**How empty cells work:**
- **Storage**: Empty cells don't exist in the container's HashMap
- **Numeric operations**: When iterating a range, empty cells return `Content::Number(0.0)` (the default Cell value)
- **String operations**: Empty cells return empty string
- **Detection**: Use `container.is_empty_cell(position)` to check if a cell was explicitly set

This means functions like SUM, AVERAGE, MIN, MAX, PRODUCT iterate over **all positions in the range** (including empty ones), and empty cells contribute `0.0` to numeric calculations. This matches spreadsheet behavior where `=SUM(A1:A10)` includes empty cells as zeros.

#### PositionN
```rust
pub type PositionN = Vec<usize>;
```
N-dimensional coordinate representing a cell position. A 3D spreadsheet uses `vec![x, y, z]`, 2D uses `vec![row, col]`, etc.

#### Content
```rust
pub enum Content {
    Formula(FormulaAst),  // Compiled formula AST
    Number(f64),          // Numeric value
    Str(String),          // Text value
}
```

#### Cell
```rust
pub struct Cell {
    content: Content,           // Actual cell content
    display_content: String,    // Cached display string (including errors)
    properties: Properties,     // User-defined properties
}
```

#### FormulaAst
Abstract Syntax Tree for formulas:
```rust
pub enum FormulaAst {
    Range(Vec<PositionN>),      // Cell range reference
    Number(f64),                // Numeric literal
    Neg(Box<FormulaAst>),       // Unary negation
    Add(Box<FormulaAst>, Box<FormulaAst>),
    Sub(Box<FormulaAst>, Box<FormulaAst>),
    Mul(Box<FormulaAst>, Box<FormulaAst>),
    Div(Box<FormulaAst>, Box<FormulaAst>),
    Mod(Box<FormulaAst>, Box<FormulaAst>),
    Str(String),                // String literal
    MacroCall(String, Vec<Self>), // Function call with arguments
}
```

#### FormulaError
```rust
pub enum FormulaError {
    TypeError(String),       // Type mismatch
    SyntaxError(String),     // Parse error
    BoundsError(String),     // Out of bounds
    ContainerError(ContainerError),
    OverlapError(String),    // Range overlap
    WrongEvaluation(String), // Partial result
    MacroDoesNotExist(String), // Unknown function
    LengthError(String),     // Wrong argument count
    RuntimeError(String),    // Execution error
}
```

### Container Trait

All containers implement the `Container` trait:

```rust
pub trait Container : Iterator<Item = (Vec<usize>, Cell)> {
    fn insert(&self, p: &PositionN, c: Cell) -> Result<(), Box<dyn Error>>;
    fn remove(&self, p: &PositionN) -> Result<(), Box<dyn Error>>;
    fn get_cells_at_axis(&self, p: &[AnyPosition]) -> Result<Vec<Cell>, Box<dyn Error>>;
    fn get_cell_at(&self, p: &PositionN) -> Result<Option<Cell>, Box<dyn Error>>;
    fn get_cells_at(&self, p: &[PositionN]) -> Result<Vec<Cell>, Box<dyn Error>>;
    fn is_empty_cell(&self, p: &PositionN) -> bool;
    fn set_cell_at(&self, p: &PositionN, nc: Cell) -> Result<(), Box<dyn Error>>;
    fn set_axes(&mut self, naxes: usize) -> Result<(), Box<dyn Error>>;
    fn get_axes(&self) -> usize;
    fn get_n_cells(&self) -> usize;
    fn remove_axis(&mut self, axispos: usize, retain_coord: Option<usize>) -> Result<(), Box<dyn Error>>;
}
```

#### AnyPosition
Used for axis queries:
```rust
pub enum AnyPosition {
    Any,              // Match any position on this axis
    Position(usize),  // Match specific position
}
```

### MapContainer

Thread-safe HashMap-based container using `Arc<RwLock<InnerContainer>>`:
- `InnerContainer` holds `HashMap<PositionN, Cell>` and axis count
- Supports dynamic axis resizing (`set_axes`)
- Supports axis removal (`remove_axis`)
- Implements `Iterator` for full traversal
- Implements `Serialize`/`Deserialize` for persistence

### Formula System

#### Parser (FormulaParser)
Uses chumsky parser combinator library. Grammar:
- **Positions**: `{x,y,z}` for single cell, `{x1,y1,z1}:{x2,y2,z2}` for ranges
- **Numbers**: Integer and decimal literals
- **Strings**: Double-quoted with escape support (`\"`, `\\`)
- **Operators**: `+`, `-`, `*`, `/`, `%` with precedence
- **Function calls**: `FUNCTION(arg1, arg2, ...)`
- **Parentheses**: Grouping with `(expr)`

#### Executor (FormulaExecutor)
Evaluates `FormulaAst` against a `Container`:
- Recursively evaluates AST nodes
- Handles range operations by iterating cells
- Executes built-in functions (macros)
- Caches results in cell's `display_content`

### Built-in Functions

All functions in `src/formula/builtins/` follow this signature:
```rust
pub fn function_name(
    args: Vec<FormulaAstInner>,
    container: &dyn Container,
    exec: &FormulaExecutor
) -> Result<FormulaAstInner, Box<dyn Error>>
```

Key functions:
- **Math**: SUM, AVERAGE, PRODUCT, MIN, MAX, ABS, CEILING, FLOOR, ROUND, ROUNDUP, ROUNDDOWN, INT, POWER, SQRT
- **Trigonometric**: (planned)
- **Logarithmic**: (planned - LN, LOG)
- **String**: CONCAT, LEFT, RIGHT, MID, LEN, TRIM, UPPER, LOWER, SUBSTITUTE, TOSTRING, TONUMBER
- **Logical**: IF, AND, OR, NOT, IFERROR, ISERROR, ISBLANK, ISNUMBER, ISTEXT
- **Lookup**: XLOOKUP, INDEX, MATCH
- **Statistical**: COUNT, COUNTA, COUNTBLANK, COUNTIF, SUMIF, AVERAGEIF
- **Data**: SORT, UNIQUE, TORANGE
- **Time**: NOW

### Adding New Functions

1. Create `src/formula/builtins/newfunc.rs`
2. Implement the function signature
3. Add module declaration in `src/formula/builtins/mod.rs`
4. Register in the macro map in `mod.rs`
5. Add tests in the module's `#[cfg(test)]` section

### Error Handling

Errors propagate through `Result<FormulaAstInner, Box<dyn Error>>`:
- Parse errors: `FormulaError::SyntaxError`
- Type errors: `FormulaError::TypeError`
- Argument count: `FormulaError::LengthError`
- Runtime: `FormulaError::RuntimeError`
- Container errors: `FormulaError::ContainerError`

Errors are stored in cell's `display_content` with prefix `"ERROR: "` for display.

### Testing

Run tests with:
```bash
cargo test
```

Run specific module tests:
```bash
cargo test formula::builtins::sum
```

Run examples:
```bash
cargo run --example test_errors
```

### Serialization

`MapContainer` implements `Serialize`/`Deserialize` via serde. Cell content including formulas can be persisted and restored.

### Concurrency

- `MapContainer` uses `RwLock` for thread-safe access
- Multiple readers or single writer
- `FormulaExecutor` is stateless and can be shared

## Development Guidelines

1. **Error Messages**: Make them specific to the function and expected types
2. **Performance**: Use `Vec::with_capacity` for range operations
3. **Testing**: Each function should have tests for empty ranges, partial data, full data, and error cases
4. **Documentation**: Add doc comments for public APIs
5. **Types**: Prefer specific types over `Box<dyn Error>` where possible