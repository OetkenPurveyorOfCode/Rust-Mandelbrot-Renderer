extern crate minifb;

use minifb::{Key, MouseButton, MouseMode, Scale, ScaleMode, Window, WindowOptions};
use rayon::prelude::*;
const WIDTH: usize = 640;
const HEIGHT: usize = 360;

#[derive(Default, Debug)]
struct Rect {
    x_min: f64,
    x_max: f64,
    y_min: f64,
    y_max: f64,
}

fn transform_pixel_to_coord_x(value: u32, width: usize, rect: &Rect) -> f64 {
    let x_coord = value as f64 / width as f64;
    let x_coord = (rect.x_max - rect.x_min) * x_coord + rect.x_min;
    x_coord
}

fn transform_pixel_to_coord_y(value: u32, height: usize, rect: &Rect) -> f64 {
    let y_coord = ((height as u32 - value) as f64) / height as f64;
    let y_coord = (rect.y_max - rect.y_min) * y_coord + rect.y_min;
    y_coord
}

fn render(buffer: &mut Vec<u32>, width: usize, height: usize, rect: &Rect, iterations: usize) {
    *buffer = (0..width * height)
        .into_par_iter()
        .map(|index| {
            // x goes from 0 to WIDTH
            let x = index as u32 % width as u32;
            let x_coord = transform_pixel_to_coord_x(x, width, rect);

            // y goes from 0 to HEIGHT
            let y = index as u32 / width as u32;
            let y_coord = transform_pixel_to_coord_y(y, height, rect);

            let mut z_real: f64 = 0.0;
            let mut z_imag: f64 = 0.0;
            let mut z_real_tmp: f64;
            let c_real: f64 = x_coord;
            let c_imag: f64 = y_coord;
            let mut iter = 0;
            while z_real * z_real + z_imag * z_imag < 4.0 && iter < iterations {
                //z = Z**2 + c
                //z = (z_real + i * z_imag) **2 + c_real + i*c_imag
                //Z = z_real**2 + 2i*z_real*z_imag - z_imag**2 + c_real + i* c_imag
                z_real_tmp = z_real * z_real - z_imag * z_imag + c_real;
                z_imag = 2.0 * z_real * z_imag + c_imag;
                z_real = z_real_tmp;
                iter += 1;
            }
            if iter == iterations {
                0x00_00_00_FF
            } else {
                0x00_FF_FF_FF
            }
        })
        .collect();
}

fn render_mouse_rect(buffer: &mut Vec<u32>, width: usize, rect: &Rect) {
    // render left and right lines
    for x in (rect.x_min as u32)..(rect.x_max as u32) {
        buffer[x as usize + rect.y_min as usize * width] = 0x00_FF_00_00;
        buffer[x as usize + rect.y_max as usize * width] = 0x00_FF_00_00;
    }
    // render top and bottom lines 
    for y in rect.y_min as usize..rect.y_max as usize {
        buffer[rect.x_min as usize + y * width] = 0x00_FF_00_00;
        buffer[rect.x_max as usize + y * width] = 0x00_FF_00_00;
    }
}

