# Lumina - Frontend Developer Guide

## Introduction

Lumina is a multidimensional spreadsheet engine. This guide is for developers building frontends (web, desktop, mobile) on top of Lumina.

## Core Concepts

### Sparse Representation & Empty Cells

**Important**: Lumina uses a **sparse container model**. The spreadsheet is conceptually infinite, but only non-empty cells are stored.

- **Storage**: Empty cells don't exist in the container's HashMap
- **Numeric operations**: When iterating a range, empty cells behave as `0.0` (the default Cell value)
- **String operations**: Empty cells behave as empty string `""`
- **Detection**: Use `container.is_empty_cell(position)` to check if a cell was explicitly set

This means functions like `SUM`, `AVERAGE`, `MIN`, `MAX`, `PRODUCT` iterate over **all positions in the range** (including empty ones), and empty cells contribute `0.0` to numeric calculations. This matches spreadsheet behavior where `=SUM(A1:A10)` includes empty cells as zeros.

### Multidimensional Grid

Unlike traditional 2D spreadsheets, Lumina supports **N-dimensional grids**:
- **1D**: Simple list `[x]`
- **2D**: Traditional rows/columns `[row, col]`
- **3D**: Sheets/Pages `[sheet, row, col]`
- **4D+**: Workbooks/Sections `[book, sheet, row, col]`, etc.

Positions are `Vec<usize>` - zero-indexed coordinates.

### Cell Structure

Each cell contains:
```rust
struct Cell {
    content: Content,        // Actual data
    display_content: String, // What to show (includes errors)
    properties: Properties,  // Custom metadata
}
```

#### Content Types
| Type | Description | Example |
|------|-------------|---------|
| `Number` | 64-bit float | `42.5`, `-3.14` |
| `Str` | UTF-8 string | `"Hello"`, `"$1,234"` |
| `Formula` | Compiled formula | `=SUM(A1:A10)` |

#### Properties
Free-form string metadata per cell:
```rust
cell.properties.inner = "style:bold;color:red".to_string();
```
Use for: formatting, validation rules, comments, custom data.

### Formulas

Formulas **must start with `=`** and support:

#### Literals
- Numbers: `42`, `3.14`, `-5`
- Strings: `"text"`, `"He said \"hi\""` (escape with `\"`)
- Ranges: `{0,0}:{2,2}` (3x3 grid from origin)

#### Operators (precedence order)
1. Parentheses: `(expr)`
2. Unary minus: `-A1`
3. Multiplicative: `*`, `/`, `%`
4. Additive: `+`, `-`

#### Function Calls
```
=SUM({0,0}:{2,2})
=IF(A1>0, "Positive", "Negative")
=XLOOKUP("search", {0,0}:{0,9}, {1,0}:{1,9})
```

#### Range Syntax
| Syntax | Description |
|--------|-------------|
| `{0,0}` | Single cell at position [0,0] |
| `{0,0}:{2,2}` | Rectangular range from [0,0] to [2,2] |
| `{0,0};{1,1};{2,2}` | Non-contiguous cells |

## Built-in Functions Reference

### Math Functions
| Function | Args | Description |
|----------|------|-------------|
| `SUM(range)` | 1 range | Sum of all numbers in range |
| `AVERAGE(range)` | 1 range | Arithmetic mean |
| `PRODUCT(range)` | 1 range | Product of all numbers |
| `MIN(range)` | 1 range | Minimum value |
| `MAX(range)` | 1 range | Maximum value |
| `ABS(range)` | 1 range | Absolute value of each cell |
| `CEILING(range)` | 1 range | Ceiling of each cell |
| `FLOOR(range)` | 1 range | Floor of each cell |
| `ROUND(range, digits)` | 1 range, 1 number | Round to decimal places |
| `ROUNDUP(range, digits)` | 1 range, 1 number | Round up |
| `ROUNDDOWN(range, digits)` | 1 range, 1 number | Round down |
| `INT(range)` | 1 range | Integer part (truncate) |
| `POWER(range, exp)` | 1 range, 1 number | Raise each to power |
| `SQRT(range, root)` | 1 range, 1 number | Nth root (default 2) |

