use eframe::egui::{self, DragValue, PointerButton, Sense};
use eframe::emath::{lerp, remap};
use eframe::epaint::{vec2, Color32, Rect, Rgba, Stroke};

const IMAGE_WIDTH: usize = 32;
const IMAGE_HEIGHT: usize = 24;

pub struct Gradient {
    pub x: f32,
    pub y: f32,
    pub length: f32,
    pub colors: (Color32, Color32),
}

struct MlaaApplication {
    selected_color: Color32,
    image_pixels: [[Color32; IMAGE_WIDTH]; IMAGE_HEIGHT],

    seam_split_position: f32,

    show_vertical_outlines: bool,
    show_vertical_gradients: bool,
    vertical_gradients: Vec<Gradient>,

    show_horizontal_outlines: bool,
    show_horizontal_gradients: bool,
    horizontal_gradients: Vec<Gradient>,
}

impl Default for MlaaApplication {
    fn default() -> MlaaApplication {
        let mut mlaa_application = MlaaApplication {
            selected_color: Color32::BLACK,
            image_pixels: Default::default(),

            seam_split_position: 0.0,

            show_vertical_outlines: true,
            show_vertical_gradients: false,
            vertical_gradients: Vec::new(),

            show_horizontal_outlines: true,
            show_horizontal_gradients: false,
            horizontal_gradients: Vec::new(),
        };

        mlaa_application.generate_test_image();
        mlaa_application.recalculate_seams();
        mlaa_application
    }
}

impl MlaaApplication {
    fn generate_test_image(&mut self) {
        let aspect_ratio = if IMAGE_WIDTH > IMAGE_HEIGHT {
            vec2(IMAGE_WIDTH as f32 / IMAGE_HEIGHT as f32, 1.0)
        } else {
            vec2(1.0, IMAGE_HEIGHT as f32 / IMAGE_WIDTH as f32)
        };

        for y in 0..IMAGE_HEIGHT {
            for x in 0..IMAGE_WIDTH {
                let v = vec2(
                    remap(x as f32, 0.0..=(IMAGE_WIDTH as f32), -1.0..=1.0),
                    remap(y as f32, 0.0..=(IMAGE_HEIGHT as f32), -1.0..=1.0),
                ) * aspect_ratio;

                self.image_pixels[y][x] = if v.length() <= 0.9 && v.length() >= 0.5 {
                    Color32::BLACK
                } else {
                    Color32::WHITE
                };
            }
        }
    }

    fn pixel(&self, x: isize, y: isize) -> Color32 {
        if (x < 0) || (x >= IMAGE_WIDTH as isize) {
            return Color32::TRANSPARENT;
        }

        if (y < 0) || (y >= IMAGE_HEIGHT as isize) {
            return Color32::TRANSPARENT;
        }

        self.image_pixels[y as usize][x as usize]
    }

    fn vertical_run<P>(&self, x: isize, y: isize, pred: P) -> isize
    where
        P: Fn((Color32, Color32)) -> bool,
    {
        let mut run_length = 0;

        while (y + run_length < IMAGE_HEIGHT as isize)
            && pred((self.pixel(x, y + run_length), self.pixel(x + 1, y + run_length)))
        {
            run_length += 1;
        }

        run_length
    }

    fn horizontal_run<P>(&self, x: isize, y: isize, pred: P) -> isize
    where
        P: Fn((Color32, Color32)) -> bool,
    {
        let mut run_length = 0;

        while (x + run_length < IMAGE_WIDTH as isize)
            && pred((self.pixel(x + run_length, y), self.pixel(x + run_length, y + 1)))
        {
            run_length += 1;
        }

        run_length
    }

    fn recalculate_seams(&mut self) {
        self.vertical_gradients.clear();
        self.horizontal_gradients.clear();

        for x in -1..IMAGE_WIDTH as isize {
            let mut y = 0;
            y += self.vertical_run(x, y, |(c1, c2)| c1 == c2);

            while y < IMAGE_HEIGHT as isize {
                let seam_colors = (self.pixel(x, y), self.pixel(x + 1, y));
                let seam_length = self.vertical_run(x, y, |c| c == seam_colors);

                'neighbor_loop: for neighbor_delta in [-1, 1] {
                    let neighbor_length = self.vertical_run(x + neighbor_delta, y + seam_length, |c| c == seam_colors);

                    if neighbor_length > 0 {
                        let gradient_x = x.max(x + neighbor_delta) as f32;

                        let gradient_y = (y as f32)
                            + (seam_length as f32 / 2.0)
                            + (seam_length as f32 / 2.0 * self.seam_split_position);

                        let gradient_length = (seam_length as f32 / 2.0) + (neighbor_length as f32 / 2.0)
                            - (seam_length as f32 / 2.0 * self.seam_split_position)
                            - (neighbor_length as f32 / 2.0 * self.seam_split_position);

                        let gradient_colors = if neighbor_delta < 0 {
                            (seam_colors.0, seam_colors.1)
                        } else {
                            (seam_colors.1, seam_colors.0)
                        };

                        self.vertical_gradients.push(Gradient {
                            x: gradient_x,
                            y: gradient_y,
                            length: gradient_length,
                            colors: gradient_colors,
                        });

                        break 'neighbor_loop;
                    }
                }

                y += seam_length;
                y += self.vertical_run(x, y, |(c1, c2)| c1 == c2);
            }
        }

