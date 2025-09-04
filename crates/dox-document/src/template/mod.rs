// TODO: Implement template processing
pub struct TemplateProcessor;

impl Default for TemplateProcessor {
    fn default() -> Self {
        Self::new()
    }
}

impl TemplateProcessor {
    pub fn new() -> Self {
        TemplateProcessor
    }

    pub fn process(&self, _template: &str, _values: &serde_json::Value) -> anyhow::Result<String> {
        anyhow::bail!("Template processing not yet implemented")
    }
}
