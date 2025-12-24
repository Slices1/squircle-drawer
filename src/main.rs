use macroquad::prelude::*;

struct Slider {
    label: String,
    value: f32,
    min: f32,
    max: f32,
    rect: Rect,
}
impl Slider {
    fn new(label: &str, value: f32, min: f32, max: f32, x: f32, y: f32) -> Self {
        Self {
            label: label.to_string(),
            value,
            min,
            max,
            rect: Rect::new(x, y, 200.0, 20.0),
        }
    }

    fn update(&mut self) -> bool {
        let mouse_pos = mouse_position();
        let m_vec = vec2(mouse_pos.0, mouse_pos.1);

        let interaction_area = Rect::new(self.rect.x, self.rect.y - 10.0, self.rect.w, self.rect.h + 20.0);

        if is_mouse_button_down(MouseButton::Left) && interaction_area.contains(m_vec) {
            let old_value = self.value;
            let ratio = (m_vec.x - self.rect.x) / self.rect.w;
            
            let clamped_ratio = ratio.clamp(0.0, 1.0);
            
            self.value = clamped_ratio * (self.max - self.min) + self.min;
            // return true if value effectively changed
            return (self.value - old_value).abs() > std::f32::EPSILON;
        }
        false
    }

    fn draw(&self) {
        // label
        draw_text(&self.label, self.rect.x, self.rect.y - 10.0, 20.0, WHITE);
        // line
        draw_line(self.rect.x, self.rect.y, self.rect.x + self.rect.w, self.rect.y, 2.0, WHITE);
        // handle
        let ratio = (self.value - self.min) / (self.max - self.min);
        let handle_x = self.rect.x + (ratio as f32 * self.rect.w);
        draw_circle(handle_x, self.rect.y, 5.0, RED);
        // value text
        draw_text(&format!("{:.2}", self.value), self.rect.x + self.rect.w + 10.0, self.rect.y + 5.0, 20.0, WHITE);
    }
}

fn render_quadrants<F>(
    c: Vec2, 
    steps: usize, 
    vertex_buffer: &[Vec2], 
    quadrants: &[(f32, f32)],
    rotation_matrix_trig_values: (f32, f32),
    mut draw_callback: F
) 
where F: FnMut(Vec2, Vec2) 
{
    for (sx, sy) in quadrants {
        let v_0 = vertex_buffer[0];
        let mut p_prev = Vec2::new(sx * v_0.x, sy * v_0.y);
        // apply rotation, then offset by centre
        let m = rotation_matrix_trig_values;
        p_prev = Vec2::new(
            m.1 * p_prev.x - m.0 * p_prev.y + c.x,
            m.0 * p_prev.x + m.1 * p_prev.y + c.y,
        );
        for i in 1..=steps {
            let v = vertex_buffer[i];

            let mut p_curr = Vec2::new(sx * v.x, sy * v.y);
            p_curr = Vec2::new(
                m.1 * p_curr.x - m.0 * p_curr.y + c.x,
                m.0 * p_curr.x + m.1 * p_curr.y + c.y,
            );

            draw_callback(p_prev, p_curr);
            p_prev = p_curr;
        }
    }
}

