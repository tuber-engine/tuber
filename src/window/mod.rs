pub trait Window {
    fn title(&self) -> &str;
    fn set_title(&mut self, title: &str);

    fn width(&self) -> u32;
    fn set_width(&mut self, width: u32);

    fn height(&self) -> u32;
    fn set_height(&mut self, height: u32);
}
