//! Output formatters for validation results.
//!
//! Maps to `IDVCOutput` / `DVCOutputJson` in the reference. Only JSON
//! is implemented initially; XML and plain text can be added behind
//! the same [`Format`] enum.

use serde::Serialize;

use crate::checker::DvcErrorInfo;
use crate::error::DvcResult;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum Format {
    #[default]
    Json,
    // Xml,   // TODO
    // Text,  // TODO
}

#[derive(Debug, Serialize)]
struct JsonRecord<'a> {
    #[serde(rename = "charIDRef")]
    char_id_ref: u32,
    #[serde(rename = "paraPrIDRef")]
    para_pr_id_ref: u32,
    text: &'a str,
    #[serde(rename = "pageNo")]
    page_no: u32,
    #[serde(rename = "lineNo")]
    line_no: u32,
    #[serde(rename = "errorCode")]
    error_code: u32,
    #[serde(rename = "tableID")]
    table_id: u32,
    #[serde(rename = "isInTable")]
    is_in_table: bool,
    #[serde(rename = "isInTableInTable")]
    is_in_table_in_table: bool,
    #[serde(rename = "tableRow")]
    table_row: u32,
    #[serde(rename = "tableCol")]
    table_col: u32,
}

impl<'a> From<&'a DvcErrorInfo> for JsonRecord<'a> {
    fn from(e: &'a DvcErrorInfo) -> Self {
        JsonRecord {
            char_id_ref: e.char_pr_id_ref,
            para_pr_id_ref: e.para_pr_id_ref,
            text: &e.text,
            page_no: e.page_no,
            line_no: e.line_no,
            error_code: e.error_code,
            table_id: e.table_id,
            is_in_table: e.is_in_table,
            is_in_table_in_table: e.is_in_table_in_table,
            table_row: e.table_row,
            table_col: e.table_col,
        }
    }
}

pub fn to_json(errors: &[DvcErrorInfo], pretty: bool) -> DvcResult<String> {
    let records: Vec<JsonRecord<'_>> = errors.iter().map(JsonRecord::from).collect();
    let s = if pretty {
        serde_json::to_string_pretty(&records)?
    } else {
        serde_json::to_string(&records)?
    };
    Ok(s)
}
