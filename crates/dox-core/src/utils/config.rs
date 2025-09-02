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
            lang: "ko".to_string(),  // 기본 언어를 한글로 설정
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
        if path.extension().and_then(|s| s.to_str()) == Some("yaml") 
            || path.extension().and_then(|s| s.to_str()) == Some("yml") {
            Ok(serde_yaml::from_str(&content)?)
        } else if path.extension().and_then(|s| s.to_str()) == Some("json") {
            Ok(serde_json::from_str(&content)?)
        } else {
            // Default to TOML
            Ok(toml::from_str(&content)?)
        }
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
            || path.extension().and_then(|s| s.to_str()) == Some("yml") {
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
                self.custom.get(key).and_then(|v| {
                    match v {
                        serde_json::Value::String(s) => Some(s.clone()),
                        v => Some(v.to_string()),
                    }
                })
            }
        }
    }
    
    /// Set a configuration value by key
    pub fn set(&mut self, key: &str, value: &str) -> Result<()> {
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
                self.custom.insert(key.to_string(), serde_json::Value::String(value.to_string()));
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
    
    /// Display the configuration in a readable format
    pub fn display(&self) -> String {
        toml::to_string_pretty(self).unwrap_or_else(|_| "Failed to display config".to_string())
    }
}