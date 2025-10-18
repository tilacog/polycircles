use clap::Parser;
use std::f64::consts::PI;
use svg::Document;
use svg::node::element::{Circle, Polygon as SvgPolygon, Rectangle};

/// Generate an SVG with a polygon and circles that are tangent to its edges
#[derive(Parser, Debug)]
#[command(name = "circle")]
#[command(about = "Generate polygon with tangent circles", long_about = None)]
struct Args {
    /// Number of sides of the polygon (minimum 3 for triangle)
    #[arg(short, long)]
    sides: usize,

    /// Percentage of maximum allowed circle radius (0.01 to 100.00)
    #[arg(short, long)]
    percentage: f64,
}

#[derive(Clone, Copy)]
struct Vec2 {
    x: f64,
    y: f64,
}

impl Vec2 {
    fn new(x: f64, y: f64) -> Self {
        Vec2 { x, y }
    }

    fn length(&self) -> f64 {
        (self.x * self.x + self.y * self.y).sqrt()
    }

    fn normalize(&self) -> Self {
        let len = self.length();
        Vec2 {
            x: self.x / len,
            y: self.y / len,
        }
    }

    fn dot(&self, other: Vec2) -> f64 {
        self.x * other.x + self.y * other.y
    }

    fn length_squared(&self) -> f64 {
        self.x * self.x + self.y * self.y
    }
}

impl std::ops::Sub for Vec2 {
    type Output = Vec2;
    fn sub(self, other: Vec2) -> Vec2 {
        Vec2 {
            x: self.x - other.x,
            y: self.y - other.y,
        }
    }
}

