use tuber_common::tilemap::Tile;

pub struct TilemapRender {
    pub identifier: String,
    pub texture_atlas_identifier: String,
    pub tile_texture_function: Box<dyn Fn(&Tile) -> Option<&str>>,
    pub dirty: bool,
}
