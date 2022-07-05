//! [ICalObject] implements FromStr and Display, see its docs and its source

pub mod content_line;
pub mod fold;
pub mod ical_object;
pub mod unfold;

pub use content_line::{ContentLine, Param};
pub use fold::fold;
pub use ical_object::ICalObject;
pub use unfold::Unfold;
