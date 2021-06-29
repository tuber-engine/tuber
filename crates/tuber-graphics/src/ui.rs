pub struct Text {
    text: String,
    font: String,
}

impl Text {
    pub fn new(text: &str, font: &str) -> Self {
        Self {
            text: text.into(),
            font: font.into()
        }
    }

    pub fn text(&self) -> &str {
        &self.text
    }

    pub fn font(&self) -> &str {
        &self.font
    }
}