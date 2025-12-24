use macroquad::prelude::*;

// slider struct
struct Slider {
    label: String,
    value: f64,
    min: f64,
    max: f64,
    x: f32,
    y: f32,
}

fn draw_slider(slider: &mut Slider) {
    // draw the label
    draw_text(&slider.label, slider.x, slider.y - 10.0, 20.0, WHITE);
    // draw the slider line
    draw_line(slider.x, slider.y, slider.x + 200.0, slider.y, 2.0, WHITE);
    // draw the slider handle
    let handle_x: f32 = ((slider.value - slider.min) / (slider.max - slider.min) * 200.0) as f32 + slider.x;
    draw_circle(handle_x as f32, slider.y, 5.0, RED);
    // draw slider value
    draw_text(&format!("{:.2}", slider.value), slider.x + 210.0, slider.y + 5.0, 20.0, WHITE);
}

fn take_slider_input(slider: &mut Slider) -> f64 {
    let mouse_x = mouse_position().0;
    let mouse_y = mouse_position().1;
    if is_mouse_button_down(MouseButton::Left) && mouse_x >= slider.x && mouse_x <= slider.x + 200.0 && mouse_y >= slider.y - 10.0 && mouse_y <= slider.y + 10.0 {
        slider.value = ((mouse_x - slider.x) / 200.0) as f64 * (slider.max - slider.min) + slider.min;
    }
    slider.value
}

#[macroquad::main("Squircle (superellipse) drawer")]
async fn main() {
    // it would be best to do this parametrically

    // formula
    /*
      | x / r_a |^n
    + | y / r_b |^n
    = 1
    */

    // parametric formula:
    /*
      x = r_a * cos(t)^(2/n)
      y = r_b * sin(t)^(2/n)
    */

    // go from t = 0 to t = pi/2, in order to find all vertices in the first quadrant
    // this is anticlockwise (if y axis is up)
    // but clockwise from the bottom right quadrant (since y axis is down)
    
    let mut r_a: f64 = screen_width() as f64 / 4.0;
    let mut r_b: f64 = r_a.clone();
    let mut n: f64 = 4.0;

    let mut steps = 85;
    // preallocate space in vertex buffer
    let mut vertex_buffer: Vec<Vec2> = Vec::with_capacity(steps); // holds a quarter of the vertices,
                                                                  // which can then be rotated and mirrored
                                                                  // to find the rest.
    let mut thickness: f32 = 2.0;

    // initialise sliders
        let mut r_a_slider: Slider = Slider {
            label: String::from("Semi-major axis (r_a)"),
            value: r_a,
            min: 10.0,
            max: 700.0,
            x: 20.0,
            y: 40.0,
        };
        let mut r_b_slider: Slider = Slider {
            label: String::from("Semi-minor axis (r_b)"),
            value: r_b,
            min: 10.0,
            max: 700.0,
            x: 20.0,
            y: 80.0,
        };
        let mut n_slider: Slider = Slider {
            label: String::from("Roundedness (n)"),
            value: n,
            min: 0.1,
            max: 20.0,
            x: 20.0,
            y: 120.0,
        };
        let mut thickness_slider: Slider = Slider {
            label: String::from("thickness"),
            value: thickness as f64,
            min: 0.1,
            max: 40.0,
            x: 20.0,
            y: 160.0,
        };
        let mut steps_slider: Slider = Slider {
            label: String::from("steps"),
            value: steps as f64,
            min: 1.0,
            max: 100.0,
            x: 20.0,
            y: 200.0,
        };


    // plot the shape using macroquad
    loop {
        let centre: Vec2 = vec2(screen_width() / 2.0, screen_height() / 2.0);

        // handle inputs for sliders for r_a, r_b, n, thickness
        r_a = take_slider_input(&mut r_a_slider);
        r_b = take_slider_input(&mut r_b_slider);
        n = take_slider_input(&mut n_slider);
        thickness = take_slider_input(&mut thickness_slider) as f32;
        steps = take_slider_input(&mut steps_slider) as usize;

        // recalculate vertices
        vertex_buffer.clear();
        for i in 0..=steps {
            let t = (std::f64::consts::FRAC_PI_2) * (i as f64) / (steps as f64);
            // Explicitly handle the quadrant edges to avoid floating point errors
            // creating gaps when 'n' is large.
            let (sin_t, cos_t) = t.sin_cos(); // optimisation
            let x = if i == steps { 
                0.0 
            } else { 
                r_a * cos_t.abs().powf(2.0 / n) * cos_t.signum() 
            };
            let y = if i == 0 { 
                0.0 
            } else { 
                r_b * sin_t.abs().powf(2.0 / n) * sin_t.signum()
            };
            
            vertex_buffer.push(vec2(x as f32, y as f32));        
        }

        clear_background(DARKGRAY);
        // draw the shape by connecting the vertices
        for i in 0..=steps {
            let v = vertex_buffer[i];
            // first quadrant
            if i > 0 {
                let v_prev = vertex_buffer[i - 1];
                draw_line(v_prev.x + centre.x, v_prev.y + centre.y, v.x + centre.x, v.y + centre.y, thickness, WHITE);
            }
            // second quadrant
            let v_mirror_x = vec2(-v.x, v.y);
            if i > 0 {
                let v_prev = vertex_buffer[i - 1];
                let v_prev_mirror_x = vec2(-v_prev.x, v_prev.y);
                draw_line(v_prev_mirror_x.x + centre.x, v_prev_mirror_x.y + centre.y, v_mirror_x.x + centre.x, v_mirror_x.y + centre.y, thickness, WHITE);
            }
            // third quadrant
            let v_mirror_xy = vec2(-v.x, -v.y);
            if i > 0 {
                let v_prev = vertex_buffer[i - 1];
                let v_prev_mirror_xy = vec2(-v_prev.x, -v_prev.y);
                draw_line(v_prev_mirror_xy.x + centre.x, v_prev_mirror_xy.y + centre.y, v_mirror_xy.x + centre.x, v_mirror_xy.y + centre.y, thickness, WHITE);
            }
            // fourth quadrant
            let v_mirror_y = vec2(v.x, -v.y);
            if i > 0 {
                let v_prev = vertex_buffer[i - 1];
                let v_prev_mirror_y = vec2(v_prev.x, -v_prev.y);
                draw_line(v_prev_mirror_y.x + centre.x, v_prev_mirror_y.y + centre.y, v_mirror_y.x + centre.x, v_mirror_y.y + centre.y, thickness, WHITE);
            }
        }

        // draw sliders
            draw_slider(&mut r_a_slider);
            draw_slider(&mut r_b_slider);
            draw_slider(&mut n_slider);
            draw_slider(&mut thickness_slider);
            draw_slider(&mut steps_slider);

        next_frame().await

    }


}
