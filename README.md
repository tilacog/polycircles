# Polygon Circle Generator

A command-line tool that generates SVG images of regular polygons (triangle, square, pentagon, hexagon, etc.) with circles positioned tangent to their edges.

## Overview

This tool creates an SVG file containing:
- A regular polygon with any number of sides (minimum 3)
- Circles for each vertex, each moving from the center toward that vertex
- Each circle stops when it becomes tangent to exactly two edges of the polygon
- Automatic validation to prevent circle overlaps

## Installation

```bash
cargo build --release
```

## Usage

Generate an SVG with a polygon of specified sides and circles sized as a percentage of the maximum allowed radius:

```bash
cargo run -- --sides <SIDES> --percentage <PERCENTAGE>
```

The percentage must be between 0.01 and 100.00, where 100% uses the maximum circle size before overlaps occur.

### Examples

**Create a triangle with circles at 50% of maximum size:**
```bash
cargo run -- --sides 3 --percentage 50.0
```

**Create a square with circles at 75% of maximum size:**
```bash
cargo run -- --sides 4 --percentage 75.0
```

**Create a pentagon with circles at 100% of maximum size:**
```bash
cargo run -- --sides 5 --percentage 100.0
```

**Create a hexagon with circles at 25% of maximum size:**
```bash
cargo run -- --sides 6 --percentage 25.0
```

**Using short flags:**
```bash
cargo run -- -s 8 -p 80.0
```

**Get help:**
```bash
cargo run -- --help
```

## Output

The generated SVG file is saved to `output/<polygon-name>.svg`, where the filename is based on the number of sides:
- 3 sides → `triangle.svg`
- 4 sides → `square.svg`
- 5 sides → `pentagon.svg`
- 6 sides → `hexagon.svg`
- 7 sides → `heptagon.svg`
- 8 sides → `octagon.svg`
- 9 sides → `nonagon.svg`
- 10 sides → `decagon.svg`
- 11+ sides → `polygon.svg`

The SVG contains:
- Canvas size: 800x800
- Polygon radius: 300 units
- Steelblue circles (one per vertex) positioned at their tangent points

## Validation

The tool automatically calculates the maximum allowed circle radius for any polygon, then uses your percentage to determine the actual size. If you specify a percentage outside the valid range (0.01 to 100.00), you'll get an error:

```bash
$ cargo run -- --sides 5 --percentage 150.0

Error: Percentage must be between 0.01 and 100.00 (got 150.00)
```

When you run the tool successfully, it will display the maximum allowed radius and the actual radius used:

```bash
$ cargo run -- --sides 5 --percentage 50.0

SVG saved to: output/pentagon.svg
Using 50.00% of maximum radius (max: 102.20, actual: 51.10)
```

**Note:** The maximum allowed radius varies by the number of sides. More sides generally allow larger circle radii.

## How It Works

1. **Polygon Generation**: Creates a regular polygon with the specified number of vertices (minimum 3)
2. **Circle Placement**: For each vertex, simulates a circle moving from the center toward that vertex
3. **Tangency Detection**: Uses point-to-line-segment distance to detect when a circle is tangent to exactly two edges
4. **Overlap Validation**: Checks all pairs of circles to ensure they don't overlap
5. **SVG Export**: Generates a clean SVG file with the polygon and all circles

## Technical Details

- **Language**: Rust
- **Dependencies**:
  - `clap` - CLI argument parsing
  - `svg` - SVG generation
- **Algorithm**: Iterative position calculation with geometric tangency detection
- **Validation**: Binary search to calculate maximum allowed radius

## Error Handling

The CLI returns proper exit codes:
- `0` - Success, SVG generated
- `1` - Error (invalid number of sides, invalid percentage, missing arguments, etc.)

Errors include:
- Number of sides less than 3
- Percentage outside valid range (0.01 to 100.00)

## License

This project is part of a scratch workspace for experimental code.
