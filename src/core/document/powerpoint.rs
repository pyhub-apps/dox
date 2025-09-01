use super::DocumentOps;
use anyhow::Result;
use quick_xml::events::{BytesText, Event};
use quick_xml::{Reader, Writer};
use std::fs::File;
use std::io::{Cursor, Read, Write};
use std::path::{Path, PathBuf};
use zip::write::{FileOptions, SimpleFileOptions};
use zip::{ZipArchive, ZipWriter};

/// Represents a PowerPoint document (.pptx)
pub struct PowerPointDocument {
    path: PathBuf,
    archive_data: Vec<u8>,  // Store the original file data
    slide_contents: Vec<(String, Vec<u8>)>,
    modified: bool,
}

impl PowerPointDocument {
    /// Open a PowerPoint document from a file path
    pub fn open(path: &Path) -> Result<Self> {
        // Read entire file into memory
        let archive_data = std::fs::read(path)?;
        
        // Parse the archive to get slide files
        let reader = Cursor::new(&archive_data);
        let mut archive = ZipArchive::new(reader)?;
        
        // Read all slide XML files
        let mut slide_contents = Vec::new();
        let mut slide_names = Vec::new();
        
        // First, collect slide file names
        for i in 0..archive.len() {
            let file = archive.by_index(i)?;
            let name = file.name().to_string();
            
            // Check if this is a slide file
            if name.starts_with("ppt/slides/slide") && name.ends_with(".xml") {
                slide_names.push(name);
            }
        }
        
        // Then read the content of each slide
        for name in slide_names {
            let mut content = Vec::new();
            let mut file = archive.by_name(&name)?;
            file.read_to_end(&mut content)?;
            slide_contents.push((name, content));
        }
        
        Ok(PowerPointDocument {
            path: path.to_path_buf(),
            archive_data,
            slide_contents,
            modified: false,
        })
    }
    
    /// Create a new PowerPoint document
    pub fn create(path: &Path) -> Result<Self> {
        // TODO: Implement creating a new PowerPoint document from scratch
        anyhow::bail!("Creating new PowerPoint documents is not yet implemented")
    }
}

impl DocumentOps for PowerPointDocument {
    fn replace_text(&mut self, old: &str, new: &str) -> Result<usize> {
        let mut total_count = 0;
        
        for (name, content) in &mut self.slide_contents {
            let mut output = Vec::new();
            let mut count = 0;
            
            // Parse and modify the XML
            let mut reader = Reader::from_reader(Cursor::new(&content));
            let mut writer = Writer::new(Cursor::new(&mut output));
            
            let mut buf = Vec::new();
            let mut in_text = false;
            
            loop {
                match reader.read_event_into(&mut buf) {
                    Ok(Event::Start(ref e)) if e.name().as_ref() == b"a:t" => {
                        in_text = true;
                        writer.write_event(Event::Start(e.clone()))?;
                    }
                    Ok(Event::End(ref e)) if e.name().as_ref() == b"a:t" => {
                        in_text = false;
                        writer.write_event(Event::End(e.clone()))?;
                    }
                    Ok(Event::Text(ref e)) if in_text => {
                        let text = e.unescape()?;
                        let replaced = text.replace(old, new);
                        if text != replaced {
                            count += text.matches(old).count();
                        }
                        writer.write_event(Event::Text(BytesText::new(&replaced)))?;
                    }
                    Ok(Event::Eof) => break,
                    Ok(e) => writer.write_event(e)?,
                    Err(e) => anyhow::bail!("Error parsing XML in {}: {}", name, e),
                }
                buf.clear();
            }
            
            if count > 0 {
                *content = output;
                total_count += count;
            }
        }
        
        if total_count > 0 {
            self.modified = true;
        }
        
        Ok(total_count)
    }
    
    fn save(&self) -> Result<()> {
        if !self.modified {
            return Ok(());
        }
        self.save_as(&self.path)
    }
    
    fn save_as(&self, path: &Path) -> Result<()> {
        let file = File::create(path)?;
        let mut writer = ZipWriter::new(file);
        
        // Re-open the archive from the stored data
        let reader = Cursor::new(&self.archive_data);
        let mut archive = ZipArchive::new(reader)?;
        
        // Copy all files from the original archive
        for i in 0..archive.len() {
            let mut file = archive.by_index(i)?;
            let name = file.name().to_string();
            
            // Check if this is a slide file we've modified
            let modified_content = self.slide_contents
                .iter()
                .find(|(n, _)| n == &name)
                .map(|(_, content)| content);
            
            if let Some(content) = modified_content {
                // Write our modified content
                writer.start_file(name, SimpleFileOptions::default())?;
                writer.write_all(content)?;
            } else {
                // Copy the original file
                writer.start_file(name, SimpleFileOptions::default())?;
                let mut buffer = Vec::new();
                file.read_to_end(&mut buffer)?;
                writer.write_all(&buffer)?;
            }
        }
        
        writer.finish()?;
        Ok(())
    }
    
    fn get_text(&self) -> Result<String> {
        let mut all_text = String::new();
        
        for (_, content) in &self.slide_contents {
            let mut reader = Reader::from_reader(Cursor::new(content));
            let mut buf = Vec::new();
            let mut in_text = false;
            
            loop {
                match reader.read_event_into(&mut buf) {
                    Ok(Event::Start(ref e)) if e.name().as_ref() == b"a:t" => {
                        in_text = true;
                    }
                    Ok(Event::End(ref e)) if e.name().as_ref() == b"a:t" => {
                        in_text = false;
                    }
                    Ok(Event::Text(ref e)) if in_text => {
                        all_text.push_str(&e.unescape()?);
                        all_text.push(' ');
                    }
                    Ok(Event::Eof) => break,
                    Ok(_) => {}
                    Err(e) => anyhow::bail!("Error parsing XML: {}", e),
                }
                buf.clear();
            }
            
            all_text.push('\n');
        }
        
        Ok(all_text.trim().to_string())
    }
}