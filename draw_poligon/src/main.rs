use raylib::prelude::*;

#[derive(Clone, Debug)]
struct Vertex {
    x: f32,
    y: f32,
}

impl Vertex {
    fn new(x: i32, y: i32, height: i32) -> Self {
        Vertex {
            x: x as f32,
            y: (height - y) as f32, // invertir eje y para imagen
        }
    }
}

fn is_convex(a: &Vertex, b: &Vertex, c: &Vertex) -> bool {
    let cross = (b.x - a.x)*(c.y - a.y) - (b.y - a.y)*(c.x - a.x);
    cross < 0.0
}

fn point_in_triangle(p: &Vertex, a: &Vertex, b: &Vertex, c: &Vertex) -> bool {
    let area = 0.5 *(-b.y*c.x + a.y*(-b.x + c.x) + a.x*(b.y - c.y) + b.x*c.y);
    let s = 1.0/(2.0*area)*(a.y*c.x - a.x*c.y + (c.y - a.y)*p.x + (a.x - c.x)*p.y);
    let t = 1.0/(2.0*area)*(a.x*b.y - a.y*b.x + (a.y - b.y)*p.x + (b.x - a.x)*p.y);
    let u = 1.0 - s - t;
    s >= 0.0 && t >= 0.0 && u >= 0.0
}

fn ear_clipping(vertices: Vec<Vertex>) -> Vec<[Vertex;3]> {
    let mut triangles = vec![];
    let mut polygon = vertices.clone();

    while polygon.len() >= 3 {
        let n = polygon.len();
        let mut ear_found = false;

        for i in 0..n {
            let prev = polygon[(i + n - 1) % n].clone();
            let curr = polygon[i].clone();
            let next = polygon[(i + 1) % n].clone();

            if is_convex(&prev, &curr, &next) {
                // comprobar si hay algún otro vértice dentro
                let mut any_inside = false;
                for j in 0..n {
                    if j == i || j == (i + 1)%n || j == (i + n - 1)%n { continue; }
                    if point_in_triangle(&polygon[j], &prev, &curr, &next) {
                        any_inside = true;
                        break;
                    }
                }
                if !any_inside {
                    // recortar oreja
                    triangles.push([prev.clone(), curr.clone(), next.clone()]);
                    polygon.remove(i);
                    ear_found = true;
                    break;
                }
            }
        }
        if !ear_found {
            println!("⚠ no se pudo recortar más orejas (posible polígono mal formado)");
            break;
        }
    }
    triangles
}

fn draw_polygon_outline(image: &mut Image, points: &[(i32, i32)], color: Color, image_height: i32) {
    for i in 0..points.len() {
        let (x1, y1) = points[i];
        let (x2, y2) = points[(i + 1) % points.len()];
        let y1_flip = image_height - y1;
        let y2_flip = image_height - y2;
        image.draw_line(x1, y1_flip, x2, y2_flip, color);
    }
}

fn main() {
    let image_width = 800;
    let image_height = 600;

    let mut new_image = Image::gen_image_color(image_width, image_height, Color::WHITE);

    let polygone1 = [
        (165, 380),(185, 360),(180, 330),(207, 345),(233, 330),
        (230, 360),(250, 380),(220, 385),(205, 410),(193, 383),
    ];

    let polygone2 = [
        (321, 335),(288, 286),(339, 251),(374, 302),
    ];

    let polygone3 = [
        (377, 249),(411, 197),(436, 249),
    ];

    let polygone4 = [
        (413, 177),(448, 159),(502, 88),(553, 53),(535, 36),
        (676, 37),(660, 52),(750, 145),(761, 179),(672, 192),
        (659, 214),(615, 214),(632, 230),(580, 230),(597, 215),
        (552, 214),(517, 144),(466, 180),
    ];

    let polygone5 = [
        (682, 175),(708, 120),(735, 148),(739, 170),
    ];

    // polígono 1
    {
        let verts: Vec<Vertex> = polygone1.iter()
            .map(|&(x,y)| Vertex::new(x,y,image_height))
            .collect();
        let tris = ear_clipping(verts);
        for tri in tris {
            new_image.draw_triangle(
                Vector2::new(tri[0].x, tri[0].y),
                Vector2::new(tri[1].x, tri[1].y),
                Vector2::new(tri[2].x, tri[2].y),
                Color::YELLOW,
            );
        }
        draw_polygon_outline(&mut new_image, &polygone1, Color::WHITE, image_height);
    }

    // polígono 2
    {
        let verts: Vec<Vertex> = polygone2.iter()
            .map(|&(x,y)| Vertex::new(x,y,image_height))
            .collect();
        let tris = ear_clipping(verts);
        for tri in tris {
            new_image.draw_triangle(
                Vector2::new(tri[0].x, tri[0].y),
                Vector2::new(tri[1].x, tri[1].y),
                Vector2::new(tri[2].x, tri[2].y),
                Color::BLUE,
            );
        }
        draw_polygon_outline(&mut new_image, &polygone2, Color::WHITE, image_height);
    }

    // polígono 3
    {
        let verts: Vec<Vertex> = polygone3.iter()
            .map(|&(x,y)| Vertex::new(x,y,image_height))
            .collect();
        let tris = ear_clipping(verts);
        for tri in tris {
            new_image.draw_triangle(
                Vector2::new(tri[0].x, tri[0].y),
                Vector2::new(tri[1].x, tri[1].y),
                Vector2::new(tri[2].x, tri[2].y),
                Color::RED,
            );
        }
        draw_polygon_outline(&mut new_image, &polygone3, Color::WHITE, image_height);
    }

    // polígono 4
    {
        let verts: Vec<Vertex> = polygone4.iter()
            .map(|&(x,y)| Vertex::new(x,y,image_height))
            .collect();
        let tris = ear_clipping(verts);
        for tri in tris {
            new_image.draw_triangle(
                Vector2::new(tri[0].x, tri[0].y),
                Vector2::new(tri[1].x, tri[1].y),
                Vector2::new(tri[2].x, tri[2].y),
                Color::GREEN,
            );
        }
        draw_polygon_outline(&mut new_image, &polygone4, Color::WHITE, image_height);
    }

    // polígono 5
    {
        let verts: Vec<Vertex> = polygone5.iter()
            .map(|&(x,y)| Vertex::new(x,y,image_height))
            .collect();
        let tris = ear_clipping(verts);
        for tri in tris {
            new_image.draw_triangle(
                Vector2::new(tri[0].x, tri[0].y),
                Vector2::new(tri[1].x, tri[1].y),
                Vector2::new(tri[2].x, tri[2].y),
                Color::WHITE,  
            );
        }
        draw_polygon_outline(&mut new_image, &polygone5, Color::WHITE, image_height);
    }

    // exportar
    let output_file_name = "out.png";
    new_image.export_image(output_file_name);
    println!("✅ Imagen '{}' guardada correctamente.", output_file_name);
}