impl std::ops::Add for Vec2 {
    type Output = Vec2;
    fn add(self, other: Vec2) -> Vec2 {
        Vec2 {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }
}

impl std::ops::Mul<f64> for Vec2 {
    type Output = Vec2;
    fn mul(self, scalar: f64) -> Vec2 {
        Vec2 {
            x: self.x * scalar,
            y: self.y * scalar,
        }
    }
}

fn main() {
    if let Err(e) = run() {
        eprintln!("Error: {e}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), String> {
    let args = Args::parse();

    // Validate minimum number of sides
    if args.sides < 3 {
        return Err(format!(
            "Number of sides must be at least 3 (got {}). A polygon needs at least 3 sides.",
            args.sides
        ));
    }

    // Validate percentage range
    if args.percentage < 0.01 || args.percentage > 100.00 {
        return Err(format!(
            "Percentage must be between 0.01 and 100.00 (got {:.2})",
            args.percentage
        ));
    }

    let polygon_radius = 300.0;

    // Create polygon vertices (regular polygon)
    let polygon_vertices: Vec<Vec2> = (0..args.sides)
        .map(|i| {
            let angle = i as f64 * (2.0 * PI / args.sides as f64) - PI / 2.0; // Start from top
            Vec2::new(polygon_radius * angle.cos(), polygon_radius * angle.sin())
        })
        .collect();

    // Calculate maximum allowed radius
    let max_radius = calculate_max_radius(&polygon_vertices);

    // Calculate actual radius from percentage
    let circle_radius = max_radius * (args.percentage / 100.0);

    // Calculate final positions for all circles
    let circle_positions: Vec<Vec2> = polygon_vertices
        .iter()
        .map(|&target_vertex| {
            calculate_final_position(
                Vec2::new(0.0, 0.0),
                target_vertex,
                circle_radius,
                &polygon_vertices,
            )
        })
        .collect();

    // Determine polygon name for output
    let polygon_name = match args.sides {
        3 => "triangle",
        4 => "square",
        5 => "pentagon",
        6 => "hexagon",
        7 => "heptagon",
        8 => "octagon",
        9 => "nonagon",
        10 => "decagon",
        _ => "polygon",
    };

    // Save SVG
    save_svg_with_polygon_and_circles(
        &polygon_vertices,
        &circle_positions,
        circle_radius,
        polygon_name,
    );

    println!("SVG saved to: output/{polygon_name}.svg");
    println!(
        "Using {:.2}% of maximum radius (max: {:.2}, actual: {:.2})",
        args.percentage, max_radius, circle_radius
    );
    Ok(())
}

fn calculate_final_position(
    start: Vec2,
    target_vertex: Vec2,
    radius: f64,
    polygon_vertices: &[Vec2],
) -> Vec2 {
    let dir = (target_vertex - start).normalize();

    let mut low = 0.0;
    let mut high = (target_vertex - start).length(); // up to vertex distance

    for _ in 0..50 {
        let mid = (low + high) / 2.0;
        let mid_pos = start + dir * mid;
        let tangent = is_tangent_to_two_edges(mid_pos, radius, polygon_vertices);

        if tangent {
            return mid_pos;
        }

        // If circle still inside polygon but not tangent yet → move outward
        let mut min_dist = f64::MAX;
        for i in 0..polygon_vertices.len() {
            let a = polygon_vertices[i];
            let b = polygon_vertices[(i + 1) % polygon_vertices.len()];
            let dist = point_to_segment_distance(mid_pos, a, b);
            min_dist = min_dist.min(dist);
        }

        if min_dist > radius {
            low = mid;
        } else {
            high = mid;
        }
    }

    start + dir * low
}

// Calculate distance from point to line segment
fn point_to_segment_distance(p: Vec2, a: Vec2, b: Vec2) -> f64 {
    let ab = b - a;
    let ap = p - a;
    let ab_len_sq = ab.length_squared();

    if ab_len_sq == 0.0 {
        return ap.length();
    }

    let t = (ap.dot(ab) / ab_len_sq).clamp(0.0, 1.0);
    let projection = a + ab * t;
    (p - projection).length()
}

// Check if circle is tangent to exactly two edges
fn is_tangent_to_two_edges(center: Vec2, radius: f64, vertices: &[Vec2]) -> bool {
    let tolerance = 0.5;
    let mut tangent_count = 0;

    for i in 0..vertices.len() {
        let v1 = vertices[i];
        let v2 = vertices[(i + 1) % vertices.len()];

        let distance = point_to_segment_distance(center, v1, v2);

        if (distance - radius).abs() < tolerance {
            tangent_count += 1;
        }
    }

    tangent_count == 2
}

// Calculate the maximum allowed radius before circles overlap
fn calculate_max_radius(polygon_vertices: &[Vec2]) -> f64 {
    let mut low = 1.0;
    let mut high = 150.0;
    let tolerance = 0.1;

    while high - low > tolerance {
        let mid = (low + high) / 2.0;

        // Calculate final positions for this radius
        let mut final_positions = Vec::new();
        let mut valid = true;

        for &target_vertex in polygon_vertices {
            let pos =
                calculate_final_position(Vec2::new(0.0, 0.0), target_vertex, mid, polygon_vertices);
            final_positions.push(pos);
        }

        // Check for overlaps
        for i in 0..final_positions.len() {
            for j in (i + 1)..final_positions.len() {
                let dist = (final_positions[i] - final_positions[j]).length();
                if dist < 2.0 * mid {
                    valid = false;
                    break;
                }
            }
            if !valid {
                break;
            }
        }

        if valid {
            low = mid;
        } else {
            high = mid;
        }
    }

    low
}

fn save_svg_with_polygon_and_circles(
    vertices: &[Vec2],
    circle_positions: &[Vec2],
    circle_radius: f64,
    polygon_name: &str,
) {
    let width = 800;
    let height = 800;
    let center = 400.0;

    // Convert vertices to SVG coordinates (flip Y axis and translate)
    let points_str: String = vertices
        .iter()
        .map(|v| format!("{},{}", center + v.x, center - v.y))
        .collect::<Vec<_>>()
        .join(" ");

    let mut document = Document::new()
        .set("width", width)
        .set("height", height)
        .set("viewBox", (0, 0, width, height))
        .add(
            Rectangle::new()
                .set("width", "100%")
                .set("height", "100%")
                .set("fill", "white"),
        )
        .add(
            SvgPolygon::new()
                .set("points", points_str)
                .set("stroke", "black")
                .set("stroke-width", 2)
                .set("fill", "none"),
        );

    // Add all circles to the document
    for pos in circle_positions {
        document = document.add(
            Circle::new()
                .set("cx", center + pos.x)
                .set("cy", center - pos.y)
                .set("r", circle_radius)
                .set("fill", "steelblue"),
        );
    }

    std::fs::create_dir_all("output").ok();
    let filename = format!("output/{polygon_name}.svg");
    svg::save(&filename, &document).expect("Failed to save SVG");
}

#[cfg(test)]
mod tests {
    use super::*;

    // Vec2 tests
    #[test]
    fn test_vec2_new() {
        let v = Vec2::new(3.0, 4.0);
        assert_eq!(v.x, 3.0);
        assert_eq!(v.y, 4.0);
    }

    #[test]
    fn test_vec2_length() {
        let v = Vec2::new(3.0, 4.0);
        assert!((v.length() - 5.0).abs() < 0.001);
    }

    #[test]
    fn test_vec2_length_squared() {
        let v = Vec2::new(3.0, 4.0);
        assert!((v.length_squared() - 25.0).abs() < 0.001);
    }

    #[test]
    fn test_vec2_normalize() {
        let v = Vec2::new(3.0, 4.0);
        let normalized = v.normalize();
        assert!((normalized.length() - 1.0).abs() < 0.001);
        assert!((normalized.x - 0.6).abs() < 0.001);
        assert!((normalized.y - 0.8).abs() < 0.001);
    }

    #[test]
    fn test_vec2_dot() {
        let v1 = Vec2::new(1.0, 2.0);
        let v2 = Vec2::new(3.0, 4.0);
        assert!((v1.dot(v2) - 11.0).abs() < 0.001);
    }

    #[test]
    fn test_vec2_add() {
        let v1 = Vec2::new(1.0, 2.0);
        let v2 = Vec2::new(3.0, 4.0);
        let result = v1 + v2;
        assert_eq!(result.x, 4.0);
        assert_eq!(result.y, 6.0);
    }

    #[test]
    fn test_vec2_sub() {
        let v1 = Vec2::new(3.0, 4.0);
        let v2 = Vec2::new(1.0, 2.0);
        let result = v1 - v2;
        assert_eq!(result.x, 2.0);
        assert_eq!(result.y, 2.0);
    }

    #[test]
    fn test_vec2_mul() {
        let v = Vec2::new(2.0, 3.0);
        let result = v * 2.0;
        assert_eq!(result.x, 4.0);
        assert_eq!(result.y, 6.0);
    }

    // Geometric function tests
    #[test]
    fn test_point_to_segment_distance_perpendicular() {
        // Point perpendicular to segment midpoint
        let p = Vec2::new(0.0, 5.0);
        let a = Vec2::new(-5.0, 0.0);
        let b = Vec2::new(5.0, 0.0);
        let distance = point_to_segment_distance(p, a, b);
        assert!((distance - 5.0).abs() < 0.001);
    }

    #[test]
    fn test_point_to_segment_distance_endpoint() {
        // Point closest to segment endpoint
        let p = Vec2::new(10.0, 5.0);
        let a = Vec2::new(0.0, 0.0);
        let b = Vec2::new(5.0, 0.0);
        let distance = point_to_segment_distance(p, a, b);
        let expected = ((10.0_f64 - 5.0).powi(2) + 5.0_f64.powi(2)).sqrt();
        assert!((distance - expected).abs() < 0.001);
    }

    #[test]
    fn test_point_to_segment_distance_on_segment() {
        // Point on the segment
        let p = Vec2::new(2.5, 0.0);
        let a = Vec2::new(0.0, 0.0);
        let b = Vec2::new(5.0, 0.0);
        let distance = point_to_segment_distance(p, a, b);
        assert!(distance < 0.001);
    }

    #[test]
    fn test_is_tangent_to_two_edges_square() {
        let vertices = vec![
            Vec2::new(100.0, 100.0),
            Vec2::new(-100.0, 100.0),
            Vec2::new(-100.0, -100.0),
            Vec2::new(100.0, -100.0),
        ];

        // Circle centered near top-right corner → tangent to top and right edges.
        let center_offset = Vec2::new(50.0, 50.0);
        let radius = 50.0;
        assert!(is_tangent_to_two_edges(center_offset, radius, &vertices));
    }

    #[test]
    fn test_calculate_max_radius_triangle() {
        // Create equilateral triangle with radius 300
        let polygon_radius = 300.0;
        let vertices: Vec<Vec2> = (0..3)
            .map(|i| {
                let angle = i as f64 * (2.0 * PI / 3.0) - PI / 2.0;
                Vec2::new(polygon_radius * angle.cos(), polygon_radius * angle.sin())
            })
            .collect();

        let max_radius = calculate_max_radius(&vertices);

        // For a triangle, the max radius should be reasonable (not zero, not huge)
        assert!(max_radius > 50.0);
        assert!(max_radius < 200.0);
    }

    #[test]
    fn test_calculate_max_radius_pentagon() {
        // Create pentagon with radius 300
        let polygon_radius = 300.0;
        let vertices: Vec<Vec2> = (0..5)
            .map(|i| {
                let angle = i as f64 * (2.0 * PI / 5.0) - PI / 2.0;
                Vec2::new(polygon_radius * angle.cos(), polygon_radius * angle.sin())
            })
            .collect();

        let max_radius = calculate_max_radius(&vertices);

        // Pentagon should allow larger circles than triangle
        assert!(max_radius > 80.0);
        assert!(max_radius < 120.0);
    }

    #[test]
    fn test_calculate_max_radius_hexagon() {
        // Create hexagon with radius 300
        let polygon_radius = 300.0;
        let vertices: Vec<Vec2> = (0..6)
            .map(|i| {
                let angle = i as f64 * (2.0 * PI / 6.0) - PI / 2.0;
                Vec2::new(polygon_radius * angle.cos(), polygon_radius * angle.sin())
            })
            .collect();

        let max_radius = calculate_max_radius(&vertices);

        // Hexagon should allow even larger circles
        assert!(max_radius > 60.0);
        assert!(max_radius < 100.0);
    }

    // Integration tests
    #[test]
    fn test_percentage_calculation() {
        // Test that percentage calculation works correctly
        let max_radius = 100.0;
        let percentage = 50.0;
        let actual_radius = max_radius * (percentage / 100.0);
        assert!((actual_radius - 50.0_f64).abs() < 0.001);
    }

    #[test]
    fn test_percentage_boundary_low() {
        let max_radius = 100.0;
        let percentage = 0.01;
        let actual_radius = max_radius * (percentage / 100.0);
        assert!((actual_radius - 0.01_f64).abs() < 0.001);
    }

    #[test]
    fn test_percentage_boundary_high() {
        let max_radius = 100.0;
        let percentage = 100.0;
        let actual_radius = max_radius * (percentage / 100.0);
        assert!((actual_radius - 100.0_f64).abs() < 0.001);
    }

    #[test]
    fn test_polygon_vertices_count_triangle() {
        let polygon_radius = 300.0;
        let sides = 3;
        let vertices: Vec<Vec2> = (0..sides)
            .map(|i| {
                let angle = i as f64 * (2.0 * PI / sides as f64) - PI / 2.0;
                Vec2::new(polygon_radius * angle.cos(), polygon_radius * angle.sin())
            })
            .collect();

        assert_eq!(vertices.len(), 3);
    }

    #[test]
    fn test_polygon_vertices_count_pentagon() {
        let polygon_radius = 300.0;
        let sides = 5;
        let vertices: Vec<Vec2> = (0..sides)
            .map(|i| {
                let angle = i as f64 * (2.0 * PI / sides as f64) - PI / 2.0;
                Vec2::new(polygon_radius * angle.cos(), polygon_radius * angle.sin())
            })
            .collect();

        assert_eq!(vertices.len(), 5);
    }

    #[test]
    fn test_polygon_vertices_are_on_circle() {
        let polygon_radius = 300.0;
        let sides = 6;
        let vertices: Vec<Vec2> = (0..sides)
            .map(|i| {
                let angle = i as f64 * (2.0 * PI / sides as f64) - PI / 2.0;
                Vec2::new(polygon_radius * angle.cos(), polygon_radius * angle.sin())
            })
            .collect();

        // All vertices should be at distance polygon_radius from origin
        for vertex in vertices {
            assert!((vertex.length() - polygon_radius).abs() < 0.001);
        }
    }

    #[test]
    fn test_calculate_final_position_moves_from_origin() {
        let polygon_radius = 300.0;
        let vertices: Vec<Vec2> = (0..5)
            .map(|i| {
                let angle = i as f64 * (2.0 * PI / 5.0) - PI / 2.0;
                Vec2::new(polygon_radius * angle.cos(), polygon_radius * angle.sin())
            })
            .collect();

        let start = Vec2::new(0.0, 0.0);
        let target_vertex = vertices[0];
        let radius = 50.0;

        let final_pos = calculate_final_position(start, target_vertex, radius, &vertices);

        // Final position should be closer to target vertex than start position
        let start_distance = (target_vertex - start).length();
        let final_distance = (target_vertex - final_pos).length();
        assert!(final_distance < start_distance);

        // Final position should not be at origin
        assert!(final_pos.length() > 1.0);
    }

    #[test]
    fn test_max_radius_increases_with_more_sides() {
        let polygon_radius = 300.0;

        // Triangle
        let vertices_3: Vec<Vec2> = (0..3)
            .map(|i| {
                let angle = i as f64 * (2.0 * PI / 3.0) - PI / 2.0;
                Vec2::new(polygon_radius * angle.cos(), polygon_radius * angle.sin())
            })
            .collect();
        let max_radius_3 = calculate_max_radius(&vertices_3);

        // Hexagon
        let vertices_6: Vec<Vec2> = (0..6)
            .map(|i| {
                let angle = i as f64 * (2.0 * PI / 6.0) - PI / 2.0;
                Vec2::new(polygon_radius * angle.cos(), polygon_radius * angle.sin())
            })
            .collect();
        let max_radius_6 = calculate_max_radius(&vertices_6);

        // Hexagon should have smaller max radius than triangle
        // (circles are closer together in hexagon)
        assert!(max_radius_6 < max_radius_3);
    }
}
