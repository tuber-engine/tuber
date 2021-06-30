use crate::*;

/// The low level API
pub trait LowLevelGraphicsAPI {
    /// Initializes the API for a given window
    fn initialize(&mut self, window: Window, window_size: WindowSize);
    /// Renders
    fn render(&mut self);

    /// Prepares the render of a quad
    fn prepare_quad(
        &mut self,
        quad_description: &QuadDescription,
        transform: &Transform2D,
        bounding_box_rendering: bool,
    );
    fn prepare_tilemap(
        &mut self,
        tilemap: &Tilemap,
        tilemap_render: &TilemapRender,
        texture_atlas: &TextureAtlas,
        transform: &Transform2D,
    );
    fn is_texture_in_memory(&self, texture_identifier: &str) -> bool;
    /// Loads a texture in memory
    fn load_texture(&mut self, texture_data: TextureData);
    /// Updates the view/projection matrix
    fn update_camera(
        &mut self,
        camera_id: EntityIndex,
        camera: &OrthographicCamera,
        transform: &Transform2D,
    );

    fn set_clear_color(&mut self, color: Color);
    fn on_window_resized(&mut self, size: WindowSize);
}

/// Describes a vertex for the low-level renderer
pub struct VertexDescription {
    /// The position in Normalized Device Coordinates
    pub position: (f32, f32, f32),
    /// The color of the vertex
    pub color: (f32, f32, f32),
    /// The normalized texture coordinates of the vertex
    pub texture_coordinates: (f32, f32),
}

pub struct TextureDescription {
    /// The identifier of the texture
    pub identifier: String,
    /// The region of the texture to use
    pub texture_region: TextureRegion,
}

/// Describes a quad for the low-level renderer
pub struct QuadDescription {
    /// Width in Normalized Device Coordinates
    pub width: f32,
    /// Height in Normalized Device Coordinates
    pub height: f32,
    /// The color used for the quad's vertices
    pub color: Color,
    /// The texture of the quad
    pub texture: Option<TextureDescription>,
}

/// Describes a mesh for the low-leven renderer
pub struct MeshDescription {
    /// The vertices of the mesh
    pub vertices: Vec<VertexDescription>,
    pub texture: TextureDescription,
}

pub struct TilemapDescription {
    pub tiles: Vec<Vec<Option<TileDescription>>>,
    pub texture: TextureDescription,
}

pub struct TileDescription;
