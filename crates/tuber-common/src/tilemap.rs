use std::collections::HashSet;

pub struct Tilemap {
    pub width: usize,
    pub height: usize,
    pub tile_width: usize,
    pub tile_height: usize,
    pub tiles: Vec<Tile>,
}

impl Tilemap {
    pub fn new(
        width: usize,
        height: usize,
        tile_width: usize,
        tile_height: usize,
        default_tags: &[String],
    ) -> Self {
        Self {
            width,
            height,
            tile_width,
            tile_height,
            tiles: vec![Tile::with_tags(default_tags); width * height],
        }
    }
}

#[derive(Clone)]
pub struct Tile {
    pub tags: HashSet<String>,
}

impl Tile {
    pub fn with_tags(tags: &[String]) -> Self {
        Self {
            tags: tags.iter().cloned().map(|s| s.to_owned()).collect(),
        }
    }
}