        for y in -1..IMAGE_HEIGHT as isize {
            let mut x = 0;
            x += self.horizontal_run(x, y, |(c1, c2)| c1 == c2);

            while x < IMAGE_WIDTH as isize {
                let seam_colors = (self.pixel(x, y), self.pixel(x, y + 1));
                let seam_length = self.horizontal_run(x, y, |c| c == seam_colors);

                'neighbor_loop: for neighbor_delta in [-1, 1] {
                    let neighbor_length =
                        self.horizontal_run(x + seam_length, y + neighbor_delta, |c| c == seam_colors);

                    if neighbor_length > 0 {
                        let gradient_y = y.max(y + neighbor_delta) as f32;

                        let gradient_x = (x as f32)
                            + (seam_length as f32 / 2.0)
                            + (seam_length as f32 / 2.0 * self.seam_split_position);

                        let gradient_length = (seam_length as f32 / 2.0) + (neighbor_length as f32 / 2.0)
                            - (seam_length as f32 / 2.0 * self.seam_split_position)
                            - (neighbor_length as f32 / 2.0 * self.seam_split_position);

                        let gradient_colors = if neighbor_delta < 0 {
                            (seam_colors.0, seam_colors.1)
                        } else {
                            (seam_colors.1, seam_colors.0)
                        };

                        self.horizontal_gradients.push(Gradient {
                            x: gradient_x,
                            y: gradient_y,
                            length: gradient_length,
                            colors: gradient_colors,
                        });

                        break 'neighbor_loop;
                    }
                }

                x += seam_length;
                x += self.horizontal_run(x, y, |(c1, c2)| c1 == c2);
            }
        }
    }
}

impl eframe::App for MlaaApplication {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.color_edit_button_srgba(&mut self.selected_color);
                ui.separator();

                ui.vertical(|ui| {
                    ui.label("New image");

                    if ui.button("Test image").clicked() {
                        self.generate_test_image();
                        self.recalculate_seams();
                    }

                    if ui.button("Blank image").clicked() {
                        self.image_pixels = [[Color32::WHITE; IMAGE_WIDTH]; IMAGE_HEIGHT];
                        self.recalculate_seams();
                    }
                });
                ui.separator();

                ui.vertical(|ui| {
                    ui.label("Split position");

                    let drag_value = DragValue::new(&mut self.seam_split_position)
                        .clamp_range(0.0..=1.0)
                        .speed(0.01);
                    if ui.add(drag_value).changed() {
                        self.recalculate_seams();
                    }
                });
                ui.separator();

                ui.vertical(|ui| {
                    ui.label("Outlines");
                    ui.checkbox(&mut self.show_vertical_outlines, "Vertical");
                    ui.checkbox(&mut self.show_horizontal_outlines, "Horizontal");
                });
                ui.separator();

