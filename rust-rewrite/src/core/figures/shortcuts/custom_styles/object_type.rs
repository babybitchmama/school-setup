#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ObjectType {
    Rect,
    Ellipse,
    /// Covers freehand/bezier paths and plain lines -- both are `<path>`
    /// once drawn in Inkscape.
    Path,
    Text,
}

impl ObjectType {
    pub fn from_tag_name(tag: &str) -> Option<Self> {
        let local = tag.rsplit(':').next().unwrap_or(tag);
        match local {
            "rect" => Some(ObjectType::Rect),
            "circle" | "ellipse" => Some(ObjectType::Ellipse),
            "path" | "line" | "polyline" | "polygon" => Some(ObjectType::Path),
            "text" | "flowRoot" => Some(ObjectType::Text),
            _ => None,
        }
    }
}
