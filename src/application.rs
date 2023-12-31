use crate::features;
use crate::types;
use gtk::gdk;
use gtk::gdk_pixbuf;
use gtk::glib;
use gtk::prelude::ContainerExt as _;
use gtk::prelude::GdkContextExt as _;
use gtk::prelude::GtkWindowExt as _;
use gtk::prelude::WidgetExt as _;
use gtk::prelude::WindowExtManual as _;
use rusty_tesseract::image;
use std::sync::{Arc, RwLock};

type Features = Arc<RwLock<Vec<features::Feature>>>;

pub struct ApplicationInstance {
    pub window: gtk::ApplicationWindow,
    pub screenshot: gdk_pixbuf::Pixbuf,
    pub features: Features,
}

unsafe impl Send for ApplicationInstance {}
unsafe impl Sync for ApplicationInstance {}

pub enum Event {
    AreaSelectionCompleted(types::Area),
}

impl ApplicationInstance {
    pub fn dispatch(&self, event: Event) {
        match event {
            Event::AreaSelectionCompleted(area) => {
                let new_pix = gdk_pixbuf::Pixbuf::new(
                    gdk_pixbuf::Colorspace::Rgb,
                    true,
                    8,
                    area.get_width() as i32,
                    area.get_height() as i32,
                )
                .unwrap();

                let start_x = *[area.start.x, area.end.x]
                    .iter()
                    .min_by(|a, b| a.total_cmp(b))
                    .unwrap();
                let start_y = *[area.start.y, area.end.y]
                    .iter()
                    .min_by(|a, b| a.total_cmp(b))
                    .unwrap();

                self.screenshot.copy_area(
                    start_x as i32,
                    start_y as i32,
                    area.get_width() as i32,
                    area.get_height() as i32,
                    &new_pix,
                    0,
                    0,
                );

                let buffer = new_pix.read_pixel_bytes().to_vec();

                let image_buffer = image::ImageBuffer::from_vec(
                    new_pix.width() as u32,
                    new_pix.height() as u32,
                    buffer,
                )
                .unwrap();

                let dynamic_image: image::DynamicImage =
                    image::DynamicImage::ImageRgba8(image_buffer);
                let img = rusty_tesseract::Image::from_dynamic_image(&dynamic_image).unwrap();
                let my_args = rusty_tesseract::Args {
                    lang: "eng".to_owned(),
                    config_variables: std::collections::HashMap::from([(
                       "tessedit_char_whitelist".to_owned(),
                       "éãúabcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789:-_ ();,.[]{}´'\"`!-=+*/%$#@&|\\/<>	".to_owned()
                    )]),
                    dpi: Some(150),
                    psm: Some(6),
                    oem: Some(3),
                };
                let output = rusty_tesseract::image_to_string(&img, &my_args).unwrap();

                println!("{}", output);
                let mut clip = arboard::Clipboard::new().unwrap();
                clip.set_text(output).unwrap();
                self.window.close();
            }
        }
    }
}

pub fn build_application(application: &gtk::Application) {
    generate_window(application)
        .map(window_to_instance)
        .map(adjust_features_vec_size)
        .map(draw_background_screenshot)
        .map(features::area_selection::add_area_selection_feature);
}

pub fn generate_window(application: &gtk::Application) -> Option<gtk::ApplicationWindow> {
    let window = gtk::ApplicationWindow::new(application);

    window.set_app_paintable(true);
    window.set_title("ScreenSHOT - OCR");
    window.set_border_width(0);
    window.set_position(gtk::WindowPosition::Center);
    window.fullscreen();
    window.show_all();

    Some(window)
}

fn window_to_instance(window: gtk::ApplicationWindow) -> Arc<ApplicationInstance> {
    let instance = ApplicationInstance {
        screenshot: get_screenshot(),
        window,
        features: Features::default(),
    };

    Arc::new(instance)
}

fn adjust_features_vec_size(instance: Arc<ApplicationInstance>) -> Arc<ApplicationInstance> {
    instance
        .features
        .write()
        .unwrap()
        .resize_with(20, || features::Feature::Disabled);

    instance
}

fn draw_background_screenshot(instance: Arc<ApplicationInstance>) -> Arc<ApplicationInstance> {
    let inst = instance.clone();
    let _handler_id = instance.window.connect_draw(move |_, context| {
        context.set_source_pixbuf(inst.screenshot.as_ref(), 0., 0.);
        let _ = context.paint();
        glib::Propagation::Proceed
    });

    instance
}

fn get_screenshot() -> gdk_pixbuf::Pixbuf {
    let root_window = gdk::Window::default_root_window();
    let geo = root_window.geometry();
    let pix = Box::new(root_window.pixbuf(geo.0, geo.1, geo.2, geo.3).unwrap());

    *pix
}