                ui.vertical(|ui| {
                    ui.label("Gradients");
                    ui.checkbox(&mut self.show_vertical_gradients, "Vertical");
                    ui.checkbox(&mut self.show_horizontal_gradients, "Horizontal");
                });
                ui.separator();
            });

            ui.separator();

            ui.scope(|ui| {
                let cell_size = vec2(24.0, 24.0);
                let mut needs_seam_recalculation = false;

                // Draw widget base
                let widget_size = cell_size * vec2(IMAGE_WIDTH as f32, IMAGE_HEIGHT as f32);
                let (rect, _response) = ui.allocate_exact_size(widget_size, Sense::hover());
                ui.painter().rect_filled(rect, 0.0, Color32::GRAY);

                // Draw pixels
                for y in 0..IMAGE_HEIGHT {
                    for x in 0..IMAGE_WIDTH {
                        let pixel_rect =
                            Rect::from_min_size(rect.left_top() + cell_size * vec2(x as f32, y as f32), cell_size);

                        let pixel_response = ui.allocate_rect(pixel_rect, Sense::click_and_drag());

                        if pixel_response.clicked_by(PointerButton::Primary) {
                            self.image_pixels[y][x] = self.selected_color;
                            needs_seam_recalculation = true;
                        }

                        if pixel_response.clicked_by(PointerButton::Secondary) {
                            self.selected_color = self.image_pixels[y][x];
                        }

                        ui.painter()
                            .rect_filled(pixel_rect.shrink(1.0), 0.0, self.image_pixels[y][x]);
                    }
                }

                // Recalculate seams if neccessary
                if needs_seam_recalculation {
                    self.recalculate_seams();
                }

                // Draw gradients
                {
                    if self.show_vertical_gradients {
                        for gradient in &self.vertical_gradients {
                            let y1 = gradient.y.floor() as usize;
                            let y2 = (gradient.y + gradient.length).ceil() as usize;
                            let x = gradient.x as usize;

                            for y in y1..y2 {
                                let pixel_rect = Rect::from_min_size(
                                    rect.left_top() + cell_size * vec2(x as f32, y as f32),
                                    cell_size,
                                );

                                let color = lerp(
                                    Rgba::from(gradient.colors.0)..=Rgba::from(gradient.colors.1),
                                    remap(y as f32 + 0.5, y1 as f32..=y2 as f32, 0.0..=1.0),
                                );

                                ui.painter().rect_filled(pixel_rect.shrink(1.0), 0.0, color);
                            }
                        }
                    }

                    if self.show_horizontal_gradients {
                        for gradient in &self.horizontal_gradients {
                            let x1 = gradient.x.floor() as usize;
                            let x2 = (gradient.x + gradient.length).ceil() as usize;
                            let y = gradient.y as usize;

                            for x in x1..x2 {
                                let pixel_rect = Rect::from_min_size(
                                    rect.left_top() + cell_size * vec2(x as f32, y as f32),
                                    cell_size,
                                );

                                let color = lerp(
                                    Rgba::from(gradient.colors.0)..=Rgba::from(gradient.colors.1),
                                    remap(x as f32 + 0.5, x1 as f32..=x2 as f32, 0.0..=1.0),
                                );

                                ui.painter().rect_filled(pixel_rect.shrink(1.0), 0.0, color);
                            }
                        }
                    }
                }

                // Draw gradient outlines
                {
                    if self.show_vertical_outlines {
                        for gradient in &self.vertical_gradients {
                            let gradient_rect = Rect::from_min_size(
                                rect.left_top() + cell_size * vec2(gradient.x, gradient.y),
                                cell_size * vec2(1.0, gradient.length),
                            );

                            let color = Color32::GREEN;
                            let stroke_thin = Stroke { width: 2.0, color };
                            let stroke_bold = Stroke { width: 3.0, color };

                            ui.painter().rect_stroke(gradient_rect, 0.0, stroke_thin);

                            ui.painter()
                                .line_segment([gradient_rect.center_top(), gradient_rect.center_bottom()], stroke_bold);

                            ui.painter()
                                .circle(gradient_rect.center_top(), 4.0, gradient.colors.0, stroke_thin);
                            ui.painter()
                                .circle(gradient_rect.center_bottom(), 4.0, gradient.colors.1, stroke_thin);
                        }
                    }

                    if self.show_horizontal_outlines {
                        for gradient in &self.horizontal_gradients {
                            let gradient_rect = Rect::from_min_size(
                                rect.left_top() + cell_size * vec2(gradient.x, gradient.y),
                                cell_size * vec2(gradient.length, 1.0),
                            );

                            let color = Color32::YELLOW;
                            let stroke_thin = Stroke { width: 2.0, color };
                            let stroke_bold = Stroke { width: 3.0, color };

                            ui.painter().rect_stroke(gradient_rect, 0.0, stroke_thin);

                            ui.painter()
                                .line_segment([gradient_rect.left_center(), gradient_rect.right_center()], stroke_bold);

                            ui.painter()
                                .circle(gradient_rect.left_center(), 4.0, gradient.colors.0, stroke_thin);
                            ui.painter()
                                .circle(gradient_rect.right_center(), 4.0, gradient.colors.1, stroke_thin);
                        }
                    }
                }
            })
        });
    }
}

fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        initial_window_size: Some(vec2(800.0, 640.0)),
        ..Default::default()
    };

    eframe::run_native("MLAA", options, Box::new(|_| Box::<MlaaApplication>::default()))
}
