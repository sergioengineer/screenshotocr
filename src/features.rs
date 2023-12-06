use std::sync::Arc;

use self::area_selection::AreaSelectionFeature;
pub mod area_selection;

pub enum Feature {
    Disabled,
    OcrAreaSelection(Arc<AreaSelectionFeature>),
}

pub enum Features {
    AreaSelection = 0,
}