#[macroquad::main("Squircle (superellipse) drawer")]
async fn main() {
    let quadrants = [
            (1.0, 1.0),
            (-1.0, 1.0),  
            (-1.0, -1.0), 
            (1.0, -1.0)   
        ]; // stores the sign multipliers for each quadrant

    // preallocate space in vertex buffer
    let mut vertex_buffer: Vec<Vec2> = Vec::with_capacity(80); // holds a quarter of the vertices,
                                                                  // which can then be rotated and mirrored
                                                                  // to find the rest.
    let mut rotation_matrix_trig_values: (f32, f32) = (0.0, 1.0); // stores the sin and cos values of the rot matrix

    let mut vertices_need_recalculation = true;
    // initialise sliders
    let mut r_a_slider = Slider::new("Semi-major axis (r_a)", screen_width() / 4.0, 10.0, screen_width(), 20.0, 40.0);
    let mut r_b_slider = Slider::new("Semi-minor axis (r_b)", screen_width() / 4.0, 10.0, screen_width(), 20.0, 80.0);
    let mut n_slider = Slider::new("Roundedness (n)", 4.0, 0.1, 12.0, 20.0, 120.0);
    let mut thickness_slider = Slider::new("Thickness", 2.0, 0.1, 40.0, 20.0, 160.0);
    let mut steps_slider = Slider::new("Steps", 25.0, 1.0, 50.0, 20.0, 200.0);
    let mut rotation_slider = Slider::new("Rotation", 0.0, 0.0, 180.0, 20.0, 240.0);
    
    let mut fill = false; // whether to fill the shape or just draw the outline

    let mut steps: usize = 25; // we need to cast steps to usize often, so we'll keep it around
    
    // plot the shape using macroquad
    loop {
        // take inputs
            // all these update would require recalculating the vertex buffer
            vertices_need_recalculation =
                vertices_need_recalculation || // this is here to ensure we recalc on first frame
                r_a_slider.update() || 
                r_b_slider.update() || 
                n_slider.update() || 
                steps_slider.update();

            thickness_slider.update();
            // set rotation
            if rotation_slider.update() {
                let rotation_degrees = rotation_slider.value;
                let rotation_radians = rotation_degrees * std::f32::consts::PI / 180.0;
                rotation_matrix_trig_values = rotation_radians.sin_cos().into();
            }
            // toggle fill mode on key press
            if is_key_pressed(KeyCode::Space) {
                fill = !fill;
            }

        if vertices_need_recalculation {
            let r_a = r_a_slider.value;
            let r_b = r_b_slider.value;
            let n = n_slider.value;       
            steps = steps_slider.value as usize;
            vertex_buffer.clear();
            vertex_buffer.push(vec2(r_a, 0.0));

            for i in 1..steps {
                let t = (std::f64::consts::FRAC_PI_2) * (i as f64) / (steps as f64);
                // Explicitly handle the quadrant edges to avoid floating point errors
                // creating gaps when 'n' is large.
                let (sin_t, cos_t) = t.sin_cos(); // optimisation
                let exponent = 2.0 / n;
                
                // let x = r_a * cos_t.abs().powf(exponent) * cos_t.signum();
                // let y = r_b * sin_t.abs().powf(exponent) * sin_t.signum();
                // we can drop the abs() and signum() calls since t is strictly in the first quadrant
                let x = r_a * (cos_t as f32).powf(exponent);
                let y = r_b * (sin_t as f32).powf(exponent);
                
                vertex_buffer.push(vec2(x as f32, y as f32));        
            }
            vertex_buffer.push(vec2(0.0, r_b_slider.value as f32));
            vertices_need_recalculation = false;
        }   

        clear_background(DARKGRAY);
        // draw the shape by connecting the vertices
        let centre: Vec2 = vec2(screen_width() / 2.0, screen_height() / 2.0);

        // draw all the quadrants
        // using a generic F allows us to pass a closure
        if fill {
            render_quadrants(centre, steps, &vertex_buffer, &quadrants, rotation_matrix_trig_values, |prev, curr| {
                draw_triangle(centre, prev, curr, WHITE);
            });
        } else {
            render_quadrants(centre, steps, &vertex_buffer, &quadrants, rotation_matrix_trig_values, |prev, curr| {
                draw_line(prev.x, prev.y, curr.x, curr.y, thickness_slider.value, WHITE);
            });
        }

        // draw sliders
            r_a_slider.draw();
            r_b_slider.draw();
            n_slider.draw();
            thickness_slider.draw();
            steps_slider.draw();
            rotation_slider.draw();
        // draw fill mode text
            let fill_text = if fill { "Fill: ON (press SPACE to toggle)" } else { "Fill: OFF (press SPACE to toggle)" };
            draw_text(fill_text, 20.0, 280.0, 20.0, WHITE);
        next_frame().await

    }
}


// changes made
// - add ability to fill the shape
// - make draw logic draw each quarter at a time, and just index the buffer differently
// - when calculating vertex edge midpoints, you know the x or y coordinate is 0,
//            so can avoid power calculations there. this also fixes gaps due to
//            floating point incaccuracy when n is large.
// - optimise the power calculations by precomputing 2.0/n
// - add rotation to the shape
// - make slider struct logic neater
// - only recalculated the vertex buffer if a relevant slider value has been changed
// - change all f64 to f32 where possible as the precision isnt needed