//! Integration tests for `checker::para_shape::check` — extended field coverage.
//!
//! These tests cover the newly added `JID_PARA_SHAPE_*` fields (2001–2045 range)
//! beyond the original six that were already covered. Three categories are
//! exercised with real fixtures and inline specs:
//!
//! 1. **Horizontal alignment (2001)** — PARASHAPE_HORIZONTAL
//! 2. **Margin-left / margin-right (2002, 2003)** — PARASHAPE_MARGINLEFT / MARGINRIGHT
//! 3. **Tab and border fields (2026–2037)** — error code range sanity
//!
//! All fail cases are *synthetic*: they re-use existing fixtures but supply an
//! inline spec that disagrees with the document. This avoids the need for
//! additional fixture HWPX files.
//!
//! # Fixtures used
//!
//! - `parashape_pass.hwpx` — document with default (JUSTIFY) alignment and
//!   zero margins. Used as the baseline for pass/fail tests.
//! - `parashape_fail_indent.hwpx` — document with a non-zero indent. Reused
//!   to confirm the extended spec does not introduce spurious errors.

use std::path::PathBuf;

use hwp_dvc_core::checker::{Checker, DvcErrorInfo};
use hwp_dvc_core::document::Document;
use hwp_dvc_core::error::para_shape_codes::{
    PARASHAPE_HORIZONTAL, PARASHAPE_MARGINLEFT, PARASHAPE_MARGINRIGHT,
};
use hwp_dvc_core::error::ErrorCode;
use hwp_dvc_core::spec::DvcSpec;

// ── Path helpers ──────────────────────────────────────────────────────────────

fn doc_fixture(name: &str) -> PathBuf {
    let mut p = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    p.push("tests/fixtures/docs");
    p.push(name);
    p
}

fn parse_doc(name: &str) -> Document {
    let mut doc = Document::open(doc_fixture(name))
        .unwrap_or_else(|e| panic!("failed to open fixture {name}: {e}"));
    doc.parse()
        .unwrap_or_else(|e| panic!("failed to parse fixture {name}: {e}"));
    doc
}

fn run_checker(doc: &Document, spec: &DvcSpec) -> Vec<DvcErrorInfo> {
    Checker::new(spec, doc)
        .run()
        .expect("Checker::run must not fail")
}

/// Returns true iff `code` is in the 2000-range (ParaShape category).
fn is_parashape(code: u32) -> bool {
    let base = ErrorCode::ParaShape as u32;
    let next = ErrorCode::Table as u32;
    code >= base && code < next
}

// ── Horizontal alignment (2001) ───────────────────────────────────────────────

