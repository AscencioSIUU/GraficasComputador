// Módulo principal de shaders

pub mod common;
pub mod advanced_noise;
pub mod sun;
pub mod mercury;
pub mod venus;
pub mod earth;
pub mod mars;
pub mod goliath;
pub mod spaceship;

// Re-exportar tipos comunes
pub use common::{Fragment, Uniforms, Vector3Ext};

// Re-exportar shaders y vertex displacement functions
pub use sun::{star_shader, vertex_displacement};
pub use mercury::{cellular_planet_shader, vertex_displacement_mercury};
pub use venus::{simplex_planet_shader, vertex_displacement_venus};
pub use earth::{voronoi_planet_shader, vertex_displacement_earth};
pub use mars::{perlin_planet_shader, vertex_displacement_mars};
pub use goliath::{planet_shader, vertex_displacement_goliath};
pub use spaceship::spaceship_shader;
