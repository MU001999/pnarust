use crate::{Result, KvsEngine};
use slog::Logger;

pub struct KvsServer<'sv> {
    logger: &'sv Logger,
    engine: &'sv mut dyn KvsEngine,
}

impl<'sv> KvsServer<'sv> {
    pub fn new(logger: &'sv Logger, engine: &'sv mut dyn KvsEngine) -> Self {
        KvsServer {
            logger,
            engine
        }
    }

    pub fn run() -> Result<()> {
        Ok(())
    }
}
