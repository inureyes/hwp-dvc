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
//! - [`run_type`] — `Vec<RunTypeInfo>` builder (issue #4).

pub mod header;
pub mod run_type;
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

    /// Return `true` when `Contents/content.hpf` contains an OPF manifest
    /// item whose `href` attribute includes `.js`.
    ///
    /// This replicates `OWPMLReader::haveMacroInDocument` from the C++
    /// reference. The function does not require `quick-xml` — it scans the
    /// raw bytes with a lightweight string search, which is sufficient
    /// because `href` values never use XML character references.
    pub fn has_macro(&self) -> bool {
        let part = match self.part("Contents/content.hpf") {
            Some(p) => p,
            None => return false,
        };
        // The OPF manifest looks like:
        //   <opf:item id="script" href="Scripts/JScript.js" .../>
        // We scan for every occurrence of `href="` and check whether the
        // value before the closing `"` contains `.js`.
        let text = match std::str::from_utf8(&part.bytes) {
            Ok(s) => s,
            Err(_) => return false,
        };
        let mut search = text;
        while let Some(pos) = search.find("href=\"") {
            search = &search[pos + 6..]; // skip past `href="`
            if let Some(end) = search.find('"') {
                let href = &search[..end];
                if href.contains(".js") {
                    return true;
                }
                search = &search[end + 1..];
            } else {
                break;
            }
        }
        false
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

/// The result of parsing an HWPX document end-to-end.
///
/// After [`Document::open`] the struct only holds the raw
/// [`HwpxArchive`]; calling [`Document::parse`] fills in
/// [`Self::header`], [`Self::sections`], and [`Self::run_type_infos`]
/// in one pass. Validators never interact with the archive directly —
/// they read from the three populated fields.
///
/// All three populated fields are `Option`-less because a successful
/// parse guarantees each has a value; the initial "unparsed" state is
/// reflected by an empty `sections` vector and `header == None`
/// rather than any sentinel. `header` stays `Option<HeaderTables>` so
/// that callers that deliberately skip parsing (e.g., listing parts
/// for debugging) can distinguish "we haven't parsed yet" from "the
/// parse produced an empty table" — both are legal states.
#[derive(Debug, Default)]
pub struct Document {
    pub archive: HwpxArchive,
    /// Parsed header tables (`Contents/header.xml`), or `None` before
    /// [`Document::parse`] is called. Validators can assume this is
    /// `Some` once `parse` has returned `Ok`.
    pub header: Option<HeaderTables>,
    /// Parsed body sections (`Contents/section*.xml`), in ascending
    /// `N` order. Empty before [`Document::parse`] is called.
    pub sections: Vec<Section>,
    /// The flattened `RunTypeInfo` stream, in document order, across
    /// all sections. This is the unit of validation every Phase 2
    /// validator consumes.
    pub run_type_infos: Vec<RunTypeInfo>,
}

/// Mirrors `RunTypeInfo` in `references/dvc/Source/OWPMLReader.h`.
///
/// # Out-of-scope fields
///
/// `page_no` / `line_no` stay `0` in this issue. Layout-engine-based
/// page and line numbering is tracked separately as issue
/// [#19](https://github.com/inureyes/hwp-dvc/issues/19) because it
/// requires porting the reference's vertical-position walk through
/// `<hp:linesegarray>` — work that is deferred behind Phases 2/3
/// since no validator currently consumes pagination.
#[derive(Debug, Default, Clone)]
pub struct RunTypeInfo {
    pub char_pr_id_ref: u32,
    pub para_pr_id_ref: u32,
    pub text: String,
    /// Always `0` in this crate version — see [`RunTypeInfo`] doc and
    /// [`crate::document::run_type::PAGE_LINE_OUT_OF_SCOPE`].
    pub page_no: u32,
    /// Always `0` in this crate version — see [`RunTypeInfo`] doc and
    /// [`crate::document::run_type::PAGE_LINE_OUT_OF_SCOPE`].
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
    /// Korean name of the style applied to the paragraph containing this run.
    /// Resolved from `paragraph.style_id_ref` via the header style table.
    /// Empty string when the paragraph has no style or the style id is not
    /// found in the header (the `is_style` flag still reflects whether the
    /// style differs from 바탕글).
    pub style_name: String,
}

impl Document {
    /// Return `true` when `Contents/content.hpf` lists at least one
    /// `<opf:item>` whose `href` attribute contains `.js`.
    ///
    /// Mirrors `OWPMLReader::haveMacroInDocument` from the reference C++
    /// source: it scans the OPF manifest for JavaScript items, which
    /// HWP/HWPX uses to embed macro scripts.
    ///
    /// Returns `false` when the archive carries no `Contents/content.hpf`
    /// part (the manifest is optional in older HWPX variants) or when the
    /// manifest cannot be parsed.
    pub fn has_macro(&self) -> bool {
        self.archive.has_macro()
    }

    pub fn open(path: impl AsRef<Path>) -> DvcResult<Self> {
        let archive = HwpxArchive::open(path)?;
        Ok(Self {
            archive,
            header: None,
            sections: Vec::new(),
            run_type_infos: Vec::new(),
        })
    }

    /// Parse the OWPML header + body into [`Self::header`],
    /// [`Self::sections`] and [`Self::run_type_infos`].
    ///
    /// Idempotent: calling it a second time re-parses from the
    /// archive bytes and replaces the previous state. `DvcError` is
    /// returned if any sub-parse (header, section) fails; the
    /// document state is left unchanged in that case.
    pub fn parse(&mut self) -> DvcResult<()> {
        let header = self.archive.read_header()?;
        let sections = self.archive.read_sections()?;
        let run_type_infos = run_type::build_run_type_infos(&header, &sections);

        // Commit only after all sub-parses succeed.
        self.header = Some(header);
        self.sections = sections;
        self.run_type_infos = run_type_infos;
        Ok(())
    }
}
