// Excel import engine.
//
// Reads a local Excel workbook (.xlsx / .xls), analyzes its structure (sheets,
// header rows, grouped/merged headers, column names), detects whether it
// contains Incoming or Outgoing records, maps the columns onto the existing
// application fields, and imports the data into the existing tables without
// modifying the database schema.
//
// All processing happens locally on the user's machine; the workbook content is
// never sent to any external service.

use calamine::{Data, Range, Reader};
use chrono::{TimeZone, Utc};
use serde::{Deserialize, Serialize};
use std::io::{BufReader, Write};
use std::path::Path;
use zip::write::SimpleFileOptions;

use crate::AppState;
use uuid::Uuid;

// ---------------------------------------------------------------------------
// Public DTOs shared with the frontend
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExcelKind {
    Incoming,
    Outgoing,
}

impl ExcelKind {
    fn as_str(&self) -> &'static str {
        match self {
            ExcelKind::Incoming => "incoming",
            ExcelKind::Outgoing => "outgoing",
        }
    }

    fn from_str(s: &str) -> ExcelKind {
        if s.eq_ignore_ascii_case("outgoing") || s.eq_ignore_ascii_case("sortie") {
            ExcelKind::Outgoing
        } else {
            ExcelKind::Incoming
        }
    }
}

/// A single mapped Excel column.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExcelColumn {
    /// Human friendly header shown to the user (group / field).
    pub header: String,
    /// Group name the column belongs to, if any (`outgoing`, `incoming`, `result`).
    pub group: Option<String>,
    /// Canonical application column this Excel column maps to. `None` = unmapped.
    pub field: Option<String>,
}

