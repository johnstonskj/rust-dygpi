use dygpi::plugin::Plugin;

// ------------------------------------------------------------------------------------------------
// Public Types
// ------------------------------------------------------------------------------------------------

#[derive(Debug, Default)]
pub struct SoundEngine;

#[derive(Debug, Default)]
pub struct MediaStream;

#[derive(Debug)]
pub struct SoundEffectPlugin {
    id: String,
    engine: SoundEngine,
    media: MediaStream,
}

// ------------------------------------------------------------------------------------------------
// Implementations
// ------------------------------------------------------------------------------------------------

impl Plugin for SoundEffectPlugin {
    fn plugin_id(&self) -> &String {
        &self.id
    }
    fn on_load(&self) -> dygpi::error::Result<()> {
        Ok(())
    }
    fn on_unload(&self) -> dygpi::error::Result<()> {
        Ok(())
    }
}

impl SoundEffectPlugin {
    pub fn new(id: &str) -> Self {
        Self {
            id: id.to_string(),
            engine: Default::default(),
            media: Default::default(),
        }
    }
    pub fn play(&self) {}
}
