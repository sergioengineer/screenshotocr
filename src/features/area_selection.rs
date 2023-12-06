use super::Feature;
use crate::application;
use crate::types;
use gtk;
use gtk::cairo;
use gtk::glib;
use gtk::prelude::WidgetExt;
use std::rc::Rc;
use std::sync::{Arc, RwLock};

struct Selecting(bool);
pub struct AreaSelectionFeature {
    selecting: Arc<RwLock<Selecting>>,
    selected_area: Arc<RwLock<Option<Rc<types::Area>>>>,
}

impl Default for AreaSelectionFeature {
    fn default() -> Self {
        Self {
            selecting: Arc::new(RwLock::new(Selecting(false))),
            selected_area: Default::default(),
        }
    }
}

pub fn add_area_selection_feature(
    instance: Arc<application::ApplicationInstance>,
) -> Arc<application::ApplicationInstance> {
    let feature = Arc::new(AreaSelectionFeature::default());
    instance
        .window
        .connect_draw(draw_selected_area(feature.clone()));

    instance
        .window
        .connect_motion_notify_event(record_area_selection(feature.clone()));

    instance
        .window
        .connect_button_release_event(stop_recording_area_selection(
            feature.clone(),
            instance.clone(),
        ));

    instance.features.write().unwrap().insert(
        super::Features::AreaSelection as usize,
        Feature::OcrAreaSelection(feature),
    );

    instance
}

fn draw_selected_area(
    feature: Arc<AreaSelectionFeature>,
) -> impl Fn(&gtk::ApplicationWindow, &cairo::Context) -> glib::Propagation {
    return move |_app, context| {
        if feature.selected_area.read().unwrap().is_none() {
            return glib::Propagation::Proceed;
        }

        let area = feature.selected_area.read().unwrap().clone().unwrap();
        let start_x = *vec![area.start.x, area.end.x]
            .iter()
            .min_by(|a, b| a.total_cmp(b))
            .unwrap();
        let start_y = *vec![area.start.y, area.end.y]
            .iter()
            .min_by(|a, b| a.total_cmp(b))
            .unwrap();

        context.set_source_rgb(32. / 255., 125. / 255., 170. / 255.);
        context.set_line_width(3.);
        context.rectangle(start_x, start_y, area.get_width(), area.get_height());

        let _ = context.stroke();
        glib::Propagation::Proceed
    };
}

fn record_area_selection(
    feature: Arc<AreaSelectionFeature>,
) -> impl Fn(&gtk::ApplicationWindow, &gtk::gdk::EventMotion) -> glib::Propagation {
    return move |app, event| {
        let mut selecting = feature.selecting.write().unwrap();
        let original_area = &feature.selected_area.clone().read().unwrap().clone();
        let mut area = feature.selected_area.write().unwrap();

        let start = if selecting.0 == false || original_area.is_none() {
            types::Position {
                x: event.position().0,
                y: event.position().1,
            }
        } else {
            types::Position {
                x: original_area.as_ref().unwrap().start.x,
                y: original_area.as_ref().unwrap().start.y,
            }
        };
        let end = types::Position {
            x: event.position().0,
            y: event.position().1,
        };

        let _ = area.insert(Rc::new(types::Area { start, end }));
        selecting.0 = true;

        app.queue_draw();
        gtk::glib::Propagation::Proceed
    };
}

fn stop_recording_area_selection(
    feature: Arc<AreaSelectionFeature>,
    instance: Arc<application::ApplicationInstance>,
) -> impl Fn(&gtk::ApplicationWindow, &gtk::gdk::EventButton) -> glib::Propagation {
    return move |app, _event| {
        let mut selecting = feature.selecting.write().unwrap();
        let area_option = feature.selected_area.read().unwrap().clone();
        selecting.0 = false;

        if let Some(area) = area_option {
            instance.dispatch(application::Event::AreaSelectionCompleted(*area))
        }
        app.queue_draw();
        gtk::glib::Propagation::Proceed
    };
}