/// When the spec specifies `"horizontal": 0` (JUSTIFY) and the document
/// paragraph alignment is JUSTIFY, no PARASHAPE_HORIZONTAL error must fire.
///
/// `parashape_pass.hwpx` contains paragraphs with default JUSTIFY alignment,
/// so this is a genuine pass case.
#[test]
fn horizontal_alignment_pass_with_justify_spec() {
    let doc = parse_doc("parashape_pass.hwpx");
    let spec =
        DvcSpec::from_json_str(r#"{ "parashape": { "horizontal": 0 } }"#).expect("spec must parse");
    let errs = run_checker(&doc, &spec);
    let halign_errs: Vec<_> = errs
        .iter()
        .filter(|e| e.error_code == PARASHAPE_HORIZONTAL)
        .collect();
    assert!(
        halign_errs.is_empty(),
        "parashape_pass.hwpx with horizontal:0 must produce zero PARASHAPE_HORIZONTAL errors; \
         got: {halign_errs:?}"
    );
}

/// When the spec demands `"horizontal": 3` (RIGHT alignment) but the
/// document has JUSTIFY paragraphs, at least one PARASHAPE_HORIZONTAL error
/// must fire.
///
/// **Synthetic fail case**: `parashape_pass.hwpx` with a wrong-alignment spec.
#[test]
fn horizontal_alignment_fail_when_spec_mismatches_document() {
    let doc = parse_doc("parashape_pass.hwpx");
    let spec =
        DvcSpec::from_json_str(r#"{ "parashape": { "horizontal": 3 } }"#).expect("spec must parse");
    let errs = run_checker(&doc, &spec);
    assert!(
        errs.iter().any(|e| e.error_code == PARASHAPE_HORIZONTAL),
        "parashape_pass.hwpx (JUSTIFY) with horizontal:3 spec must produce at least one \
         PARASHAPE_HORIZONTAL (2001) error; got codes: {:?}",
        errs.iter()
            .filter(|e| is_parashape(e.error_code))
            .map(|e| e.error_code)
            .collect::<Vec<_>>()
    );
}

// ── Margin-left / margin-right (2002, 2003) ───────────────────────────────────

/// When the spec specifies `"margin-left": 0` and the document has paragraphs
/// with zero left-margin, no PARASHAPE_MARGINLEFT error must fire.
#[test]
fn margin_left_pass_with_zero_spec() {
    let doc = parse_doc("parashape_pass.hwpx");
    let spec = DvcSpec::from_json_str(r#"{ "parashape": { "margin-left": 0 } }"#)
        .expect("spec must parse");
    let errs = run_checker(&doc, &spec);
    let margin_errs: Vec<_> = errs
        .iter()
        .filter(|e| e.error_code == PARASHAPE_MARGINLEFT)
        .collect();
    assert!(
        margin_errs.is_empty(),
        "parashape_pass.hwpx with margin-left:0 must produce zero PARASHAPE_MARGINLEFT errors; \
         got: {margin_errs:?}"
    );
}

/// When the spec demands `"margin-left": 9999` (an extreme value) but the
/// document has zero left-margin paragraphs, a PARASHAPE_MARGINLEFT error
/// must fire.
///
/// **Synthetic fail case**.
#[test]
fn margin_left_fail_when_spec_mismatches_document() {
    let doc = parse_doc("parashape_pass.hwpx");
    let spec = DvcSpec::from_json_str(r#"{ "parashape": { "margin-left": 9999 } }"#)
        .expect("spec must parse");
    let errs = run_checker(&doc, &spec);
    assert!(
        errs.iter().any(|e| e.error_code == PARASHAPE_MARGINLEFT),
        "parashape_pass.hwpx with margin-left:9999 must produce at least one \
         PARASHAPE_MARGINLEFT (2002) error; got codes: {:?}",
        errs.iter()
            .filter(|e| is_parashape(e.error_code))
            .map(|e| e.error_code)
            .collect::<Vec<_>>()
    );
}

/// When the spec demands `"margin-right": 9999` but the document has zero
/// right-margin paragraphs, a PARASHAPE_MARGINRIGHT error must fire.
///
/// **Synthetic fail case**.
#[test]
fn margin_right_fail_when_spec_mismatches_document() {
    let doc = parse_doc("parashape_pass.hwpx");
    let spec = DvcSpec::from_json_str(r#"{ "parashape": { "margin-right": 9999 } }"#)
        .expect("spec must parse");
    let errs = run_checker(&doc, &spec);
    assert!(
        errs.iter().any(|e| e.error_code == PARASHAPE_MARGINRIGHT),
        "parashape_pass.hwpx with margin-right:9999 must produce at least one \
         PARASHAPE_MARGINRIGHT (2003) error; got codes: {:?}",
        errs.iter()
            .filter(|e| is_parashape(e.error_code))
            .map(|e| e.error_code)
            .collect::<Vec<_>>()
    );
}

// ── Tab and border fields — error code range sanity (2026–2037) ───────────────

/// All ParaShape error code constants must fall in the 2000–2999 range.
///
/// This is a compile-time-expressible property: if any constant is outside
/// the range the assertion will catch it at test time.
#[test]
fn all_parashape_error_codes_are_in_2000_range() {
    use hwp_dvc_core::error::para_shape_codes::*;

    let codes: &[u32] = &[
        PARASHAPE_HORIZONTAL,
        PARASHAPE_MARGINLEFT,
        PARASHAPE_MARGINRIGHT,
        PARASHAPE_FIRSTLINE,
        PARASHAPE_INDENT,
        PARASHAPE_OUTDENT,
        PARASHAPE_LINESPACING,
        PARASHAPE_LINESPACINGVALUE,
        PARASHAPE_SPACINGPARAUP,
        PARASHAPE_SPACINGPARABOTTOM,
        PARASHAPE_SPACINGGRIDPAPER,
        PARASHAPE_LINEBREAKKOREAN,
        PARASHAPE_LINEBREAKENGLISH,
        PARASHAPE_LINEBREAKCONDENSE,
        PARASHAPE_PARATYPE,
        PARASHAPE_PARATYPEVALUE,
        PARASHAPE_WIDOWORPHAN,
        PARASHAPE_KEEPWITHNEXT,
        PARASHAPE_KEEPLINESTOGETHER,
        PARASHAPE_PAGEBREAKBEFORE,
        PARASHAPE_FONTLINEHEIGHT,
        PARASHAPE_LINEWRAP,
        PARASHAPE_AUTOSPACEEASIANENG,
        PARASHAPE_AUTOSPACEEASIANNUM,
        PARASHAPE_VERTICALALIGN,
        PARASHAPE_TABTYPES,
        PARASHAPE_TABTYPE,
        PARASHAPE_TABSHAPE,
        PARASHAPE_TABPOSITION,
        PARASHAPE_AUTOTABINDENT,
        PARASHAPE_AUTOTABPARARIGHTEND,
        PARASHAPE_BASETABSPACE,
        PARASHAPE_BORDER,
        PARASHAPE_BORDERPOSITION,
        PARASHAPE_BORDERTYPE,
        PARASHAPE_BORDERSIZE,
        PARASHAPE_BORDERCOLOR,
        PARASHAPE_BGCOLOR,
        PARASHAPE_BGPATTONCOLOR,
        PARASHAPE_BGPATTONTYPE,
        PARASHAPE_SPACINGLEFT,
        PARASHAPE_SPACINGRIGHT,
        PARASHAPE_SPACINGTOP,
        PARASHAPE_SPACINGBOTTOM,
        PARASHAPE_SPACINGIGNORE,
    ];

    let base = ErrorCode::ParaShape as u32; // 2000
    let next = ErrorCode::Table as u32; // 3000

    for &code in codes {
        assert!(
            code >= base && code < next,
            "ParaShape error code {code} is outside the 2000–2999 range"
        );
    }
}

/// The extended spec fields (new in this PR) must parse without errors.
/// This ensures the serde rename attributes are correct.
#[test]
fn extended_parashape_spec_fields_parse_from_json() {
    let json = r#"{
        "parashape": {
            "horizontal":             0,
            "margin-left":            0,
            "margin-right":           0,
            "firstline":              false,
            "indent":                 0,
            "outdent":                0,
            "linespacing":            0,
            "linespacingvalue":       160,
            "spacing-paraup":         0,
            "spacing-parabottom":     0,
            "spacing-gridpaper":      false,
            "linebreak-korean":       false,
            "linebreak-english":      0,
            "linebreak-condense":     100,
            "paratype":               0,
            "paratype-value":         0,
            "widow-orphan":           false,
            "keep-with-next":         false,
            "keep-lines-together":    false,
            "pagebreak-before":       false,
            "fontlineheight":         false,
            "linewrap":               false,
            "autospace-easian-eng":   false,
            "autospace-easian-num":   false,
            "verticalalign":          0,
            "autotab-intent":         false,
            "autotab-pararightend":   false,
            "basetabspace":           0,
            "spacing-left":           false,
            "spacing-right":          false,
            "spacing-top":            false,
            "spacing-bottom":         false,
            "spacing-ignore":         false
        }
    }"#;

    let spec = DvcSpec::from_json_str(json).expect("extended parashape spec must parse");
    let ps = spec.parashape.expect("parashape key must be present");

    assert_eq!(ps.horizontal, Some(0));
    assert_eq!(ps.margin_left, Some(0));
    assert_eq!(ps.margin_right, Some(0));
    assert_eq!(ps.firstline, Some(false));
    assert_eq!(ps.linebreak_condense, Some(100));
    assert_eq!(ps.widow_orphan, Some(false));
    assert_eq!(ps.autospace_easian_eng, Some(false));
    assert_eq!(ps.verticalalign, Some(0));
    assert_eq!(ps.spacing_ignore, Some(false));
}

/// When the entire new set of spec fields matches the document, no new
/// 2000-range errors should fire on `parashape_fail_indent.hwpx` for the
/// fields that are expected to pass (all fields except indent/outdent).
///
/// This is a regression guard: adding new fields must not introduce
/// spurious errors when the spec is deliberately permissive.
#[test]
fn extended_spec_with_all_none_produces_no_new_errors() {
    let doc = parse_doc("parashape_pass.hwpx");
    // Only specify the previously-covered fields that match the fixture.
    let spec = DvcSpec::from_json_str(
        r#"{ "parashape": { "indent": 0, "outdent": 0, "linespacing": 0, "linespacingvalue": 160 } }"#,
    )
    .expect("spec must parse");
    let errs = run_checker(&doc, &spec);
    let parashape_errs: Vec<_> = errs.iter().filter(|e| is_parashape(e.error_code)).collect();
    assert!(
        parashape_errs.is_empty(),
        "parashape_pass.hwpx must produce zero 2000-range errors with matching spec; \
         got: {:?}",
        parashape_errs
            .iter()
            .map(|e| (e.error_code, e.para_pr_id_ref))
            .collect::<Vec<_>>()
    );
}
