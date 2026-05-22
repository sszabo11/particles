use rand::RngExt;

pub struct State {
    pub particles: Vec<Particle>,
    pub fabric: Vec<f32>,
    pub width: u32,
    pub height: u32,
}

impl State {
    pub fn new(n_particles: usize, width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            fabric: vec![0.0; (width * height) as usize],
            particles: (0..n_particles)
                .map(|i| {
                    let mut rng = rand::rng();
                    //let center_x = rng.random_range(0..width) as f32;
                    //let center_y = rng.random_range(0..height) as f32;

                    ////let vx = rng.random_range(0..1) as f32;
                    ////let vy = rng.random_range(0..1) as f32;
                    //let angle = rng.random_range(0..1) as f32 * 2.0 * std::f32::consts::PI;
                    //let speed = rng.random_range(1.0..3.0) as f32;

                    let center_x = width as f32 / 2.0;
                    let center_y = height as f32 / 2.0;

                    // Spawn particles in a ring around the center
                    let radius = 20.0;
                    let angle = (i as f32) * 0.1; // Distribute them evenly

                    let px = center_x + angle.cos() * radius;
                    let py = center_y + angle.sin() * radius;

                    // THE ORBIT SECRET: Set velocity perpendicular to the position vector
                    //let orbit_speed = 0.01;
                    let orbit_speed = rng.random_range(0.01..0.3);
                    let pvx = -angle.sin() * orbit_speed; // Flipped sin/cos creates a perfect swirl
                    let pvy = angle.cos() * orbit_speed;

                    let mass = rng.random_range(0.5..50.);
                    Particle {
                        x: px,
                        y: py,
                        vx: pvx,
                        vy: pvy,
                        mass: mass,
                        ax: 0.,
                        ay: 0.,
                    }
                })
                .collect(),
        }
    }

    // After you seed particles
    pub fn update_fabric(&mut self) {
        let radius = 100;
        let radius_f = radius as f32;
        //for x in 0..self.width {
        //    for y in 0..self.height {
        //        let i = (y * self.width + x) as usize;
        //    }
        //}
        self.fabric.fill(0.);

        for particle in self.particles.iter() {
            let px = particle.x as i32;
            let py = particle.y as i32;

            for offset_x in -radius..=radius {
                for offset_y in -radius..=radius {
                    let x = px + offset_x;
                    let y = py + offset_y;
                    if x >= self.width as i32 || y >= self.height as i32 || x < 0 || y < 0 {
                        continue;
                    }

                    let dist = (offset_x * offset_x + offset_y * offset_y) as f32;

                    if dist <= radius_f * radius_f {
                        let dist = dist.sqrt();
                        let norm_dist = dist / radius_f;
                        //let falloff = 1.0 - (norm_dist * norm_dist * (3.0 - 2.0 * norm_dist));
                        //let falloff = 0.01 * (1. / norm_dist); // nice
                        let falloff = 0.05 * (1. / norm_dist); // nice
                        let i = (y * self.width as i32 + x) as usize;
                        self.fabric[i] += falloff * particle.mass;
                    }
                }
            }
            if py >= 0 && ((py as u32) < self.height) && px >= 0 && (px as u32) < self.width {
                let i = (py * self.width as i32 + px) as usize;
                self.fabric[i] = -1. * particle.mass;
            }
        }
    }
}

pub struct Particle {
    pub x: f32,
    pub y: f32,
    pub vx: f32,
    pub mass: f32,
    pub vy: f32,
    pub ax: f32,
    pub ay: f32,
}

pub fn get_distance(a: Particle, b: Particle) -> f32 {
    let dx = b.x - a.x;
    let dy = b.y - a.y;

    let dist = (dx.powi(2) + dy.powi(2)).sqrt();
    dist
}
