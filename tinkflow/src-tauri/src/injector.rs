use enigo::{Enigo, Keyboard, Settings};

pub struct TextInjector {
    enigo: Enigo,
}

impl TextInjector {
    pub fn new() -> Result<Self, String> {
        let enigo = Enigo::new(&Settings::default()).map_err(|e| e.to_string())?;
        Ok(Self { enigo })
    }

    pub fn inject(&mut self, text: &str) -> Result<(), String> {
        // Enigo v0.6 handles text strings nicely using text()
        self.enigo.text(text).map_err(|e| e.to_string())?;
        Ok(())
    }
}
