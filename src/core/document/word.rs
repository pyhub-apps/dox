use super::DocumentOps;
use anyhow::Result;
use quick_xml::events::{BytesEnd, BytesStart, BytesText, Event};
use quick_xml::{Reader, Writer};
use std::fs::File;
use std::io::{Cursor, Read, Write};
use std::path::{Path, PathBuf};
use zip::write::{FileOptions, SimpleFileOptions};
use zip::{ZipArchive, ZipWriter};

/// Represents a Word document (.docx)
pub struct WordDocument {
    path: PathBuf,
    archive_data: Vec<u8>,  // Store the original file data
    content: Vec<u8>,
    modified: bool,
}

impl WordDocument {
    /// Open a Word document from a file path
    pub fn open(path: &Path) -> Result<Self> {
        // Read entire file into memory
        let archive_data = std::fs::read(path)?;
        
        // Parse the archive to get document.xml
        let reader = Cursor::new(&archive_data);
        let mut archive = ZipArchive::new(reader)?;
        
        // Read document.xml content
        let mut content = Vec::new();
        {
            let mut doc_file = archive.by_name("word/document.xml")?;
            doc_file.read_to_end(&mut content)?;
        }
        
        Ok(WordDocument {
            path: path.to_path_buf(),
            archive_data,
            content,
            modified: false,
        })
    }
    
    /// Create a new Word document
    pub fn create(path: &Path) -> Result<Self> {
        // TODO: Implement creating a new Word document from scratch
        anyhow::bail!("Creating new Word documents is not yet implemented")
    }
}

impl DocumentOps for WordDocument {
    fn replace_text(&mut self, old: &str, new: &str) -> Result<usize> {
        let mut count = 0;
        let mut output = Vec::new();
        
        // Parse and modify the XML
        let mut reader = Reader::from_reader(Cursor::new(&self.content));
        let mut writer = Writer::new(Cursor::new(&mut output));
        
        let mut buf = Vec::new();
        let mut in_text = false;
        
        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(ref e)) if e.name().as_ref() == b"w:t" => {
                    in_text = true;
                    writer.write_event(Event::Start(e.clone()))?;
                }
                Ok(Event::End(ref e)) if e.name().as_ref() == b"w:t" => {
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
                Err(e) => anyhow::bail!("Error parsing XML: {}", e),
            }
            buf.clear();
        }
        
        if count > 0 {
            self.content = output;
            self.modified = true;
        }
        
        Ok(count)
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
            
            if name == "word/document.xml" {
                // Write our modified content
                writer.start_file(name, SimpleFileOptions::default())?;
                writer.write_all(&self.content)?;
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
        let mut text = String::new();
        let mut reader = Reader::from_reader(Cursor::new(&self.content));
        let mut buf = Vec::new();
        let mut in_text = false;
        
        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(ref e)) if e.name().as_ref() == b"w:t" => {
                    in_text = true;
                }
                Ok(Event::End(ref e)) if e.name().as_ref() == b"w:t" => {
                    in_text = false;
                }
                Ok(Event::Text(ref e)) if in_text => {
                    text.push_str(&e.unescape()?);
                    text.push(' ');
                }
                Ok(Event::Eof) => break,
                Ok(_) => {}
                Err(e) => anyhow::bail!("Error parsing XML: {}", e),
            }
            buf.clear();
        }
        
        Ok(text.trim().to_string())
    }
}