use pixels::{Pixels, SurfaceTexture};
use winit::monitor::MonitorHandle;
use winit::platform::x11::WindowAttributesExtX11;
mod state;
use std::sync::Arc;
use std::time::{Duration, Instant};
use winit::application::ApplicationHandler;
use winit::event::{MouseScrollDelta, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::window::{Fullscreen, Window, WindowId};

use crate::state::{Particle, State, get_distance};

const WINDOW_WIDTH: u32 = 1200;
const WINDOW_HEIGHT: u32 = 1200;

const SIM_WIDTH: u32 = 6000;
const SIM_HEIGHT: u32 = 6000;

const VIEW_WIDTH: u32 = 1000;
const VIEW_HEIGHT: u32 = 1000;

struct App {
    window: Option<Arc<Window>>,
    pixels: Option<Pixels<'static>>,
    frame_count: u32,
    state: State,

    mouse_down: bool,
    offset_x: u32,
    offset_y: u32,

    camera_x: f32,
    camera_y: f32,
    zoom: f32,

    last_mouse_x: f32,
    last_mouse_y: f32,

    last_update: Instant,
}

impl App {
    pub fn update(&mut self) {
        for particle in self.state.particles.iter_mut() {
            if particle.x > 1.0
                && particle.x < (SIM_WIDTH - 1) as f32
                && particle.y > 1.0
                && particle.y < (SIM_HEIGHT - 1) as f32
            {
                let x = particle.x as usize;
                let y = particle.y as usize;

                let i_left = y * SIM_WIDTH as usize + x - 1;
                let i_right = y * SIM_WIDTH as usize + x + 1;
                let i_top = (y - 1) * SIM_WIDTH as usize + x;
                let i_bottom = (y + 1) * SIM_WIDTH as usize + x;

                let left = self.state.fabric[i_left];
                let right = self.state.fabric[i_right];
                let top = self.state.fabric[i_top];
                let bottom = self.state.fabric[i_bottom];

                let x_slope = right - left;
                let y_slope = bottom - top;

                let gravity_strength = 0.0002;

                particle.vx += x_slope * gravity_strength;
                particle.vy += y_slope * gravity_strength;

                particle.vx *= 0.999999;
                particle.vy *= 0.999999;

                particle.x += particle.vx;
                particle.y += particle.vy;
            }
            //let bounce = -0.5; // Invert velocity and cut it in half to absorb impact
            //if particle.x < 1.0 {
            //    particle.x = 1.0;
            //    particle.vx *= bounce;
            //}
            //if particle.x >= (SIM_WIDTH - 2) as f32 {
            //    particle.x = (SIM_WIDTH - 2) as f32;
            //    particle.vx *= bounce;
            //}
            //if particle.y < 1.0 {
            //    particle.y = 1.0;
            //    particle.vy *= bounce;
            //}
            //if particle.y >= (SIM_HEIGHT - 2) as f32 {
            //    particle.y = (SIM_HEIGHT - 2) as f32;
            //    particle.vy *= bounce;
            //}
        }
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_none() {
            let window_attributes = Window::default_attributes()
                .with_title("Atomic Universe Sandbox")
                .with_inner_size(winit::dpi::LogicalSize::new(
                    WINDOW_WIDTH as f64,
                    WINDOW_HEIGHT as f64,
                ));
            //.with_fullscreen(Some(winit::window::Fullscreen::Borderless(None)));
            //.with_maximized(true);

            let window = Arc::new(event_loop.create_window(window_attributes).unwrap());

            let window_size = window.inner_size();
            println!("window size: {:?}", window_size);
            let surface_texture =
                SurfaceTexture::new(window_size.width, window_size.height, window.clone());
            let pixels = Pixels::new(VIEW_WIDTH, VIEW_HEIGHT, surface_texture).unwrap();

            self.window = Some(window);
            self.pixels = Some(pixels);
        }
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),

            WindowEvent::Resized(size) => {
                if let Some(pixels) = &mut self.pixels {
                    pixels.resize_surface(size.width, size.height).unwrap();
                }
            }
            WindowEvent::CursorMoved {
                device_id,
                position,
            } => {
                //println!("{} {}", position.x, position.y);
                let mx = position.x as f32;
                let my = position.y as f32;

                if self.mouse_down {
                    let dx = mx - self.last_mouse_x;
                    let dy = my - self.last_mouse_y;

                    self.camera_x -= dx / self.zoom;
                    self.camera_y -= dy / self.zoom;
                }
                self.last_mouse_x = mx;
                self.last_mouse_y = my;
            }

            WindowEvent::MouseWheel {
                device_id,
                delta,
                phase,
            } => {
                match delta {
                    MouseScrollDelta::LineDelta(x, y) => {
                        self.zoom += (y * 0.1).clamp(-5., 5.);
                        println!("Scroll lines: x: {}, y: {}", x, y);
                    }
                    MouseScrollDelta::PixelDelta(pos) => {
                        //println!("Scroll pixels: x: {}, y: {}", pos.x, pos.y);
                    }
                }
            }
            WindowEvent::MouseInput {
                device_id,
                state,
                button,
            } => {
                if state.is_pressed() {
                    println!("down");
                    self.mouse_down = true;
                } else {
                    println!("up");
                    self.mouse_down = false;
                }
            }

            WindowEvent::RedrawRequested => {
                self.frame_count += 1;
                let physics_sub_steps = 20;

                //print_fabric(&self.state.fabric);
                //println!();

                for _ in 0..physics_sub_steps {
                    self.state.update_fabric();
                    self.update();
                }

                if let Some(pixels) = &mut self.pixels {
                    let frame = pixels.frame_mut();
                    frame.fill(0);

                    let half_w = VIEW_WIDTH as f32 / 2.0;
                    let half_h = VIEW_HEIGHT as f32 / 2.0;

                    let zoom_inv = 1.0 / self.zoom;

                    for sx in 0..VIEW_WIDTH {
                        for sy in 0..VIEW_HEIGHT {
                            let world_x = (self.camera_x + (sx as f32 - half_w) * zoom_inv) as i32;
                            let world_y = (self.camera_y + (sy as f32 - half_h) * zoom_inv) as i32;

                            let intensity = if world_x >= 0
                                && world_x < SIM_WIDTH as i32
                                && world_y >= 0
                                && world_y < SIM_HEIGHT as i32
                            {
                                let idx =
                                    (world_y as usize * SIM_WIDTH as usize) + world_x as usize;
                                let depth = self.state.fabric[idx];

                                (depth * 55.0).clamp(0.0, 255.0) as u8
                            } else {
                                0
                            };
                            let i = ((sy * VIEW_WIDTH as u32 + sx) * 4) as usize;
                            //let depth = self.state.fabric[x as usize][y as usize];
                            //let i = ((world_y * VIEW_WIDTH + world_x) * 4) as usize;
                            //let intensity = (depth * 55.0).clamp(0.0, 255.0) as u8;

                            frame[i] = intensity;
                            frame[i + 1] = intensity / 4;
                            frame[i + 2] = 20;
                            frame[i + 3] = 255;
                        }
                    }
                    for particle in self.state.particles.iter() {
                        let screen_x = ((particle.x - self.camera_x) * self.zoom
                            + VIEW_WIDTH as f32 / 2.0)
                            as i32;
                        let screen_y = ((particle.y - self.camera_y) * self.zoom
                            + VIEW_HEIGHT as f32 / 2.0)
                            as i32;

                        // Only draw if particle is visible on screen
                        if screen_x >= 0
                            && screen_x < VIEW_WIDTH as i32
                            && screen_y >= 0
                            && screen_y < VIEW_HEIGHT as i32
                        {
                            let i = ((screen_y as u32 * VIEW_WIDTH + screen_x as u32) * 4) as usize;
                            let hue = (particle.mass * 36.0) % 360.0; // tweak multiplier for color spread

                            let (r, g, b) = hsl_to_rgb(hue, 0.95, 0.58); // high saturation, medium lightness
                            // Bright particle (yellow-orange)
                            frame[i] = r; // R
                            frame[i + 1] = g; // G
                            frame[i + 2] = b; // B
                            frame[i + 3] = 255; // A

                            // Optional: Make them bigger (3x3)
                            // You can expand this into a small cross or square if you want
                            //let offsets = [(-1, 0), (1, 0), (0, -1), (0, 1)];
                            //for (dx, dy) in offsets {
                            //    let px = screen_x + dx;
                            //    let py = screen_y + dy;
                            //    if px >= 0
                            //        && px < VIEW_WIDTH as i32
                            //        && py >= 0
                            //        && py < VIEW_HEIGHT as i32
                            //    {
                            //        let i2 = ((py as u32 * VIEW_WIDTH + px as u32) * 4) as usize;
                            //        frame[i2] = 255;
                            //        frame[i2 + 1] = 180;
                            //        frame[i2 + 2] = 40;
                            //    }
                            //}
                        }
                    }
                    //for particle in self.state.particles.iter_mut() {
                    //    //            if particle.x >= WIDTH - 1 || particle.x <= 1 {
                    //    //                particle.x = 0;
                    //    //            }
                    //    //            if particle.y >= HEIGHT - 1 || particle.y <= 1 {
                    //    //                particle.y = 0;
                    //    //            }

                    //    let px = particle.x as u32;
                    //    let py = particle.y as u32;

                    //    if (px < SIM_WIDTH - 1 && px > 1) && (py < SIM_HEIGHT - 1 && py > 1) {
                    //        let x = particle.x as u32;
                    //        let y = particle.y as u32;

                    //        let i = ((y * SIM_WIDTH + x) * 4) as usize;
                    //        //if x >= WIDTH || y >= HEIGHT {
                    //        //    particle.y = 1.;
                    //        //}

                    //        //frame[i] = ((x + self.frame_count) % 255) as u8; // Red
                    //        //frame[i + 1] = ((y + self.frame_count) % 255) as u8; // Red
                    //        frame[i] = (i % 255) as u8; // Red
                    //        frame[i + 1] = (i % 255) as u8; // Red
                    //        frame[i + 2] = 128;
                    //        frame[i + 3] = 255;
                    //    }
                    //}
                    // Loop through your pixel buffer [R, G, B, A]
                    //for (i, pixel) in frame.chunks_exact_mut(4).enumerate() {
                    //    let x = (i % WIDTH as usize) as u32;
                    //    let y = (i / WIDTH as usize) as u32;

                    //    pixel[0] = ((x + self.frame_count) % 255) as u8; // Red
                    //    pixel[1] = ((y + self.frame_count) % 255) as u8; // Green
                    //    pixel[2] = 28; // Blue
                    //    pixel[3] = 255; // Alpha
                    //}

                    if let Err(err) = pixels.render() {
                        println!("Render error: {}", err);
                        event_loop.exit();
                    }
                }

                if let Some(window) = &self.window {
                    window.request_redraw();
                }
            }
            _ => (),
        }
    }

    // This forces the loop to continuously poll and trigger RedrawRequested
    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        if self.last_update.elapsed() >= Duration::from_secs(8) {
            if let Some(window) = &self.window {
                window.request_redraw();
            }
            self.last_update = Instant::now();
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let event_loop = EventLoop::new()?;
    event_loop.set_control_flow(ControlFlow::Poll);

    let n_particles = 30;

    let mut app = App {
        window: None,
        pixels: None,
        offset_y: 500,
        offset_x: 500,
        mouse_down: false,
        last_mouse_y: 0.,
        last_mouse_x: 0.,
        zoom: 1.,
        camera_x: SIM_WIDTH as f32 / 2.,
        last_update: Instant::now(),
        camera_y: SIM_WIDTH as f32 / 2.,
        frame_count: 0,
        state: State::new(n_particles, SIM_WIDTH, SIM_HEIGHT),
    };
    //app.state.particles.clear();
    //app.state.particles.push((99, 99));
    //app.state.particles.push((50, 50));
    //app.state.particles.push(Particle {
    //    x: 5.,
    //    y: 5.,
    //    vx: 0.,
    //    vy: 0.,
    //    ax: 0.,
    //    ay: 0.,
    //    mass: 1.,
    //});

    app.state.update_fabric();
    event_loop.run_app(&mut app)?;
    Ok(())
}

fn print_fabric(fabric: &Vec<Vec<f32>>) {
    for x in 0..fabric.len() {
        for y in 0..fabric[0].len() {
            let depth = fabric[x as usize][y as usize];
            print!("{} ", depth.abs());
        }
        println!();
    }
}

fn hsl_to_rgb(h: f32, s: f32, l: f32) -> (u8, u8, u8) {
    let h = h / 360.0;
    let c = (1.0 - (2.0 * l - 1.0).abs()) * s;
    let x = c * (1.0 - ((h * 6.0) % 2.0 - 1.0).abs());
    let m = l - c / 2.0;

    let (r, g, b) = match (h * 6.0).floor() as i32 {
        0 => (c, x, 0.0),
        1 => (x, c, 0.0),
        2 => (0.0, c, x),
        3 => (0.0, x, c),
        4 => (x, 0.0, c),
        _ => (c, 0.0, x),
    };

    (
        ((r + m) * 255.0).clamp(0.0, 255.0) as u8,
        ((g + m) * 255.0).clamp(0.0, 255.0) as u8,
        ((b + m) * 255.0).clamp(0.0, 255.0) as u8,
    )
}
