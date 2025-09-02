// TODO: Implement template processing
pub struct TemplateProcessor;

impl TemplateProcessor {
    pub fn new() -> Self {
        TemplateProcessor
    }
    
    pub fn process(&self, _template: &str, _values: &serde_json::Value) -> anyhow::Result<String> {
        anyhow::bail!("Template processing not yet implemented")
    }
}