//! Utility functions for document processing

use crate::provider::{DocumentError, DocumentType};
use std::io::{Read, Seek, Write};
use std::path::Path;
use zip::write::SimpleFileOptions;
use zip::{ZipArchive, ZipWriter};

/// Extract a ZIP archive to a temporary directory
pub fn extract_zip(zip_data: &[u8]) -> Result<ZipArchive<std::io::Cursor<&[u8]>>, DocumentError> {
    let reader = std::io::Cursor::new(zip_data);
    let archive = ZipArchive::new(reader)?;
    Ok(archive)
}

/// Create a new ZIP archive with the given files
pub fn create_zip<W: Write + Seek>(
    writer: W,
    files: impl Iterator<Item = (String, Vec<u8>)>,
) -> Result<(), DocumentError> {
    let mut zip_writer = ZipWriter::new(writer);

    for (name, content) in files {
        zip_writer.start_file(name, SimpleFileOptions::default())?;
        zip_writer.write_all(&content)?;
    }

    zip_writer.finish()?;
    Ok(())
}

/// Check if a file is an Office document based on file extension
pub fn is_office_document(path: &Path) -> bool {
    path.extension()
        .and_then(|s| s.to_str())
        .and_then(DocumentType::from_extension)
        .is_some()
}

/// Read a file from a ZIP archive by name
pub fn read_zip_file(
    archive: &mut ZipArchive<std::io::Cursor<&[u8]>>,
    file_name: &str,
) -> Result<Vec<u8>, DocumentError> {
    let mut file = archive.by_name(file_name)?;
    let mut content = Vec::new();
    file.read_to_end(&mut content)?;
    Ok(content)
}

/// Copy all files from source ZIP to destination ZIP, optionally replacing some files
pub fn copy_zip_with_replacements<W: Write + Seek>(
    source_data: &[u8],
    destination: W,
    replacements: &std::collections::HashMap<String, Vec<u8>>,
) -> Result<(), DocumentError> {
    let reader = std::io::Cursor::new(source_data);
    let mut source_archive = ZipArchive::new(reader)?;
    let mut dest_writer = ZipWriter::new(destination);

    // Copy all files from source, replacing when necessary
    for i in 0..source_archive.len() {
        let mut file = source_archive.by_index(i)?;
        let name = file.name().to_string();

        dest_writer.start_file(&name, SimpleFileOptions::default())?;

        if let Some(replacement_content) = replacements.get(&name) {
            // Use replacement content
            dest_writer.write_all(replacement_content)?;
        } else {
            // Copy original content
            let mut buffer = Vec::new();
            file.read_to_end(&mut buffer)?;
            dest_writer.write_all(&buffer)?;
        }
    }

    dest_writer.finish()?;
    Ok(())
}

/// Find all text content in XML that can be replaced
pub fn find_replaceable_text(
    xml_content: &[u8],
    text_tags: &[&str],
) -> Result<Vec<String>, DocumentError> {
    use quick_xml::events::Event;
    use quick_xml::Reader;

    let mut reader = Reader::from_reader(std::io::Cursor::new(xml_content));
    let mut buf = Vec::new();
    let mut texts = Vec::new();
    let mut in_text = false;
    let mut current_tag = String::new();

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) => {
                let tag_name = std::str::from_utf8(e.name().as_ref())
                    .unwrap_or("")
                    .to_string();

                if text_tags.contains(&tag_name.as_str()) {
                    in_text = true;
                    current_tag = tag_name;
                }
            }
            Ok(Event::End(ref e)) => {
                let tag_name = std::str::from_utf8(e.name().as_ref())
                    .unwrap_or("")
                    .to_string();

                if tag_name == current_tag {
                    in_text = false;
                    current_tag.clear();
                }
            }
            Ok(Event::Text(ref e)) if in_text => {
                let text = e.unescape()?;
                if !text.trim().is_empty() {
                    texts.push(text.to_string());
                }
            }
            Ok(Event::Eof) => break,
            Ok(_) => {}
            Err(e) => return Err(DocumentError::Xml(e)),
        }
        buf.clear();
    }

    Ok(texts)
}

/// Replace text in XML content while preserving structure
pub fn replace_text_in_xml(
    xml_content: &[u8],
    text_tags: &[&str],
    old: &str,
    new: &str,
) -> Result<(Vec<u8>, usize), DocumentError> {
    use quick_xml::events::{BytesText, Event};
    use quick_xml::{Reader, Writer};

    let mut reader = Reader::from_reader(std::io::Cursor::new(xml_content));
    let mut output = Vec::new();
    let mut writer = Writer::new(std::io::Cursor::new(&mut output));
    let mut buf = Vec::new();
    let mut replacement_count = 0;
    let mut in_text = false;
    let mut current_tag = String::new();

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) => {
                let tag_name = std::str::from_utf8(e.name().as_ref())
                    .unwrap_or("")
                    .to_string();

                if text_tags.contains(&tag_name.as_str()) {
                    in_text = true;
                    current_tag = tag_name;
                }
                writer.write_event(Event::Start(e.clone()))?;
            }
            Ok(Event::End(ref e)) => {
                let tag_name = std::str::from_utf8(e.name().as_ref())
                    .unwrap_or("")
                    .to_string();

                if tag_name == current_tag {
                    in_text = false;
                    current_tag.clear();
                }
                writer.write_event(Event::End(e.clone()))?;
            }
            Ok(Event::Text(ref e)) if in_text => {
                let text = e.unescape()?;
                let replaced = text.replace(old, new);
                if text != replaced {
                    replacement_count += text.matches(old).count();
                }
                writer.write_event(Event::Text(BytesText::new(&replaced)))?;
            }
            Ok(Event::Eof) => break,
            Ok(e) => writer.write_event(e)?,
            Err(e) => return Err(DocumentError::Xml(e)),
        }
        buf.clear();
    }

    Ok((output, replacement_count))
}

/// Extract text content from XML
pub fn extract_text_from_xml(
    xml_content: &[u8],
    text_tags: &[&str],
) -> Result<String, DocumentError> {
    use quick_xml::events::Event;
    use quick_xml::Reader;

    let mut reader = Reader::from_reader(std::io::Cursor::new(xml_content));
    let mut buf = Vec::new();
    let mut text = String::new();
    let mut in_text = false;
    let mut current_tag = String::new();

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) => {
                let tag_name = std::str::from_utf8(e.name().as_ref())
                    .unwrap_or("")
                    .to_string();

                if text_tags.contains(&tag_name.as_str()) {
                    in_text = true;
                    current_tag = tag_name;
                }
            }
            Ok(Event::End(ref e)) => {
                let tag_name = std::str::from_utf8(e.name().as_ref())
                    .unwrap_or("")
                    .to_string();

                if tag_name == current_tag {
                    in_text = false;
                    current_tag.clear();
                }
            }
            Ok(Event::Text(ref e)) if in_text => {
                text.push_str(&e.unescape()?);
                text.push(' ');
            }
            Ok(Event::Eof) => break,
            Ok(_) => {}
            Err(e) => return Err(DocumentError::Xml(e)),
        }
        buf.clear();
    }

    Ok(text.trim().to_string())
}
