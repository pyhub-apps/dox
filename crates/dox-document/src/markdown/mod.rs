// TODO: Implement markdown parsing and conversion
pub struct MarkdownParser;

impl Default for MarkdownParser {
    fn default() -> Self {
        Self::new()
    }
}

impl MarkdownParser {
    pub fn new() -> Self {
        MarkdownParser
    }

    pub fn parse(&self, _content: &str) -> anyhow::Result<()> {
        anyhow::bail!("Markdown parsing not yet implemented")
    }
}