### String Functions
| Function | Args | Description |
|----------|------|-------------|
| `CONCAT(range)` | 1 range | Concatenate all cells (skips empty) |
| `LEFT(text, len?)` | 1 str, 1 opt num | Left N chars (default 1) |
| `RIGHT(text, len?)` | 1 str, 1 opt num | Right N chars (default 1) |
| `MID(text, len?)` | 1 str, 1 opt num | Middle N chars (default 1) |
| `LEN(text)` | 1 str | String length |
| `TRIM(text)` | 1 str | Trim whitespace |
| `UPPER(text)` | 1 str | Uppercase |
| `LOWER(text)` | 1 str | Lowercase |
| `SUBSTITUTE(text, old, new)` | 3 str | Replace all occurrences |
| `TOSTRING(number)` | 1 number | Convert to string |
| `TONUMBER(text)` | 1 str | Parse number from string |

### Logical Functions
| Function | Args | Description |
|----------|------|-------------|
| `IF(condition, true_val, false_val)` | 3 any | Ternary conditional |
| `AND(...)` | 1+ bool | Logical AND |
| `OR(...)` | 1+ bool | Logical OR |
| `NOT(value)` | 1 bool | Logical NOT |
| `IFERROR(formula_str, fallback)` | 1 str, 1 any | Catch errors in formula |
| `ISERROR(range)` | 1 range | Check if any cell is error |
| `ISBLANK(range)` | 1 range | Count empty cells |
| `ISNUMBER(range)` | 1 range | Check if all numbers |
| `ISTEXT(range)` | 1 range | Check if all strings |

### Lookup Functions
| Function | Args | Description |
|----------|------|-------------|
| `XLOOKUP(lookup, lookup_range, result_range)` | 1 any, 2 ranges | Find value, return corresponding |
| `INDEX(range, pos1, pos2, ...)` | 1 range, N nums | Get cell at relative position |
| `MATCH(lookup, range)` | 1 any, 1 range | Find position of value |

### Statistical Functions
| Function | Args | Description |
|----------|------|-------------|
| `COUNT(range)` | 1 range | Count numeric cells |
| `COUNTA(range)` | 1 range | Count non-empty cells |
| `COUNTBLANK(range)` | 1 range | Count empty cells |
| `COUNTIF(range, criteria)` | 1 range, 1 str/num | Count matching criteria |
| `SUMIF(range, criteria)` | 1 range, 1 str/num | Sum matching criteria |
| `AVERAGEIF(range, criteria)` | 1 range, 1 str/num | Average matching criteria |

### Data Functions
| Function | Args | Description |
|----------|------|-------------|
| `SORT(range)` | 1 range | Sort range values |
| `UNIQUE(range)` | 1 range | Unique values |
| `TORANGE(number)` | 1 number | Convert to single-cell range |

### Time Functions
| Function | Args | Description |
|----------|------|-------------|
| `NOW()` | 0 | Current UNIX timestamp |

## Error Handling

Errors appear in `cell.display_content` prefixed with `"ERROR: "`:

```
ERROR: SUM requires a range as its first argument
ERROR: Division by zero
ERROR: XLOOKUP value not found
ERROR: First argument to LEFT must be a string
```

Common error types:
- **SyntaxError**: Invalid formula syntax
- **TypeError**: Wrong argument type
- **LengthError**: Wrong number of arguments
- **RuntimeError**: Execution failure (e.g., not found)
- **BoundsError**: Position out of range

## API Usage

### Basic Setup
```rust
use lumina::{MapContainer, Cell, Content, PositionN};
use lumina::formula::{FormulaParser, FormulaExecutor};

let container = MapContainer::new(3);  // 3D: [sheet, row, col]
let parser = FormulaParser::new();
let executor = FormulaExecutor::new();
```

### Inserting Data
```rust
// Insert a number
let mut cell = Cell::new();
cell.content = Content::Number(42.0);
container.insert(&vec![0, 5, 3], cell)?;

// Insert a string
let mut cell = Cell::new();
cell.content = Content::Str("Hello".to_string());
container.insert(&vec![0, 5, 4], cell)?;

// Insert a formula
let ast = parser.parse("=SUM({0,0}:{0,9})")?;
let mut cell = Cell::new();
cell.content = Content::Formula(ast);
container.insert(&vec![0, 0, 0], cell)?;
```

