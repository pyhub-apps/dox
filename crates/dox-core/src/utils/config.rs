use anyhow::Result;
use dirs;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

/// Application configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Config {
    #[serde(default)]
    pub global: GlobalConfig,

    #[serde(default)]
    pub replace: ReplaceConfig,

    #[serde(default)]
    pub generate: GenerateConfig,

    #[serde(default)]
    pub openai: OpenAIConfig,

    #[serde(default)]
    pub claude: ClaudeConfig,

    #[serde(flatten)]
    pub custom: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlobalConfig {
    pub verbose: bool,
    pub quiet: bool,
    pub lang: String,
    pub no_color: bool,
}

impl Default for GlobalConfig {
    fn default() -> Self {
        GlobalConfig {
            verbose: false,
            quiet: false,
            lang: "ko".to_string(), // 기본 언어를 한글로 설정
            no_color: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplaceConfig {
    pub backup: bool,
    pub recursive: bool,
    pub concurrent: bool,
    pub max_workers: usize,
}

impl Default for ReplaceConfig {
    fn default() -> Self {
        ReplaceConfig {
            backup: true,
            recursive: true,
            concurrent: true,
            max_workers: 4,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerateConfig {
    pub model: String,
    pub max_tokens: usize,
    pub temperature: f32,
    pub content_type: String,
}

impl Default for GenerateConfig {
    fn default() -> Self {
        GenerateConfig {
            model: "gpt-3.5-turbo".to_string(),
            max_tokens: 2000,
            temperature: 0.7,
            content_type: "blog".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct OpenAIConfig {
    pub api_key: Option<String>,
    pub model: Option<String>,
    pub max_tokens: Option<usize>,
    pub temperature: Option<f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ClaudeConfig {
    pub api_key: Option<String>,
    pub model: Option<String>,
    pub max_tokens: Option<usize>,
    pub temperature: Option<f32>,
}

impl Config {
    /// Get the default configuration path
    pub fn default_path() -> Result<PathBuf> {
        let config_dir = dirs::config_dir()
            .ok_or_else(|| anyhow::anyhow!("Could not determine config directory"))?;
        Ok(config_dir.join("dox").join("config.toml"))
    }

    /// Initialize a new configuration file
    pub fn init() -> Result<()> {
        let path = Self::default_path()?;

        // Create directory if it doesn't exist
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        // Create default config
        let config = Config::default();
        config.save_to(&path)?;

        Ok(())
    }

    /// Load configuration from file
    pub fn load() -> Result<Self> {
        let path = Self::default_path()?;
        Self::load_from(&path)
    }

    /// Load configuration from a specific path
    pub fn load_from(path: &Path) -> Result<Self> {
        if !path.exists() {
            return Ok(Config::default());
        }

        let content = fs::read_to_string(path)?;

        // Try different formats
        let config: Config = if path.extension().and_then(|s| s.to_str()) == Some("yaml")
            || path.extension().and_then(|s| s.to_str()) == Some("yml")
        {
            serde_yaml::from_str(&content)?
        } else if path.extension().and_then(|s| s.to_str()) == Some("json") {
            serde_json::from_str(&content)?
        } else {
            // Default to TOML
            toml::from_str(&content)?
        };

        // Validate the loaded config
        config.validate()?;
        Ok(config)
    }

    /// Save configuration to file
    pub fn save(&self) -> Result<()> {
        let path = Self::default_path()?;
        self.save_to(&path)
    }

    /// Save configuration to a specific path
    pub fn save_to(&self, path: &Path) -> Result<()> {
        // Create directory if it doesn't exist
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        // Serialize based on file extension
        let content = if path.extension().and_then(|s| s.to_str()) == Some("yaml")
            || path.extension().and_then(|s| s.to_str()) == Some("yml")
        {
            serde_yaml::to_string(self)?
        } else if path.extension().and_then(|s| s.to_str()) == Some("json") {
            serde_json::to_string_pretty(self)?
        } else {
            // Default to TOML
            toml::to_string_pretty(self)?
        };

        fs::write(path, content)?;
        Ok(())
    }

    /// Get a configuration value by key
    pub fn get(&self, key: &str) -> Option<String> {
        let parts: Vec<&str> = key.split('.').collect();

        match parts.as_slice() {
            ["global", "verbose"] => Some(self.global.verbose.to_string()),
            ["global", "quiet"] => Some(self.global.quiet.to_string()),
            ["global", "lang"] => Some(self.global.lang.clone()),
            ["global", "no_color"] => Some(self.global.no_color.to_string()),

            ["replace", "backup"] => Some(self.replace.backup.to_string()),
            ["replace", "recursive"] => Some(self.replace.recursive.to_string()),
            ["replace", "concurrent"] => Some(self.replace.concurrent.to_string()),
            ["replace", "max_workers"] => Some(self.replace.max_workers.to_string()),

            ["generate", "model"] => Some(self.generate.model.clone()),
            ["generate", "max_tokens"] => Some(self.generate.max_tokens.to_string()),
            ["generate", "temperature"] => Some(self.generate.temperature.to_string()),
            ["generate", "content_type"] => Some(self.generate.content_type.clone()),

            ["openai", "api_key"] => self.openai.api_key.clone(),
            ["openai", "model"] => self.openai.model.clone(),

            ["claude", "api_key"] => self.claude.api_key.clone(),
            ["claude", "model"] => self.claude.model.clone(),

            _ => {
                // Check custom values
                self.custom.get(key).map(|v| match v {
                    serde_json::Value::String(s) => s.clone(),
                    v => v.to_string(),
                })
            }
        }
    }

    /// Set a configuration value by key
    pub fn set(&mut self, key: &str, value: &str) -> Result<()> {
        // Validate the field before setting
        self.validate_field(key, value)?;

        let parts: Vec<&str> = key.split('.').collect();

        match parts.as_slice() {
            ["global", "verbose"] => self.global.verbose = value.parse()?,
            ["global", "quiet"] => self.global.quiet = value.parse()?,
            ["global", "lang"] => self.global.lang = value.to_string(),
            ["global", "no_color"] => self.global.no_color = value.parse()?,

            ["replace", "backup"] => self.replace.backup = value.parse()?,
            ["replace", "recursive"] => self.replace.recursive = value.parse()?,
            ["replace", "concurrent"] => self.replace.concurrent = value.parse()?,
            ["replace", "max_workers"] => self.replace.max_workers = value.parse()?,

            ["generate", "model"] => self.generate.model = value.to_string(),
            ["generate", "max_tokens"] => self.generate.max_tokens = value.parse()?,
            ["generate", "temperature"] => self.generate.temperature = value.parse()?,
            ["generate", "content_type"] => self.generate.content_type = value.to_string(),

            ["openai", "api_key"] => self.openai.api_key = Some(value.to_string()),
            ["openai", "model"] => self.openai.model = Some(value.to_string()),

            ["claude", "api_key"] => self.claude.api_key = Some(value.to_string()),
            ["claude", "model"] => self.claude.model = Some(value.to_string()),

            _ => {
                // Set as custom value
                self.custom.insert(
                    key.to_string(),
                    serde_json::Value::String(value.to_string()),
                );
            }
        }

        Ok(())
    }

    /// Remove a configuration value by key
    pub fn unset(&mut self, key: &str) -> Result<()> {
        let parts: Vec<&str> = key.split('.').collect();

        match parts.as_slice() {
            ["openai", "api_key"] => self.openai.api_key = None,
            ["openai", "model"] => self.openai.model = None,

            ["claude", "api_key"] => self.claude.api_key = None,
            ["claude", "model"] => self.claude.model = None,

            _ => {
                self.custom.remove(key);
            }
        }

        Ok(())
    }

    /// Validate the configuration
    pub fn validate(&self) -> Result<()> {
        self.validate_global()?;
        self.validate_replace()?;
        self.validate_generate()?;
        self.validate_openai()?;
        self.validate_claude()?;
        Ok(())
    }

    /// Validate a specific field
    pub fn validate_field(&self, key: &str, value: &str) -> Result<()> {
        let parts: Vec<&str> = key.split('.').collect();

        match parts.as_slice() {
            ["global", "verbose"] | ["global", "quiet"] | ["global", "no_color"] => {
                value.parse::<bool>().map_err(|_| {
                    anyhow::anyhow!("'{}' must be true or false, got '{}'", key, value)
                })?;
            }
            ["global", "lang"] => {
                if !["ko", "en"].contains(&value) {
                    return Err(anyhow::anyhow!(
                        "'{}' must be 'ko' or 'en', got '{}'",
                        key,
                        value
                    ));
                }
            }
            ["replace", "backup"] | ["replace", "recursive"] | ["replace", "concurrent"] => {
                value.parse::<bool>().map_err(|_| {
                    anyhow::anyhow!("'{}' must be true or false, got '{}'", key, value)
                })?;
            }
            ["replace", "max_workers"] => {
                let workers: u32 = value
                    .parse()
                    .map_err(|_| anyhow::anyhow!("'{}' must be a number, got '{}'", key, value))?;
                if !(1..=32).contains(&workers) {
                    return Err(anyhow::anyhow!(
                        "'{}' must be between 1 and 32, got {}",
                        key,
                        workers
                    ));
                }
            }
            ["generate", "max_tokens"] => {
                let tokens: u32 = value
                    .parse()
                    .map_err(|_| anyhow::anyhow!("'{}' must be a number, got '{}'", key, value))?;
                if !(1..=10000).contains(&tokens) {
                    return Err(anyhow::anyhow!(
                        "'{}' must be between 1 and 10000, got {}",
                        key,
                        tokens
                    ));
                }
            }
            ["generate", "temperature"] => {
                let temp: f32 = value
                    .parse()
                    .map_err(|_| anyhow::anyhow!("'{}' must be a number, got '{}'", key, value))?;
                if !(0.0..=2.0).contains(&temp) {
                    return Err(anyhow::anyhow!(
                        "'{}' must be between 0.0 and 2.0, got {}",
                        key,
                        temp
                    ));
                }
            }
            ["generate", "model"] => {
                let valid_models = ["gpt-3.5-turbo", "gpt-4", "gpt-4-turbo"];
                if !valid_models.contains(&value) {
                    return Err(anyhow::anyhow!(
                        "'{}' must be one of {:?}, got '{}'",
                        key,
                        valid_models,
                        value
                    ));
                }
            }
            ["generate", "content_type"] => {
                let valid_types = ["blog", "report", "summary", "email", "proposal"];
                if !valid_types.contains(&value) {
                    return Err(anyhow::anyhow!(
                        "'{}' must be one of {:?}, got '{}'",
                        key,
                        valid_types,
                        value
                    ));
                }
            }
            ["openai", "model"] | ["claude", "model"] => {
                if value.is_empty() {
                    return Err(anyhow::anyhow!("'{}' cannot be empty", key));
                }
            }
            _ => {
                // Custom values - basic validation
                if key.contains("..") || key.starts_with('.') || key.ends_with('.') {
                    return Err(anyhow::anyhow!("Invalid key format: '{}'", key));
                }
            }
        }

        Ok(())
    }

    fn validate_global(&self) -> Result<()> {
        if self.global.verbose && self.global.quiet {
            return Err(anyhow::anyhow!("verbose and quiet cannot both be true"));
        }
        if !["ko", "en"].contains(&self.global.lang.as_str()) {
            return Err(anyhow::anyhow!(
                "lang must be 'ko' or 'en', got '{}'",
                self.global.lang
            ));
        }
        Ok(())
    }

    fn validate_replace(&self) -> Result<()> {
        if !(1..=32).contains(&self.replace.max_workers) {
            return Err(anyhow::anyhow!(
                "max_workers must be between 1 and 32, got {}",
                self.replace.max_workers
            ));
        }
        Ok(())
    }

    fn validate_generate(&self) -> Result<()> {
        if !(1..=10000).contains(&self.generate.max_tokens) {
            return Err(anyhow::anyhow!(
                "max_tokens must be between 1 and 10000, got {}",
                self.generate.max_tokens
            ));
        }
        if !(0.0..=2.0).contains(&self.generate.temperature) {
            return Err(anyhow::anyhow!(
                "temperature must be between 0.0 and 2.0, got {}",
                self.generate.temperature
            ));
        }
        let valid_models = ["gpt-3.5-turbo", "gpt-4", "gpt-4-turbo"];
        if !valid_models.contains(&self.generate.model.as_str()) {
            return Err(anyhow::anyhow!(
                "model must be one of {:?}, got '{}'",
                valid_models,
                self.generate.model
            ));
        }
        let valid_types = ["blog", "report", "summary", "email", "proposal"];
        if !valid_types.contains(&self.generate.content_type.as_str()) {
            return Err(anyhow::anyhow!(
                "content_type must be one of {:?}, got '{}'",
                valid_types,
                self.generate.content_type
            ));
        }
        Ok(())
    }

    fn validate_openai(&self) -> Result<()> {
        if let Some(model) = &self.openai.model {
            if model.is_empty() {
                return Err(anyhow::anyhow!("openai.model cannot be empty"));
            }
        }
        Ok(())
    }

    fn validate_claude(&self) -> Result<()> {
        if let Some(model) = &self.claude.model {
            if model.is_empty() {
                return Err(anyhow::anyhow!("claude.model cannot be empty"));
            }
        }
        Ok(())
    }

    /// Display the configuration in a readable format
    pub fn display(&self) -> String {
        toml::to_string_pretty(self).unwrap_or_else(|_| "Failed to display config".to_string())
    }

    /// Display the configuration with colors and better formatting
    pub fn display_colored(&self) -> String {
        use colored::*;

        let mut output = String::new();

        // Global settings
        output.push_str(&format!("{}\n", "[global]".blue().bold()));
        output.push_str(&format!(
            "  {} = {}\n",
            "verbose".green(),
            format!("{}", self.global.verbose).yellow()
        ));
        output.push_str(&format!(
            "  {} = {}\n",
            "quiet".green(),
            format!("{}", self.global.quiet).yellow()
        ));
        output.push_str(&format!(
            "  {} = {}\n",
            "lang".green(),
            format!("\"{}\"", self.global.lang).yellow()
        ));
        output.push_str(&format!(
            "  {} = {}\n",
            "no_color".green(),
            format!("{}", self.global.no_color).yellow()
        ));
        output.push('\n');

        // Replace settings
        output.push_str(&format!("{}\n", "[replace]".blue().bold()));
        output.push_str(&format!(
            "  {} = {}\n",
            "backup".green(),
            format!("{}", self.replace.backup).yellow()
        ));
        output.push_str(&format!(
            "  {} = {}\n",
            "recursive".green(),
            format!("{}", self.replace.recursive).yellow()
        ));
        output.push_str(&format!(
            "  {} = {}\n",
            "concurrent".green(),
            format!("{}", self.replace.concurrent).yellow()
        ));
        output.push_str(&format!(
            "  {} = {}\n",
            "max_workers".green(),
            format!("{}", self.replace.max_workers).yellow()
        ));
        output.push('\n');

        // Generate settings
        output.push_str(&format!("{}\n", "[generate]".blue().bold()));
        output.push_str(&format!(
            "  {} = {}\n",
            "model".green(),
            format!("\"{}\"", self.generate.model).yellow()
        ));
        output.push_str(&format!(
            "  {} = {}\n",
            "max_tokens".green(),
            format!("{}", self.generate.max_tokens).yellow()
        ));
        output.push_str(&format!(
            "  {} = {}\n",
            "temperature".green(),
            format!("{}", self.generate.temperature).yellow()
        ));
        output.push_str(&format!(
            "  {} = {}\n",
            "content_type".green(),
            format!("\"{}\"", self.generate.content_type).yellow()
        ));
        output.push('\n');

        // OpenAI settings
        output.push_str(&format!("{}\n", "[openai]".blue().bold()));
        if let Some(ref api_key) = self.openai.api_key {
            let masked_key = if api_key.len() > 8 {
                format!("{}***", &api_key[..8])
            } else {
                "***".to_string()
            };
            output.push_str(&format!(
                "  {} = {}\n",
                "api_key".green(),
                format!("\"{}\"", masked_key).yellow()
            ));
        } else {
            output.push_str(&format!("  {} = {}\n", "api_key".green(), "null".red()));
        }

        if let Some(ref model) = self.openai.model {
            output.push_str(&format!(
                "  {} = {}\n",
                "model".green(),
                format!("\"{}\"", model).yellow()
            ));
        } else {
            output.push_str(&format!("  {} = {}\n", "model".green(), "null".red()));
        }

        if let Some(max_tokens) = self.openai.max_tokens {
            output.push_str(&format!(
                "  {} = {}\n",
                "max_tokens".green(),
                format!("{}", max_tokens).yellow()
            ));
        }

        if let Some(temperature) = self.openai.temperature {
            output.push_str(&format!(
                "  {} = {}\n",
                "temperature".green(),
                format!("{}", temperature).yellow()
            ));
        }
        output.push('\n');

        // Claude settings
        output.push_str(&format!("{}\n", "[claude]".blue().bold()));
        if let Some(ref api_key) = self.claude.api_key {
            let masked_key = if api_key.len() > 8 {
                format!("{}***", &api_key[..8])
            } else {
                "***".to_string()
            };
            output.push_str(&format!(
                "  {} = {}\n",
                "api_key".green(),
                format!("\"{}\"", masked_key).yellow()
            ));
        } else {
            output.push_str(&format!("  {} = {}\n", "api_key".green(), "null".red()));
        }

        if let Some(ref model) = self.claude.model {
            output.push_str(&format!(
                "  {} = {}\n",
                "model".green(),
                format!("\"{}\"", model).yellow()
            ));
        } else {
            output.push_str(&format!("  {} = {}\n", "model".green(), "null".red()));
        }

        if let Some(max_tokens) = self.claude.max_tokens {
            output.push_str(&format!(
                "  {} = {}\n",
                "max_tokens".green(),
                format!("{}", max_tokens).yellow()
            ));
        }

        if let Some(temperature) = self.claude.temperature {
            output.push_str(&format!(
                "  {} = {}\n",
                "temperature".green(),
                format!("{}", temperature).yellow()
            ));
        }

        // Custom settings
        if !self.custom.is_empty() {
            output.push('\n');
            output.push_str(&format!("{}\n", "[custom]".blue().bold()));
            for (key, value) in &self.custom {
                let value_str = match value {
                    serde_json::Value::String(s) => format!("\"{}\"", s),
                    v => v.to_string(),
                };
                output.push_str(&format!("  {} = {}\n", key.green(), value_str.yellow()));
            }
        }

        output
    }
}
