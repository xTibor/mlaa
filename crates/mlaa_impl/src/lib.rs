pub enum Gradient<C> {
    Vertical {
        x: f32,
        y: f32,
        height: f32,
        colors: (C, C),
    },
    Horizontal {
        x: f32,
        y: f32,
        width: f32,
        colors: (C, C),
    },
}

pub fn mlaa<P, C, G>(
    image_width: usize,
    image_height: usize,
    image_pixels: P,
    seam_split_position: f32,
    mut emit_gradient: G,
) where
    P: Fn(isize, isize) -> C,
    C: PartialEq + Copy + Clone,
    G: FnMut(Gradient<C>),
{
    let vertical_run = |x: isize, y: isize, pred: Box<dyn Fn((C, C)) -> bool>| -> isize {
        let mut run_length = 0;

        while (y + run_length < image_height as isize)
            && pred((image_pixels(x, y + run_length), image_pixels(x + 1, y + run_length)))
        {
            run_length += 1;
        }

        run_length
    };

    let horizontal_run = |x: isize, y: isize, pred: Box<dyn Fn((C, C)) -> bool>| -> isize {
        let mut run_length = 0;

        while (x + run_length < image_width as isize)
            && pred((image_pixels(x + run_length, y), image_pixels(x + run_length, y + 1)))
        {
            run_length += 1;
        }

        run_length
    };

    for x in -1..image_width as isize {
        let mut y = 0;
        y += vertical_run(x, y, Box::new(|(c1, c2)| c1 == c2));

        while y < image_height as isize {
            let seam_colors = (image_pixels(x, y), image_pixels(x + 1, y));
            let seam_length = vertical_run(x, y, Box::new(move |c| c == seam_colors));

            'neighbor_loop: for neighbor_delta in [-1, 1] {
                let neighbor_length =
                    vertical_run(x + neighbor_delta, y + seam_length, Box::new(move |c| c == seam_colors));

                if neighbor_length > 0 {
                    let gradient_x = x.max(x + neighbor_delta) as f32;

                    let gradient_y =
                        (y as f32) + (seam_length as f32 / 2.0) + (seam_length as f32 / 2.0 * seam_split_position);

                    let gradient_length = (seam_length as f32 / 2.0) + (neighbor_length as f32 / 2.0)
                        - (seam_length as f32 / 2.0 * seam_split_position)
                        - (neighbor_length as f32 / 2.0 * seam_split_position);

                    let gradient_colors = if neighbor_delta < 0 {
                        (seam_colors.0, seam_colors.1)
                    } else {
                        (seam_colors.1, seam_colors.0)
                    };

                    emit_gradient(Gradient::Vertical {
                        x: gradient_x,
                        y: gradient_y,
                        height: gradient_length,
                        colors: gradient_colors,
                    });

                    break 'neighbor_loop;
                }
            }

            y += seam_length;
            y += vertical_run(x, y, Box::new(|(c1, c2)| c1 == c2));
        }
    }

    for y in -1..image_height as isize {
        let mut x = 0;
        x += horizontal_run(x, y, Box::new(|(c1, c2)| c1 == c2));

        while x < image_width as isize {
            let seam_colors = (image_pixels(x, y), image_pixels(x, y + 1));
            let seam_length = horizontal_run(x, y, Box::new(move |c| c == seam_colors));

            'neighbor_loop: for neighbor_delta in [-1, 1] {
                let neighbor_length =
                    horizontal_run(x + seam_length, y + neighbor_delta, Box::new(move |c| c == seam_colors));

                if neighbor_length > 0 {
                    let gradient_y = y.max(y + neighbor_delta) as f32;

                    let gradient_x =
                        (x as f32) + (seam_length as f32 / 2.0) + (seam_length as f32 / 2.0 * seam_split_position);

                    let gradient_length = (seam_length as f32 / 2.0) + (neighbor_length as f32 / 2.0)
                        - (seam_length as f32 / 2.0 * seam_split_position)
                        - (neighbor_length as f32 / 2.0 * seam_split_position);

                    let gradient_colors = if neighbor_delta < 0 {
                        (seam_colors.0, seam_colors.1)
                    } else {
                        (seam_colors.1, seam_colors.0)
                    };

                    emit_gradient(Gradient::Horizontal {
                        x: gradient_x,
                        y: gradient_y,
                        width: gradient_length,
                        colors: gradient_colors,
                    });

                    break 'neighbor_loop;
                }
            }

            x += seam_length;
            x += horizontal_run(x, y, Box::new(|(c1, c2)| c1 == c2));
        }
    }
}
