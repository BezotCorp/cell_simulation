#[derive(Default)]
pub(crate) struct RunStatus {
    initialized: bool,
    error: Option<String>,
}

impl RunStatus {
    pub(crate) fn new() -> Self {
        Self::default()
    }

    pub(crate) fn initialized(&self) -> bool {
        self.initialized
    }

    pub(crate) fn error(&self) -> Option<&str> {
        self.error.as_deref()
    }

    pub(crate) fn mark_initialized(&mut self) {
        self.initialized = true;
    }

    pub(crate) fn record_error(&mut self, error: String) {
        self.error = Some(error);
    }
}