### Evaluating Formulas
```rust
// Execute formula at position
let ast = parser.parse("=SUM({0,0}:{0,9})")?;
executor.execute(ast, &vec![0, 0, 0], &container)?;

// Get result
let cell = container.get_cell_at(&vec![0, 0, 0])?;
println!("Display: {}", cell.display_content);
```

### Working with Ranges
```rust
// Get a 2D slice (all sheets, row 5, all cols)
let cells = container.get_cells_at_axis(&[
    AnyPosition::Any,      // All sheets
    AnyPosition::Position(5), // Row 5
    AnyPosition::Any,      // All columns
])?;

// Get specific cells
let cells = container.get_cells_at(&[
    vec![0, 0, 0],
    vec![0, 1, 1],
    vec![1, 0, 0],
])?;
```

### Dynamic Dimensions
```rust
// Add a dimension (e.g., add "workbook" axis)
container.set_axes(4)?;  // Now [book, sheet, row, col]

// Remove a dimension (keep only coordinate 0)
container.remove_axis(0, Some(0))?;
```

### Persistence
```rust
use serde_json;

// Save
let json = serde_json::to_string(&container)?;
// Save to file...

// Load
let container: MapContainer = serde_json::from_str(&json)?;
```

## Frontend Integration Patterns

### 1. Cell Rendering
```javascript
// Pseudo-code for frontend
function renderCell(cell) {
    if (cell.display_content.startsWith("ERROR: ")) {
        return <span class="error">{cell.display_content}</span>;
    }
    if (cell.content.type === "Formula") {
        return <span class="formula">{cell.display_content}</span>;
    }
    return <span>{cell.display_content}</span>;
}
```

### 2. Formula Editor
- Show raw formula when editing: `=SUM(A1:A10)`
- Show result when not editing: `42.5`
- Parse on `=` prefix, otherwise treat as literal

### 3. Range Selection
- Map UI selection to `PositionN` arrays
- Support multi-dimensional selection UI
- Convert to `{origin}:{dest}` for formulas

### 4. Real-time Updates
- Re-evaluate affected formulas on cell change
- Track dependencies (which cells reference which)
- Use `FormulaExecutor` for batch evaluation

### 5. Undo/Redo
- Serialize container state for history
- Use `serde_json` for snapshots

## Best Practices

1. **Batch Operations**: Group multiple inserts before executing formulas
2. **Lazy Evaluation**: Only recalculate visible cells
3. **Dimension Management**: Set axes once at startup; avoid frequent changes
4. **Error Display**: Show `display_content` for user-facing values
5. **Properties**: Store UI state (selection, formatting) in `properties.inner`

## Example: Simple 2D Spreadsheet

```rust
let sheet = MapContainer::new(2);  // [row, col]

// Header row
for col in 0..10 {
    let mut cell = Cell::new();
    cell.content = Content::Str(format!("Col {}", col));
    sheet.insert(&vec![0, col], cell)?;
}

// Data
for row in 1..100 {
    for col in 0..10 {
        let mut cell = Cell::new();
        cell.content = Content::Number((row * col) as f64);
        sheet.insert(&vec![row, col], cell)?;
    }
}

// Total row
let ast = parser.parse("=SUM({1,0}:{99,0})")?;
let mut cell = Cell::new();
cell.content = Content::Formula(ast);
sheet.insert(&vec![100, 0], cell)?;

// Execute
executor.execute(ast, &vec![100, 0], &sheet)?;
```

## Migration from 2D Spreadsheets

| 2D Concept | Lumina Equivalent |
|------------|-------------------|
| `A1` | `{0,0}` |
| `A1:B10` | `{0,0}:{9,1}` |
| Sheet | Axis 0 (e.g., `[sheet, row, col]`) |
| Workbook | Axis 0 + Axis 1 (e.g., `[book, sheet, row, col]`) |
| Named ranges | Store mapping in Properties or external map |

## Performance Tips

- Use `get_cells_at_axis` for rendering entire rows/columns
- Cache parsed formulas (AST) to avoid re-parsing
- Limit evaluated range size for large sheets
- Use `is_empty_cell` for sparse data checks

## Version Compatibility

- `FormulaAst` is versioned via serde
- Container format is stable
- New functions added without breaking changes