fn main() {
    let mut buffer: Vec<u32> = vec![0; WIDTH * HEIGHT];
    let options: WindowOptions = WindowOptions {
        borderless: false,
        title: true,
        resize: true,
        scale: Scale::X1,
        scale_mode: ScaleMode::Center,
        topmost: false,
        transparency: false,
        none: false,
    };
    let mut window = Window::new(
        "Mandelbrot - ESC: EXIT- I/D: ITERATIONS=100 - Arrows: MOVEMENT - +/-: ZOOM",
        WIDTH,
        HEIGHT,
        options,
    )
    .unwrap_or_else(|e| {
        panic!("{}", e);
    });

    window.limit_update_rate(Some(std::time::Duration::from_micros(16600)));

    let mut dirty = true;
    let mut oldwidth = 0;
    let mut oldheight = 0;
    let mut oldmouse_down = false;
    let mut oldmouse_pos_x = -1.0;
    let mut oldmouse_pos_y = -1.0;
    let mut mouse_pos_x = -1.0;
    let mut mouse_pos_y = -1.0;
    let mut oldbuffer = buffer.clone();
    let mut rect = Rect {
        x_min: -2.0,
        x_max: 1.0,
        y_min: -1.0,
        y_max: 1.0,
    };
    let mut iterations = 100;
    let zoom = 0.8;
    let panning = 0.1;
    while window.is_open() && !window.is_key_down(Key::Escape) {
        let (width, height): (usize, usize) = window.get_size();
        if !(width == oldwidth && height == oldheight) {
            dirty = true;
        }
        let mouse_down = window.get_mouse_down(MouseButton::Left);
        //println!("mouse down first {mouse_down}");
        match (oldmouse_down, mouse_down) {
            (false, false) => {} // no action
            (false, true) => {
                // first mouse down
                let (mouse_x, mouse_y): (f32, f32) = window
                    .get_mouse_pos(MouseMode::Clamp)
                    .unwrap_or((-1.0, -1.0)); // FIXME
                oldmouse_down = true;
                oldmouse_pos_x = mouse_x as f64;
                oldmouse_pos_y = mouse_y as f64;
            }
            (true, false) => {
                let selection: Rect = Rect {
                    x_min: if oldmouse_pos_x < mouse_pos_x {
                        oldmouse_pos_x
                    } else {
                        mouse_pos_x
                    },
                    x_max: if oldmouse_pos_x > mouse_pos_x {
                        oldmouse_pos_x
                    } else {
                        mouse_pos_x
                    },
                    y_min: if oldmouse_pos_y < mouse_pos_y {
                        oldmouse_pos_y
                    } else {
                        mouse_pos_y
                    },
                    y_max: if oldmouse_pos_y > mouse_pos_y {
                        oldmouse_pos_y
                    } else {
                        mouse_pos_y
                    },
                };
                rect = Rect {
                    x_min: transform_pixel_to_coord_x(selection.x_min as u32, width, &rect),
                    x_max: transform_pixel_to_coord_x(selection.x_max as u32, width, &rect),
                    y_min: transform_pixel_to_coord_y(selection.y_min as u32, height, &rect),
                    y_max: transform_pixel_to_coord_y(selection.y_max as u32, height, &rect),
                };
                // release
                dirty = true;
                oldmouse_down = false;
            }
            (true, true) => {
                // dragging
                let (mouse_x, mouse_y): (f32, f32) = window
                    .get_mouse_pos(MouseMode::Clamp)
                    .unwrap_or((-1.0, -1.0)); // FIXME
                mouse_pos_x = mouse_x as f64;
                mouse_pos_y = mouse_y as f64;
                let selection: Rect = Rect {
                    x_min: if oldmouse_pos_x < mouse_pos_x {
                        oldmouse_pos_x
                    } else {
                        mouse_pos_x
                    },
                    x_max: if oldmouse_pos_x > mouse_pos_x {
                        oldmouse_pos_x
                    } else {
                        mouse_pos_x
                    },
                    y_min: if oldmouse_pos_y < mouse_pos_y {
                        oldmouse_pos_y
                    } else {
                        mouse_pos_y
                    },
                    y_max: if oldmouse_pos_y > mouse_pos_y {
                        oldmouse_pos_y
                    } else {
                        mouse_pos_y
                    },
                };
                buffer = oldbuffer.clone();
                render_mouse_rect(&mut buffer, width, &selection);
            }
        };
        if window.is_key_down(Key::R) {
            rect = Rect {
                x_min: -2.0,
                x_max: 1.0,
                y_min: -1.0,
                y_max: 1.0,
            };
            dirty = true;
        }
        if window.is_key_down(Key::I) {
            let mut increase = 1;
            if window.is_key_down(Key::LeftShift) || window.is_key_down(Key::RightShift) {
                increase = 10;
            }
            if iterations < usize::MAX - increase {
                iterations += increase;
            }
            let title = format!("Mandelbrot - ESC: EXIT- I/D: ITERATIONS={iterations} - Arrows: MOVEMENT - +/-: ZOOM");
            window.set_title(&title);
            dirty = true;
        }
        if window.is_key_down(Key::D) {
            let mut decrease = 1;
            if window.is_key_down(Key::LeftShift) || window.is_key_down(Key::RightShift) {
                decrease = 10;
            }
            if iterations > decrease {
                iterations -= decrease;
            }
            let title = format!("Mandelbrot - ESC: EXIT- I/D: ITERATIONS={iterations} - Arrows: MOVEMENT - +/-: ZOOM");
            window.set_title(&title);
            dirty = true;
        }
        if window.is_key_down(Key::NumPadPlus) {
            let center_x = (rect.x_max + rect.x_min) / 2.0;
            let center_y = (rect.y_max + rect.y_min) / 2.0;
            let edge_x = (rect.x_max - rect.x_min) / 2.0;
            let edge_y = (rect.y_max - rect.y_min) / 2.0;
            rect = Rect {
                x_min: center_x - edge_x * zoom,
                x_max: center_x + edge_x * zoom,
                y_min: center_y - edge_y * zoom,
                y_max: center_y + edge_y * zoom,
            };
            dirty = true;
        }
        if window.is_key_down(Key::NumPadMinus) {
            let center_x = (rect.x_max + rect.x_min) / 2.0;
            let center_y = (rect.y_max + rect.y_min) / 2.0;
            let edge_x = (rect.x_max - rect.x_min) / 2.0;
            let edge_y = (rect.y_max - rect.y_min) / 2.0;
            rect = Rect {
                x_min: center_x - edge_x / zoom,
                x_max: center_x + edge_x / zoom,
                y_min: center_y - edge_y / zoom,
                y_max: center_y + edge_y / zoom,
            };
            dirty = true;
        }
        if window.is_key_down(Key::Left) {
            let pan = (rect.x_max - rect.x_min) * panning;
            rect.x_max -= pan;
            rect.x_min -= pan;
            dirty = true;
        }
        if window.is_key_down(Key::Right) {
            let pan = (rect.x_max - rect.x_min) * panning;
            rect.x_max += pan;
            rect.x_min += pan;
            dirty = true;
        }
        if window.is_key_down(Key::Up) {
            let pan = (rect.y_max - rect.y_min) * panning;
            rect.y_max += pan;
            rect.y_min += pan;
            dirty = true;
        }
        if window.is_key_down(Key::Down) {
            let pan = (rect.y_max - rect.y_min) * panning;
            rect.y_max -= pan;
            rect.y_min -= pan;
            dirty = true;
        }

        if dirty {
            buffer.resize(width * height, 0);
            render(&mut buffer, width, height, &rect, iterations);
            oldbuffer = buffer.clone();
            dirty = false;
            oldwidth = width;
            oldheight = height;
        }
        window.update_with_buffer(&buffer, width, height).unwrap();
    }
}
