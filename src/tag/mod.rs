mod tag;
pub use tag::*;

mod id;
pub use id::*;

mod refs;
pub use refs::*;

// These modules contain things like Iter, IntoIter, etc. which will clash
// So instead we make them public and only delegate their essentials, like std does too
pub mod list;
pub use list::TagList;
pub use list::tag_list;

pub mod map;
pub use map::TagMap;
pub use map::tag_map;

// this used to be in this module
pub(self) use super::unsafe_ops;