/// A single parsed data row from the workbook.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExcelDataRow {
    /// 1-based row number in the original worksheet.
    pub source_row: u32,
    /// One value per `columns` entry, in the same order.
    pub values: Vec<Option<String>>,
    /// Whether the sequential number carried an explicit "مكرر" marker
    /// (only meaningful for incoming).
    pub is_duplicate: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExcelPreviewRow {
    pub source_row: u32,
    pub cells: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RowIssue {
    pub source_row: u32,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExcelAnalysis {
    pub file_name: String,
    pub sheet_name: String,
    pub kind: String,
    pub kind_confident: bool,
    pub header_rows: usize,
    pub columns: Vec<ExcelColumn>,
    /// All data rows (one per non-blank data row in the worksheet), in order.
    pub rows: Vec<ExcelDataRow>,
    /// A small window of the same rows, used for the on-screen preview.
    pub preview: Vec<ExcelPreviewRow>,
    pub total_rows: usize,
    pub valid_rows: usize,
    pub invalid_rows: usize,
    pub duplicate_rows: usize,
    pub sample_issues: Vec<RowIssue>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExcelImportRequest {
    pub file_name: String,
    pub kind: String,
    pub columns: Vec<ExcelColumn>,
    pub rows: Vec<ExcelDataRow>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RowFailure {
    pub source_row: u32,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExcelImportResult {
    pub total: usize,
    pub imported: usize,
    pub skipped: usize,
    pub duplicates: usize,
    pub errors: usize,
    pub failures: Vec<RowFailure>,
}

// ---------------------------------------------------------------------------
// Keyword normalisation & detection
// ---------------------------------------------------------------------------

fn norm(s: &str) -> String {
    let mut out = String::new();
    for c in s.chars() {
        // Strip Arabic diacritics and tatweel.
        if ('\u{064B}'..='\u{0652}').contains(&c) || c == '\u{0640}' || c == '\u{0670}' {
            continue;
        }
        match map_arabic_digit(c) {
            Some(d) => out.push(d),
            None => {
                if c.is_ascii_alphanumeric() {
                    out.push(c.to_ascii_lowercase());
                } else if is_arabic_letter(c) {
                    out.push(c);
                }
                // Punctuation and whitespace are dropped.
            }
        }
    }
    out
}

fn map_arabic_digit(c: char) -> Option<char> {
    let digits = [
        '٠', '١', '٢', '٣', '٤', '٥', '٦', '٧', '٨', '٩',
        '۰', '۱', '۲', '۳', '۴', '۵', '۶', '۷', '۸', '۹',
    ];
    for (i, d) in digits.iter().enumerate() {
        if c == *d {
            return Some(char::from(b'0' + (i % 10) as u8));
        }
    }
    None
}

fn is_arabic_letter(c: char) -> bool {
    ('\u{0621}'..='\u{064A}').contains(&c) || c == '\u{060C}' || c == '\u{061B}'
}

fn has_any(hay: &str, needles: &[&str]) -> bool {
    needles.iter().any(|n| hay.contains(n))
}

/// Canonical column concepts (pre-kind).
const SEQ_STRONG: &[&str] = &[
    "الرقمالترتيبي", "رقمالترتيبي", "الرقمالرقم", "الترتيبي", "التسلسلي", "تسلسلي",
    "numerodordre", "numordred", "ordred", "ordinateur", "زوم",
];
const SEQ_WEAK: &[&str] = &["الرقم", "رقم", "numero", "num", "no", "n", "number", "sequence"];

const SUBJECT: &[&str] = &["الموضوع", "المواضيع", "objet", "sujet", "subject", "الغرض"];
const RECIPIENT: &[&str] = &[
    "المرسلاليه", "المرسلالية", "الموجهاليه", "الموجهإليه", "destinataire", "المرسلية",
];
const OUT_DATE: &[&str] = &["تاريخالصادرة", "تاريخالصادر", "تارخالصادرة", "الادات", "dade", "date", "تارخ"];
const IN_DATE: &[&str] = &[
    "تاريخالواردة", "تاريخالوارد", "تارخالواردة", "tarihالاستقبال", "dateالاستقبال",
    "datedereception", "daterecu", "reception", "recus", "الاستقبال",
];
const SOURCE: &[&str] = &[
    "المصدر", "الجواب", "الرد", "reponse", "réponse", "source", "الامر", "مرجع",
];
const SENDER: &[&str] = &["المرسل", "expediteur", "expéditeur", "المرسلة"];
const ARRIVAL: &[&str] = &["تاريخالوصول", "تارخالوصول", "تاريخالقدوم", "arrive", "arrivé"];
const RESULT: &[&str] = &["النتيجة", "النتائج", "resultat", "résultat", "result", "الخلاصة"];
const DEST: &[&str] = &["المصلحة", "الخدمة", "المديرية", "service", "direction"];

/// Group detector keywords used to recognise parent/grouped headers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum GroupKey {
    Outgoing,
    Incoming,
    Result,
    Other,
}

fn detect_group(n: &str) -> Option<GroupKey> {
    if has_any(n, &["خرج", "صادر", "صادرة", "الصادرة", "sortie", "outgoing", "المنصرف"]) {
        return Some(GroupKey::Outgoing);
    }
    if has_any(n, &["دخل", "وارد", "واردة", "الواردة", "entrée", "entree", "reçu", "recu", "incoming", "رد"]) {
        return Some(GroupKey::Incoming);
    }
    if has_any(n, &["النتيجة", "النتائج", "resultat", "result"]) {
        return Some(GroupKey::Result);
    }
    None
}

fn group_scope_str(g: GroupKey) -> &'static str {
    match g {
        GroupKey::Outgoing => "outgoing",
        GroupKey::Incoming => "incoming",
        GroupKey::Result => "result",
        GroupKey::Other => "other",
    }
}

/// Detect the column concept using keywords and (optional) group context, then
/// resolve it to a concrete application field string for the given kind.
fn resolve_field(header: &str, group: &Option<GroupKey>, kind: ExcelKind) -> Option<String> {
    let n = norm(header);
    if n.is_empty() {
        return None;
    }

    // Groups give strong context and take priority.
    match group {
        Some(GroupKey::Result) => {
            return Some("notes".to_string());
        }
        Some(GroupKey::Outgoing) => {
            if has_any(&n, SUBJECT) {
                return Some("subject".to_string());
            }
            if has_any(&n, RECIPIENT) || has_any(&n, &["المستلم", "الالية"]) {
                return Some("recipient".to_string());
            }
            // Otherwise a date under an outgoing group is the outgoing date.
            if has_any(&n, OUT_DATE) || has_any(&n, IN_DATE) {
                return Some("date".to_string());
            }
            return None;
        }
        Some(GroupKey::Incoming) => {
            if has_any(&n, SUBJECT) {
                return Some("subject".to_string());
            }
            if has_any(&n, SOURCE) {
                return Some("source".to_string());
            }
            if has_any(&n, IN_DATE) || has_any(&n, OUT_DATE) || has_any(&n, &["التاريخ", "date"]) {
                return if kind == ExcelKind::Outgoing {
                    Some("correspondence_number".to_string())
                } else {
                    Some("date".to_string())
                };
            }
            return None;
        }
        _ => {}
    }

    // No group context: rely on keyword precedence.
    if has_any(&n, SEQ_STRONG) {
        return Some("registration_number".to_string());
    }
    if has_any(&n, SUBJECT) {
        return Some("subject".to_string());
    }
    if has_any(&n, RECIPIENT) {
        return Some("recipient".to_string());
    }
    if has_any(&n, SENDER) && !has_any(&n, RECIPIENT) {
        // "المرسل" is the sender for incoming; for outgoing there is no sender.
        return if kind == ExcelKind::Incoming {
            Some("sender".to_string())
        } else {
            None
        };
    }
    if has_any(&n, ARRIVAL) {
        return if kind == ExcelKind::Incoming {
            Some("arrival_date".to_string())
        } else {
            None
        };
    }
    if has_any(&n, IN_DATE) {
        return if kind == ExcelKind::Outgoing {
            Some("correspondence_number".to_string())
        } else {
            Some("date".to_string())
        };
    }
    if has_any(&n, OUT_DATE) {
        return Some("date".to_string());
    }
    if has_any(&n, RESULT) {
        return Some("notes".to_string());
    }
    if has_any(&n, SOURCE) {
        return Some("source".to_string());
    }
    if has_any(&n, DEST) {
        return Some("destination_service".to_string());
    }
    // Weak sequential indicators (only when the cell looks like a number header).
    if has_any(&n, SEQ_WEAK) {
        return Some("registration_number".to_string());
    }
    None
}

// ---------------------------------------------------------------------------
// Excel value -> string helpers
// ---------------------------------------------------------------------------

fn cell_to_string(data: &Data) -> Option<String> {
    match data {
        Data::Int(i) => Some(i.to_string()),
        Data::Float(f) => Some(format_number(*f)),
        Data::String(s) => {
            let s = s.trim();
            if s.is_empty() { None } else { Some(s.to_string()) }
        }
        Data::DateTime(dt) => Some(format_datetime_f64(dt.as_f64())),
        Data::Bool(b) => Some(if *b { "true" } else { "false" }.to_string()),
        Data::DateTimeIso(s) | Data::DurationIso(s) => {
            Some(normalize_date_str(s))
        }
        Data::Error(_) | Data::Empty => None,
    }
}

fn format_number(f: f64) -> String {
    if f == f.trunc() && f.abs() < 1e15 {
        format!("{}", f as i64)
    } else {
        format!("{}", f)
    }
}

/// calamine exposes Excel serial dates as fractional days since 1899-12-30.
/// Convert to YYYY-MM-DD. Applies the standard Excel 1900 leap-year offset.
fn format_datetime_f64(days: f64) -> String {
    // Values >= 60 are already in the modern 1900 date system; the phantom
    // day before 1900-03-01 is skipped, so serials below 60 get shifted.
    let f = if days >= 60.0 { days } else { days + 1.0 };
    let secs = ((f - 25569.0) * 86400.0) as i64;
    let dt = Utc.timestamp_opt(secs, 0).single();
    match dt {
        Some(dt) => dt.format("%Y-%m-%d").to_string(),
        None => String::new(),
    }
}

/// Normalise human-readable date strings to YYYY-MM-DD when possible.
fn normalize_date_str(s: &str) -> String {
    let s = s.trim();
    let mut cleaned = String::new();
    for c in s.chars() {
        if c.is_ascii_digit() || c == '/' || c == '-' || c == '.' {
            cleaned.push(c);
        } else if let Some(d) = map_arabic_digit(c) {
            cleaned.push(d);
        } else if c == ' ' {
            cleaned.push(' ');
        }
    }
    let cleaned = cleaned.split_whitespace().collect::<Vec<_>>().join(" ");

    // Try dd/mm/yyyy, dd-mm-yyyy, dd.mm.yyyy
    for sep in ['/', '-', '.'] {
        let parts: Vec<&str> = cleaned.split(sep).collect();
        if parts.len() == 3 {
            let a = parts[0].trim();
            let b = parts[1].trim();
            let c = parts[2].trim();
            if let (Ok(da), Ok(db), Ok(dc)) = (a.parse::<i32>(), b.parse::<i32>(), c.parse::<i32>()) {
                if dc >= 2000 && dc < 2100 {
                    return format!("{:04}-{:02}-{:02}", dc, db, da);
                }
                if da >= 2000 && da < 2100 && db <= 12 && dc <= 31 {
                    return format!("{:04}-{:02}-{:02}", da, db, dc);
                }
            }
        }
    }
    // Already an ISO date: keep it.
    if cleaned.len() >= 10 && cleaned.chars().all(|c| c.is_ascii_digit() || c == '-') {
        return cleaned[..10].to_string();
    }
    s.to_string()
}

// ---------------------------------------------------------------------------
// Workbook reading / analysis
// ---------------------------------------------------------------------------

const MAX_PREVIEW_ROWS: usize = 50;
const HEADER_SCAN_ROWS: usize = 10;

struct SheetGrid {
    values: Vec<Vec<Option<String>>>,
}

fn range_to_grid(range: &Range<Data>) -> SheetGrid {
    let mut values: Vec<Vec<Option<String>>> = Vec::new();
    for row in range.rows() {
        let mut r = Vec::with_capacity(row.len());
        for cell in row {
            r.push(cell_to_string(cell));
        }
        values.push(r);
    }
    SheetGrid { values }
}

/// Detect whether a workbook looks like incoming or outgoing and how confident.
fn detect_kind(grid: &SheetGrid) -> (ExcelKind, bool) {
    let mut out_score = 0i32;
    let mut in_score = 0i32;

    for row in grid.values.iter().take(HEADER_SCAN_ROWS * 2) {
        for cell in row.iter().flatten() {
            let n = norm(cell);
            if n.is_empty() {
                continue;
            }
            if has_any(&n, &["المصدر", "المرسل إليه", "المرسلاليه", "destinataire", "المرسلية"])
                || has_any(&n, &["date", "تاريخالصادرة", "الصادرة", "sortie"])
            {
                out_score += 1;
            }
            if has_any(&n, &["تاريخالواردة", "الواردة", "datedereception", "reception", "الاستقبال"])
                || has_any(&n, &["المصدر والجواب", "المرسل"])
            {
                in_score += 1;
            }
        }
    }

    // A clear distinction favours the dominant score.
    if out_score > in_score && out_score - in_score >= 2 {
        (ExcelKind::Outgoing, true)
    } else if in_score > out_score && in_score - out_score >= 2 {
        (ExcelKind::Incoming, true)
    } else if out_score + in_score == 0 {
        // Unknown: default to outgoing but not confident.
        (ExcelKind::Outgoing, false)
    } else {
        // Close tie.
        if out_score >= in_score {
            (ExcelKind::Outgoing, false)
        } else {
            (ExcelKind::Incoming, false)
        }
    }
}

fn analyze_grid(grid: &SheetGrid) -> ExcelAnalysis {
    // 1) Determine groups from the top rows.
    //    Group rows: a row that contains at least one group keyword cell.
    //    Within such a row, a group cell spans until the next non-empty cell.
    let mut group_ranges: Vec<Vec<(GroupKey, usize, usize)>> = Vec::new(); // per row

    for row in grid.values.iter().take(HEADER_SCAN_ROWS) {
        let mut row_groups: Vec<(GroupKey, usize, usize)> = Vec::new();
        let mut col = 0usize;
        while col < row.len() {
            if let Some(v) = &row[col] {
                let n = norm(v);
                let mut key: Option<GroupKey> = detect_group(&n);
                // Also allow grouping cells that are just "خرج/دخول" style labels.
                if key.is_none()
                    && (has_any(&n, &["خرج", "دخول", "وارد", "صادر", "نتيجة", "sortie", "entree", "incoming", "outgoing", "reçu", "recu"]))
                {
                    key = detect_group(&n);
                }
                if let Some(k) = key {
                    // Advance until next non-empty cell in this row.
                    let mut end = col + 1;
                    while end < row.len() {
                        match &row[end] {
                            Some(v) if !norm(v).is_empty() => break,
                            _ => end += 1,
                        }
                    }
                    if k != GroupKey::Other {
                        row_groups.push((k, col, end));
                        col = end;
                        continue;
                    }
                }
            }
            col += 1;
        }
        if !row_groups.is_empty() {
            group_ranges.push(row_groups);
        }
    }

    // 2) Determine the data start: the first row after the last meaningful
    //    header row. A meaningful header row is one that shares at least one
    //    column with a detected group OR contains a recognised keyword.
    let mut data_start;
    {
        let mut last_header = 0usize;
        for (ri, row) in grid.values.iter().take(HEADER_SCAN_ROWS).enumerate() {
            let mut meaningful = false;
            for cell in row.iter().flatten() {
                let n = norm(cell);
                if n.is_empty() {
                    continue;
                }
                if detect_group(&n).is_some()
                    || has_any(&n, SEQ_STRONG)
                    || has_any(&n, SEQ_WEAK)
                    || has_any(&n, SUBJECT)
                    || has_any(&n, &["التاريخ", "date", "المرسل", "destinataire", "المستلم"])
                    || has_any(&n, RESULT)
                {
                    meaningful = true;
                    break;
                }
            }
            if meaningful {
                last_header = ri + 1;
            }
        }
        data_start = last_header.max(HEADER_SCAN_ROWS.min(grid.values.len()));
        // Skip blank rows after the header.
        while data_start < grid.values.len() && is_blank_row(&grid.values[data_start]) {
            data_start += 1;
        }
    }

    // 3) Determine the sheet kind (once).
    let (kind, kind_confident) = detect_kind(grid);

    // 4) Build columns using the last header row labels, enriched by groups.
    let label_row_idx = if data_start > 0 { data_start - 1 } else { 0 };
    let label_row = if label_row_idx < grid.values.len() {
        grid.values[label_row_idx].clone()
    } else {
        Vec::new()
    };

    let mut columns: Vec<ExcelColumn> = Vec::new();
    let width = label_row.len().max(1);
    for ci in 0..width {
        let header_cell = label_row.get(ci).and_then(|v| v.clone()).unwrap_or_default();
        let group = group_at(&group_ranges, ci, data_start);
        let field = resolve_field(&header_cell, &group, kind);
        let group_str = group.map(group_scope_str).map(|s| s.to_string());
        let header = build_header_label(&header_cell, &group_str);
        columns.push(ExcelColumn { header, group: group_str, field });
    }

    // Trim trailing fully-mapped-but-empty, but keep columns of interest only.
    columns.retain(|c| c.field.is_some() || !c.header.is_empty());

    // 5) Read data rows.
    let mut total_rows = 0usize;
    let mut preview: Vec<ExcelPreviewRow> = Vec::new();
    let mut rows: Vec<ExcelDataRow> = Vec::new();
    let mut valid_rows = 0usize;
    let mut invalid_rows = 0usize;
    let mut duplicate_rows = 0usize;
    let mut sample_issues: Vec<RowIssue> = Vec::new();
    let mut seen_numbers: std::collections::HashSet<String> = std::collections::HashSet::new();

    let ncols = columns.len();
    let seq_idx = columns.iter().position(|c| c.field.as_deref() == Some("registration_number"));

    for (ri, row) in grid.values.iter().enumerate().skip(data_start) {
        if is_blank_row(row) {
            continue;
        }
        let source_row = (ri + 1) as u32;
        total_rows += 1;

        // Extract values mapped to columns.
        let mut vals: Vec<Option<String>> = Vec::with_capacity(ncols);
        for ci in 0..ncols {
            let mut v = row.get(ci).and_then(|x| x.clone());
            // Normalise dates for mapped date fields.
            if let Some(c) = columns.get(ci) {
                let f = c.field.as_deref();
                if f == Some("date") || f == Some("arrival_date") || f == Some("correspondence_number") {
                    if let Some(s) = v.take() {
                        v = Some(normalize_date_str(&s));
                    }
                }
            }
            vals.push(v);
        }

        // Validate: at least one non-empty mapped cell must exist to be a record.
        let has_data = vals.iter().any(|v| v.as_deref().map(|s| !s.is_empty()).unwrap_or(false));

        // Strip the "مكرر" suffix from the sequential number and capture the flag.
        let mut is_dup = false;
        if let Some(idx) = seq_idx {
            if let Some(num) = vals.get_mut(idx) {
                if let Some(s) = num.as_deref() {
                    if s.contains("مكرر") {
                        is_dup = true;
                    }
                }
                if let Some(s) = num.as_deref() {
                    let clean = s
                        .replace("مكرر", "")
                        .replace("مكرار", "")
                        .trim()
                        .to_string();
                    *num = if clean.is_empty() { None } else { Some(clean) };
                }
            }
        }

        // Duplicate detection within the file (sequential number).
        let mut row_valid = has_data;
        let mut row_issue: Option<String> = None;

        if let Some(idx) = seq_idx {
            let num = vals.get(idx).and_then(|v| v.clone()).unwrap_or_default();
            let num = num.trim().to_string();
            if !num.is_empty() && num.parse::<i64>().is_err() {
                row_valid = false;
                row_issue = Some("رقم ترتيبي غير صالح".to_string());
            } else if !num.is_empty() {
                if seen_numbers.contains(&num) {
                    row_valid = false;
                    row_issue = Some("رقم ترتيبي مكرر في الملف".to_string());
                    duplicate_rows += 1;
                } else {
                    seen_numbers.insert(num);
                }
            }
        }

        if row_valid {
            valid_rows += 1;
        } else {
            invalid_rows += 1;
            if sample_issues.len() < 10 {
                sample_issues.push(RowIssue {
                    source_row,
                    reason: row_issue.unwrap_or_else(|| "صف غير مكتمل".to_string()),
                });
            }
        }

        rows.push(ExcelDataRow {
            source_row,
            values: vals,
            is_duplicate: is_dup && kind == ExcelKind::Incoming,
        });

        if preview.len() < MAX_PREVIEW_ROWS {
            let cells = columns
                .iter()
                .enumerate()
                .map(|(ci, _)| {
                    let v = rows.last().unwrap().values.get(ci).and_then(|x| x.clone()).unwrap_or_default();
                    let mut s = v;
                    if ci == seq_idx.unwrap_or(usize::MAX) && is_dup {
                        s = format!("{} مكرر", s);
                    }
                    s
                })
                .collect();
            preview.push(ExcelPreviewRow { source_row, cells });
        }
    }

    ExcelAnalysis {
        file_name: String::new(), // filled by caller
        sheet_name: String::new(),
        kind: kind.as_str().to_string(),
        kind_confident,
        header_rows: data_start.min(HEADER_SCAN_ROWS),
        columns,
        rows,
        preview,
        total_rows,
        valid_rows,
        invalid_rows,
        duplicate_rows,
        sample_issues,
    }
}

fn build_header_label(cell: &str, group: &Option<String>) -> String {
    let base = if cell.trim().is_empty() { "عمود" } else { cell.trim() };
    match group {
        Some(g) => format!("{} / {}", g, base),
        None => base.to_string(),
    }
}

/// Find the group that applies to a column index, from the lowest group row.
fn group_at(
    group_ranges: &[Vec<(GroupKey, usize, usize)>],
    col: usize,
    _data_start: usize,
) -> Option<GroupKey> {
    for row_groups in group_ranges.iter().rev() {
        for (key, start, end) in row_groups {
            if col >= *start && col < *end {
                return Some(*key);
            }
        }
    }
    None
}

fn is_blank_row(row: &[Option<String>]) -> bool {
    row.iter().all(|c| match c {
        Some(s) => s.trim().is_empty(),
        None => true,
    })
}

// ---------------------------------------------------------------------------
// Commands
// ---------------------------------------------------------------------------

/// Read a workbook given an explicit path and run the analysis. Supports both
/// .xlsx and .xls; anything else is rejected early.
pub fn analyze_file(path: &str) -> Result<ExcelAnalysis, String> {
    let p = Path::new(path);
    let ext = p
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_lowercase())
        .unwrap_or_default();

    if ext != "xlsx" && ext != "xls" {
        return Err("الملف المدعوم هو .xlsx أو .xls فقط".to_string());
    }

    let mut analysis = if ext == "xlsx" {
        let wb: calamine::Xlsx<_> =
            calamine::open_workbook(p).map_err(|e| format!("ملف Excel غير صالح: {}", e))?;
        analyze_workbook(wb)
    } else {
        let wb: calamine::Xls<_> =
            calamine::open_workbook(p).map_err(|e| format!("ملف Excel غير صالح: {}", e))?;
        analyze_workbook(wb)
    };

    analysis.file_name = p
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("workbook")
        .to_string();
    Ok(analysis)
}

fn analyze_workbook<R>(mut wb: R) -> ExcelAnalysis
where
    R: Reader<BufReader<std::fs::File>>,
{
    // Choose the first non-empty worksheet; prefer one that clearly looks like data.
    let mut best: Option<(usize, String, ExcelAnalysis)> = None;
    for sheet in wb.sheet_names().to_vec() {
        match wb.worksheet_range(&sheet) {
            Ok(range) => {
                if range.is_empty() {
                    continue;
                }
                let grid = range_to_grid(&range);
                if grid.values.is_empty() {
                    continue;
                }
                let mut a = analyze_grid(&grid);
                a.sheet_name = sheet.clone();
                let score = a.valid_rows + a.total_rows + a.columns.len();
                if best.as_ref().map_or(true, |(s, _, _)| score > *s) {
                    best = Some((score, sheet, a));
                }
            }
            Err(_) => continue,
        }
    }

    best.map(|(_, _, a)| a).unwrap_or_else(|| ExcelAnalysis {
        file_name: String::new(),
        sheet_name: String::new(),
        kind: "incoming".to_string(),
        kind_confident: false,
        header_rows: 0,
        columns: Vec::new(),
        rows: Vec::new(),
        preview: Vec::new(),
        total_rows: 0,
        valid_rows: 0,
        invalid_rows: 0,
        duplicate_rows: 0,
        sample_issues: Vec::new(),
    })
}

#[tauri::command]
pub fn analyze_excel(path: String) -> Result<ExcelAnalysis, String> {
    analyze_file(&path)
}

#[tauri::command]
pub fn import_excel(
    state: tauri::State<'_, AppState>,
    request: ExcelImportRequest,
) -> Result<ExcelImportResult, String> {
    let kind = ExcelKind::from_str(&request.kind);
    let now = Utc::now();
    let now_str = now.format("%Y-%m-%dT%H:%M:%SZ").to_string();

    let mut result = ExcelImportResult {
        total: request.rows.len(),
        imported: 0,
        skipped: 0,
        duplicates: 0,
        errors: 0,
        failures: Vec::new(),
    };

    // Track sequential numbers already used in the DB for this kind to avoid
    // accidental duplicates, plus numbers seen within this import.
    let mut seen: std::collections::HashSet<String> = std::collections::HashSet::new();

    for data_row in &request.rows {
        // Build a map: column index -> resolved db field.
        let mut record: std::collections::HashMap<String, String> = std::collections::HashMap::new();
        let is_duplicate = if kind == ExcelKind::Incoming {
            data_row.is_duplicate
        } else {
            false
        };

        for (ci, col) in request.columns.iter().enumerate() {
            let Some(field) = col.field.clone() else { continue };
            let value = data_row
                .values
                .get(ci)
                .and_then(|v| v.clone())
                .unwrap_or_default()
                .trim()
                .to_string();

            // The sequential number was already stripped of any "مكرر" suffix
            // during analysis; the explicit flag is carried on the row.
            if field == "registration_number" {
                let num = value
                    .replace("مكرر", "")
                    .replace("مكرار", "")
                    .trim()
                    .to_string();
                if !num.is_empty() {
                    record.insert(field.clone(), num);
                }
                continue;
            }

            if !value.is_empty() {
                record.insert(field.clone(), value);
            }
        }

        // ---- Validation ---------------------------------------------------
        let mut fail_reason: Option<String> = None;

        let seq = record.get("registration_number").cloned();
        let seq_valid = match &seq {
            Some(s) => s.parse::<i64>().is_ok(),
            None => false,
        };

        if !seq_valid {
            fail_reason = Some("رقم ترتيبي غير صالح".to_string());
        } else if let Some(s) = &seq {
            if seen.contains(s) {
                fail_reason = Some("رقم ترتيبي مكرر".to_string());
            } else if db_has_number(&state, kind, s) {
                fail_reason = Some("الرقم موجود مسبقاً في قاعدة البيانات".to_string());
            }
        }

        let required: &[&str] = match kind {
            ExcelKind::Incoming => &["date", "subject", "sender"],
            ExcelKind::Outgoing => &["recipient", "subject"],
        };
        for rf in required {
            if record.get(*rf).map(|v| v.is_empty()).unwrap_or(true) {
                if fail_reason.is_none() {
                    fail_reason = Some(format!("حقل مطلوب ناقص: {}", *rf));
                }
                break;
            }
        }

        // Dates must be valid if present.
        for dfield in ["date", "arrival_date", "correspondence_number"] {
            if let Some(v) = record.get(dfield) {
                if !v.is_empty() && normalize_date_str(v).len() < 10 {
                    if fail_reason.is_none() {
                        fail_reason = Some(format!("تاريخ غير صالح: {}", v));
                    }
                    break;
                }
            }
        }

        // ---- Insert -------------------------------------------------------
        if let Some(reason) = fail_reason {
            result.errors += 1;
            result.failures.push(RowFailure {
                source_row: data_row.source_row,
                reason,
            });
            continue;
        }

        let seq = record.get("registration_number").cloned().unwrap_or_default();
        if seen.insert(seq.clone()) {
            let insert_result = match kind {
                ExcelKind::Incoming => insert_incoming(&state, &now_str, &seq, is_duplicate, &record),
                ExcelKind::Outgoing => insert_outgoing(&state, &now_str, &seq, &record),
            };

            match insert_result {
                Ok(()) => {
                    result.imported += 1;
                }
                Err(e) => {
                    result.errors += 1;
                    result.failures.push(RowFailure {
                        source_row: data_row.source_row,
                        reason: e,
                    });
                }
            }
        } else {
            result.duplicates += 1;
        }
    }

    result.skipped = result.duplicates + result.errors;
    Ok(result)
}

fn db_has_number(state: &tauri::State<'_, AppState>, kind: ExcelKind, number: &str) -> bool {
    let table = match kind {
        ExcelKind::Incoming => "incoming",
        ExcelKind::Outgoing => "outgoing",
    };
    let sql = format!(
        "SELECT COUNT(*) FROM {} WHERE registration_number = ?1 AND deleted_at IS NULL",
        table
    );
    state
        .db
        .query_row(&sql, &[&number], |r| r.get::<_, i64>(0))
        .unwrap_or(0)
        > 0
}

fn insert_incoming(
    state: &tauri::State<'_, AppState>,
    now: &str,
    seq: &str,
    is_duplicate: bool,
    rec: &std::collections::HashMap<String, String>,
) -> Result<(), String> {
    let id = Uuid::new_v4().to_string();
    state
        .db
        .execute(
            "INSERT INTO incoming (id, registration_number, correspondence_number, date, arrival_date, subject, sender, destination_service, source, notes, is_duplicate, created_at, updated_at, sync_version) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, 1)",
            &[
                &&*id,
                &seq,
                &rec.get("correspondence_number").map(String::as_str),
                &rec.get("date").map(String::as_str).unwrap_or_default(),
                &rec.get("arrival_date").map(String::as_str),
                &rec.get("subject").map(String::as_str).unwrap_or_default(),
                &rec.get("sender").map(String::as_str).unwrap_or_default(),
                &rec.get("destination_service")
                    .map(String::as_str)
                    .unwrap_or_default(),
                &rec.get("source").map(String::as_str),
                &rec.get("notes").map(String::as_str),
                &is_duplicate,
                &now,
                &now,
            ],
        )
        .map_err(|e| format!("فشل إدخال السجل: {}", e))?;
    Ok(())
}

fn insert_outgoing(
    state: &tauri::State<'_, AppState>,
    now: &str,
    seq: &str,
    rec: &std::collections::HashMap<String, String>,
) -> Result<(), String> {
    let id = Uuid::new_v4().to_string();
    let recipient = rec.get("recipient").cloned().unwrap_or_default();
    let subject = rec.get("subject").cloned().unwrap_or_default();
    let date = rec.get("date").cloned().unwrap_or_default();
    state
        .db
        .execute(
            "INSERT INTO outgoing (id, registration_number, correspondence_number, date, subject, recipient, destination_service, source, notes, created_at, updated_at, sync_version) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, 1)",
            &[
                &&*id,
                &seq,
                &rec.get("correspondence_number").map(String::as_str),
                &date,
                &subject,
                &recipient,
                &rec.get("destination_service")
                    .map(String::as_str)
                    .unwrap_or_default(),
                &rec.get("source").map(String::as_str),
                &rec.get("notes").map(String::as_str),
                &now,
                &now,
            ],
        )
        .map_err(|e| format!("فشل إدخال السجل: {}", e))?;
    Ok(())
}

// ---------------------------------------------------------------------------
// Excel template generation
// ---------------------------------------------------------------------------

fn build_template(path: &str) -> Result<(), String> {
    let file = std::fs::File::create(path).map_err(|e| format!("فشل إنشاء الملف: {}", e))?;
    let mut zip = zip::ZipWriter::new(file);
    let opts = SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated)
        .compression_level(Some(6));

    let incoming_headers = [
        "الرقم الترتيبي", "رقم المراسلة", "تاريخ الرسالة", "تاريخ الوصول",
        "الموضوع", "المرسل", "المصلحة", "المصدر والجواب", "ملاحظات",
    ];
    let outgoing_headers = [
        "الرقم الترتيبي", "رقم المراسلة", "تاريخ الرسالة",
        "الموضوع", "المستلم", "المصلحة", "المصدر والجواب", "ملاحظات",
    ];

    // Collect all shared strings.
    let mut strings: Vec<String> = Vec::new();
    let mut string_index = std::collections::HashMap::new();

    let add_str = |s: &str, strings: &mut Vec<String>, index: &mut std::collections::HashMap<String, usize>| -> usize {
        if let Some(&i) = index.get(s) {
            return i;
        }
        let i = strings.len();
        strings.push(s.to_string());
        index.insert(s.to_string(), i);
        i
    };

    // Add headers to shared strings.
    for h in &incoming_headers {
        add_str(h, &mut strings, &mut string_index);
    }
    for h in &outgoing_headers {
        add_str(h, &mut strings, &mut string_index);
    }

    // --- [Content_Types].xml ---
    zip.start_file("[Content_Types].xml", opts.clone()).map_err(|e| e.to_string())?;
    zip.write_all(CONTENT_TYPES_XML.as_bytes()).map_err(|e| e.to_string())?;

    // --- _rels/.rels ---
    zip.start_file("_rels/.rels", opts.clone()).map_err(|e| e.to_string())?;
    zip.write_all(RELS_XML.as_bytes()).map_err(|e| e.to_string())?;

    // --- xl/_rels/workbook.xml.rels ---
    zip.start_file("xl/_rels/workbook.xml.rels", opts.clone()).map_err(|e| e.to_string())?;
    zip.write_all(WORKBOOK_RELS_XML.as_bytes()).map_err(|e| e.to_string())?;

    // --- xl/styles.xml ---
    zip.start_file("xl/styles.xml", opts.clone()).map_err(|e| e.to_string())?;
    zip.write_all(STYLES_XML.as_bytes()).map_err(|e| e.to_string())?;

    // --- xl/workbook.xml ---
    zip.start_file("xl/workbook.xml", opts.clone()).map_err(|e| e.to_string())?;
    zip.write_all(WORKBOOK_XML.as_bytes()).map_err(|e| e.to_string())?;

    // --- xl/sharedStrings.xml ---
    {
        let mut ss = String::from(
            "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\n\
             <sst xmlns=\"http://schemas.openxmlformats.org/spreadsheetml/2006/main\" \
             count=\"0\" uniqueCount=\"0\">"
        );
        for s in &strings {
            let escaped = s.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;");
            ss.push_str(&format!("<si><t>{}</t></si>", escaped));
        }
        ss.push_str("</sst>");
        zip.start_file("xl/sharedStrings.xml", opts.clone()).map_err(|e| e.to_string())?;
        zip.write_all(ss.as_bytes()).map_err(|e| e.to_string())?;
    }

    // --- xl/worksheets/sheet1.xml (واردات) ---
    {
        let sheet_xml = build_sheet_xml(&incoming_headers, &string_index, 50);
        zip.start_file("xl/worksheets/sheet1.xml", opts.clone()).map_err(|e| e.to_string())?;
        zip.write_all(sheet_xml.as_bytes()).map_err(|e| e.to_string())?;
    }

    // --- xl/worksheets/sheet2.xml (صادرات) ---
    {
        let sheet_xml = build_sheet_xml(&outgoing_headers, &string_index, 50);
        zip.start_file("xl/worksheets/sheet2.xml", opts.clone()).map_err(|e| e.to_string())?;
        zip.write_all(sheet_xml.as_bytes()).map_err(|e| e.to_string())?;
    }

    zip.finish().map_err(|e| format!("فشل حفظ الملف: {}", e))?;
    Ok(())
}

fn col_letter(idx: usize) -> String {
    let mut s = String::new();
    let mut i = idx;
    loop {
        s.push((b'A' + (i % 26) as u8) as char);
        i /= 26;
        if i == 0 { break; }
        i -= 1;
    }
    s
}

fn build_sheet_xml(headers: &[&str], strings: &std::collections::HashMap<String, usize>, empty_rows: u32) -> String {
    let mut xml = String::from(
        "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\n\
         <worksheet xmlns=\"http://schemas.openxmlformats.org/spreadsheetml/2006/main\" \
         xmlns:r=\"http://schemas.openxmlformats.org/officeDocument/2006/relationships\">\n\
         <sheetViews><sheetView tabSelected=\"1\" workbookViewId=\"0\" rightToLeft=\"1\"/></sheetViews>\n"
    );

    // Column widths.
    xml.push_str("<cols>");
    let widths: Vec<f64> = headers.iter().map(|h| {
        // Approximate width based on character count.
        if h.contains("الموضوع") || h.contains("ملاحظات") { 35.0 }
        else if h.contains("المرسل") || h.contains("المستلم") || h.contains("المصلحة") || h.contains("المصدر") { 25.0 }
        else { 18.0 }
    }).collect();
    for (i, w) in widths.iter().enumerate() {
        let min = (i + 1) as u32;
        xml.push_str(&format!("<col min=\"{}\" max=\"{}\" width=\"{}\" customWidth=\"1\"/>", min, min, w));
    }
    xml.push_str("</cols>");

    xml.push_str("<sheetData>");

    // Header row (row 1), bold style index 1.
    xml.push_str("<row r=\"1\" ht=\"30\">");
    for (ci, h) in headers.iter().enumerate() {
        let col = col_letter(ci);
        let idx = strings.get(*h).unwrap_or(&0);
        xml.push_str(&format!(
            "<c r=\"{}1\" t=\"s\" s=\"1\"><v>{}</v></c>",
            col, idx
        ));
    }
    xml.push_str("</row>");

    // Empty rows with borders.
    for row in 2..=(empty_rows + 1) {
        xml.push_str(&format!("<row r=\"{}\">", row));
        for ci in 0..headers.len() {
            let col = col_letter(ci);
            xml.push_str(&format!("<c r=\"{}{}\" s=\"2\"/>", col, row));
        }
        xml.push_str("</row>");
    }

    xml.push_str("</sheetData></worksheet>");
    xml
}

// Static XML constants for the xlsx structure.
const CONTENT_TYPES_XML: &str = "\
<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\
<Types xmlns=\"http://schemas.openxmlformats.org/package/2006/content-types\">\
<Default Extension=\"rels\" ContentType=\"application/vnd.openxmlformats-package.relationships+xml\"/>\
<Default Extension=\"xml\" ContentType=\"application/xml\"/>\
<Override PartName=\"/xl/workbook.xml\" ContentType=\"application/vnd.openxmlformats-officedocument.spreadsheetml.sheet.main+xml\"/>\
<Override PartName=\"/xl/worksheets/sheet1.xml\" ContentType=\"application/vnd.openxmlformats-officedocument.spreadsheetml.worksheet+xml\"/>\
<Override PartName=\"/xl/worksheets/sheet2.xml\" ContentType=\"application/vnd.openxmlformats-officedocument.spreadsheetml.worksheet+xml\"/>\
<Override PartName=\"/xl/styles.xml\" ContentType=\"application/vnd.openxmlformats-officedocument.spreadsheetml.styles+xml\"/>\
<Override PartName=\"/xl/sharedStrings.xml\" ContentType=\"application/vnd.openxmlformats-officedocument.spreadsheetml.sharedStrings+xml\"/>\
</Types>";

const RELS_XML: &str = "\
<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\
<Relationships xmlns=\"http://schemas.openxmlformats.org/package/2006/relationships\">\
<Relationship Id=\"rId1\" Type=\"http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument\" Target=\"xl/workbook.xml\"/>\
</Relationships>";

const WORKBOOK_XML: &str = "\
<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\
<workbook xmlns=\"http://schemas.openxmlformats.org/spreadsheetml/2006/main\" \
xmlns:r=\"http://schemas.openxmlformats.org/officeDocument/2006/relationships\">\
<sheets>\
<sheet name=\"واردات\" sheetId=\"1\" r:id=\"rId1\"/>\
<sheet name=\"صادرات\" sheetId=\"2\" r:id=\"rId2\"/>\
</sheets></workbook>";

const WORKBOOK_RELS_XML: &str = "\
<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\
<Relationships xmlns=\"http://schemas.openxmlformats.org/package/2006/relationships\">\
<Relationship Id=\"rId1\" Type=\"http://schemas.openxmlformats.org/officeDocument/2006/relationships/worksheet\" Target=\"worksheets/sheet1.xml\"/>\
<Relationship Id=\"rId2\" Type=\"http://schemas.openxmlformats.org/officeDocument/2006/relationships/worksheet\" Target=\"worksheets/sheet2.xml\"/>\
<Relationship Id=\"rId3\" Type=\"http://schemas.openxmlformats.org/officeDocument/2006/relationships/styles\" Target=\"styles.xml\"/>\
<Relationship Id=\"rId4\" Type=\"http://schemas.openxmlformats.org/officeDocument/2006/relationships/sharedStrings\" Target=\"sharedStrings.xml\"/>\
</Relationships>";

const STYLES_XML: &str = "\
<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\
<styleSheet xmlns=\"http://schemas.openxmlformats.org/spreadsheetml/2006/main\">\
<numFmts count=\"1\">\
<numFmt numFmtId=\"14\" formatCode=\"yyyy/mm/dd\"/>\
</numFmts>\
<fonts count=\"2\">\
<font><sz val=\"11\"/><name val=\"Arial\"/></font>\
<font><b/><sz val=\"12\"/><name val=\"Arial\"/></font>\
</fonts>\
<fills count=\"3\">\
<fill><patternFill patternType=\"none\"/></fill>\
<fill><patternFill patternType=\"gray125\"/></fill>\
<fill><patternFill patternType=\"solid\"><fgColor rgb=\"FFD6EAF8\"/></patternFill></fill>\
</fills>\
<borders count=\"3\">\
<border><left/><right/><top/><bottom/><diagonal/></border>\
<border>\
<left style=\"medium\"/><right style=\"medium\"/><top style=\"medium\"/><bottom style=\"medium\"/><diagonal/>\
</border>\
<border>\
<left style=\"thin\"/><right style=\"thin\"/><top style=\"thin\"/><bottom style=\"thin\"/><diagonal/>\
</border>\
</borders>\
<cellStyleXfs count=\"1\"><xf numFmtId=\"0\" fontId=\"0\" fillId=\"0\" borderId=\"0\"/></cellStyleXfs>\
<cellXfs count=\"3\">\
<xf numFmtId=\"0\" fontId=\"0\" fillId=\"0\" borderId=\"0\" xfId=\"0\"/>\
<xf numFmtId=\"0\" fontId=\"1\" fillId=\"2\" borderId=\"1\" xfId=\"0\" applyFont=\"1\" applyFill=\"1\" applyBorder=\"1\" applyAlignment=\"1\"><alignment horizontal=\"center\" vertical=\"center\" wrapText=\"1\"/></xf>\
<xf numFmtId=\"0\" fontId=\"0\" fillId=\"0\" borderId=\"2\" xfId=\"0\" applyBorder=\"1\" applyAlignment=\"1\"><alignment vertical=\"center\"/></xf>\
</cellXfs>\
</styleSheet>";

#[tauri::command]
pub fn generate_excel_template() -> Result<String, String> {
    let temp_dir = std::env::temp_dir();
    let file_path = temp_dir.join("قالب_الاستيراد.xlsx");
    let path_str = file_path.to_str().unwrap_or("template.xlsx");

    build_template(path_str)?;

    // Open the file with the default application.
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        std::process::Command::new("cmd")
            .args(["/C", "start", "", path_str])
            .creation_flags(0x08000000)
            .spawn()
            .map_err(|e| format!("فشل فتح الملف: {}", e))?;
    }
    #[cfg(not(windows))]
    {
        std::process::Command::new("xdg-open")
            .arg(path_str)
            .spawn()
            .map_err(|e| format!("فشل فتح الملف: {}", e))?;
    }

    Ok(path_str.to_string())
}
