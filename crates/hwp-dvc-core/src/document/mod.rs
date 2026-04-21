//! HWPX document reader (OWPML).
//!
//! An HWPX file is a ZIP container holding OWPML-formatted XML parts:
//!
//! ```text
//! *.hwpx
//! ├── META-INF/container.xml      (part manifest)
//! ├── Contents/
//! │   ├── header.xml               (charshapes, parashapes, borderfills, bullets, styles, ...)
//! │   ├── section0.xml             (document body: paragraphs, runs, tables, objects)
//! │   └── section1.xml, ...
//! └── ...
//! ```
//!
//! The reference C++ implementation delegates this to Hancom's OWPML
//! model DLL. In Rust we parse the XML directly with `quick-xml`.
//!
//! Submodules:
//! - [`header`] — `Contents/header.xml` shape tables (issue #2).
//! - [`section`] — `Contents/section*.xml` paragraph AST (issue #3).

pub mod header;
pub mod section;

use std::io::Read;
use std::path::Path;

use crate::error::{DvcError, DvcResult};

pub use header::HeaderTables;
pub use section::Section;

/// A minimal HWPX archive handle.
///
/// Owns the unzipped byte contents of each part. HWPX files are
/// typically small (hundreds of kilobytes), so loading everything into
/// memory is acceptable — streaming can be added later if needed.
#[derive(Debug, Default)]
pub struct HwpxArchive {
    parts: Vec<Part>,
}

#[derive(Debug)]
pub struct Part {
    pub name: String,
    pub bytes: Vec<u8>,
}

impl HwpxArchive {
    pub fn open(path: impl AsRef<Path>) -> DvcResult<Self> {
        let file = std::fs::File::open(path)?;
        let mut zip = zip::ZipArchive::new(file)?;
        let mut parts = Vec::with_capacity(zip.len());
        for i in 0..zip.len() {
            let mut entry = zip.by_index(i)?;
            if entry.is_dir() {
                continue;
            }
            let name = entry.name().to_owned();
            let mut bytes = Vec::with_capacity(entry.size() as usize);
            entry.read_to_end(&mut bytes)?;
            parts.push(Part { name, bytes });
        }
        Ok(Self { parts })
    }

    pub fn part_names(&self) -> impl Iterator<Item = &str> {
        self.parts.iter().map(|p| p.name.as_str())
    }

    pub fn part(&self, name: &str) -> Option<&Part> {
        self.parts.iter().find(|p| p.name == name)
    }

    /// Parse `Contents/header.xml` from this archive.
    ///
    /// Returns [`DvcError::Document`] if the archive has no
    /// `Contents/header.xml` part, or [`DvcError::Xml`] if the part's
    /// bytes fail to parse.
    pub fn read_header(&self) -> DvcResult<HeaderTables> {
        let part = self
            .part("Contents/header.xml")
            .ok_or_else(|| DvcError::Document("missing Contents/header.xml".into()))?;
        header::parser::parse_header(&part.bytes)
    }

    /// Parse every `Contents/sectionN.xml` part in ascending numeric
    /// order and return one [`Section`] per part.
    ///
    /// Non-numeric suffixes (`Contents/sectionBad.xml`) are silently
    /// skipped because no conforming HWPX writer produces them; a
    /// present-but-unnumbered section would be an authoring error
    /// unrelated to this parser.
    ///
    /// Returns an empty vector if the archive declares no section
    /// parts; that is a documented HWPX edge case (a cover-only
    /// archive) rather than an error.
    pub fn read_sections(&self) -> DvcResult<Vec<Section>> {
        // Collect (index, &Part) pairs, filter to `Contents/sectionN.xml`,
        // sort by N, then parse in order.
        let mut numbered: Vec<(u32, &Part)> = Vec::new();
        for part in &self.parts {
            if let Some(idx) = section_index(&part.name) {
                numbered.push((idx, part));
            }
        }
        numbered.sort_by_key(|(idx, _)| *idx);

        let mut sections = Vec::with_capacity(numbered.len());
        for (idx, part) in numbered {
            let sec = section::parser::parse_section(idx, &part.bytes).map_err(|e| match e {
                DvcError::Document(msg) => DvcError::Document(format!("{} in {}", msg, part.name)),
                other => other,
            })?;
            sections.push(sec);
        }
        Ok(sections)
    }
}

/// Extract the numeric suffix `N` from `Contents/sectionN.xml`.
/// Returns `None` if the part name does not match that pattern.
fn section_index(name: &str) -> Option<u32> {
    let rest = name.strip_prefix("Contents/section")?;
    let num = rest.strip_suffix(".xml")?;
    if num.is_empty() {
        return None;
    }
    num.parse::<u32>().ok()
}

/// Placeholder result of parsing the OWPML document — to be fleshed
/// out as validators start needing concrete shape data.
#[derive(Debug, Default)]
pub struct Document {
    pub archive: HwpxArchive,
    pub run_type_infos: Vec<RunTypeInfo>,
}

/// Mirrors `RunTypeInfo` in `references/dvc/Source/OWPMLReader.h`.
#[derive(Debug, Default, Clone)]
pub struct RunTypeInfo {
    pub char_pr_id_ref: u32,
    pub para_pr_id_ref: u32,
    pub text: String,
    pub page_no: u32,
    pub line_no: u32,
    pub is_in_table: bool,
    pub is_in_table_in_table: bool,
    pub is_in_shape: bool,
    pub table_id: u32,
    pub table_row: u32,
    pub table_col: u32,
    pub outline_shape_id_ref: u32,
    pub is_hyperlink: bool,
    pub is_style: bool,
}

impl Document {
    pub fn open(path: impl AsRef<Path>) -> DvcResult<Self> {
        let archive = HwpxArchive::open(path)?;
        Ok(Self {
            archive,
            run_type_infos: Vec::new(),
        })
    }

    /// Parse the OWPML body into `RunTypeInfo` entries.
    /// TODO: port from `OWPMLReader::GetRunTypeInfos`.
    pub fn parse(&mut self) -> DvcResult<()> {
        Err(DvcError::NotImplemented("Document::parse (OWPML reader)"))
    }
}
