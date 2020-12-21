use conrod_core::{widget, widget_ids, Sizeable, UiBuilder, UiCell, Widget};
use piston_window::{Flip, G2dTexture, PistonWindow, Texture, TextureSettings, UpdateEvent, Window, WindowSettings};

static WIDTH: u32 = 128;
static HEIGHT: u32 = 128;

#[derive(Debug)]
struct State {
    image: conrod_core::image::Id,
}

widget_ids! {
    pub struct Ids {
        image
    }
}

fn init_gui() {
    let mut window: PistonWindow =
        WindowSettings::new("All Widgets - Piston Backend", (WIDTH, HEIGHT))
            .samples(4)
            .exit_on_esc(true)
            .vsync(true)
            .decorated(false)
            .transparent(true)
            .build()
            .unwrap();

    // construct our `Ui`.
    let mut ui = UiBuilder::new([WIDTH as f64, HEIGHT as f64]).build();

    // Instantiate the generated list of widget identifiers.
    let ids = Ids::new(ui.widget_id_generator());

    // Create texture context to perform operations on textures.
    let mut texture_context = window.create_texture_context();

    // Load the rust logo from file to a piston_window texture.
    let image: G2dTexture = {
        let path = "widewalk.gif";
        let settings = TextureSettings::new();
        Texture::from_path(&mut texture_context, &path, Flip::None, &settings).unwrap()
    };

    // Create our `conrod_core::image::Map` which describes each of our widget->image mappings.
    let mut image_map = conrod_core::image::Map::new();
    let image = image_map.insert(image);

    let mut state = State { image };

    // Poll events from the window.
    while let Some(event) = window.next() {
        // Convert the src event to a conrod event.
        let size = window.size();
        let (win_w, win_h) = (
            size.width as conrod_core::Scalar,
            size.height as conrod_core::Scalar,
        );
        if let Some(e) = conrod_piston::event::convert(event.clone(), win_w, win_h) {
            ui.handle_event(e);
        }

        event.update(|_| {
            let mut ui = ui.set_widgets();
            draw_gui(&mut ui, &ids, &mut state);
        });

        window.draw_2d(&event, |context, graphics, device| {
            if let Some(primitives) = ui.draw_if_changed() {

                // Specify how to get the drawable texture from the image. In this case, the image
                // *is* the texture.
                fn texture_from_image<T>(img: &T) -> &T {
                    img
                }

                // Draw the conrod `render::Primitives`.
                conrod_piston::draw::primitives(
                    primitives,
                    context,
                    graphics,
                    &mut text_texture_cache,
                    &mut glyph_cache,
                    &image_map,
                    cache_queued_glyphs,
                    texture_from_image,
                );

                texture_context.encoder.flush(device);
            }
        });
    }
}

fn draw_gui(ui: &mut UiCell, ids: &Ids, state: &mut State) {
    widget::Image::new(state.image)
        .w_h(WIDTH as f64, HEIGHT as f64)
        .set(ids.image, ui);
}
