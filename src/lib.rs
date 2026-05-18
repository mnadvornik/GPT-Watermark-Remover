use eframe::egui;
use egui::text::{LayoutJob, TextFormat};
use egui::{Color32, FontId, Margin, RichText, Stroke, Vec2};
use std::io::{Cursor, Write};
use std::path::{Path, PathBuf};
use unicode_general_category::{GeneralCategory, get_general_category};
use unicode_normalization::UnicodeNormalization;
use zip::write::SimpleFileOptions;
use zip::{CompressionMethod, ZipWriter};

const INVISIBLE_CODEPOINTS: &[char] = &[
    '\u{00AD}', '\u{034F}', '\u{061C}', '\u{115F}', '\u{1160}', '\u{17B4}', '\u{17B5}', '\u{180B}',
    '\u{180C}', '\u{180D}', '\u{180E}', '\u{180F}', '\u{200B}', '\u{200C}', '\u{200D}', '\u{200E}',
    '\u{200F}', '\u{202A}', '\u{202B}', '\u{202C}', '\u{202D}', '\u{202E}', '\u{2060}', '\u{2061}',
    '\u{2062}', '\u{2063}', '\u{2064}', '\u{2066}', '\u{2067}', '\u{2068}', '\u{2069}', '\u{206A}',
    '\u{206B}', '\u{206C}', '\u{206D}', '\u{206E}', '\u{206F}', '\u{2800}', '\u{3164}', '\u{FEFF}',
    '\u{FFA0}',
];

const APP_BG: Color32 = Color32::from_rgb(244, 246, 248);
const SURFACE: Color32 = Color32::from_rgb(255, 255, 255);
const SURFACE_ALT: Color32 = Color32::from_rgb(249, 250, 251);
const BORDER: Color32 = Color32::from_rgb(218, 224, 231);
const TEXT: Color32 = Color32::from_rgb(31, 41, 55);
const MUTED_TEXT: Color32 = Color32::from_rgb(93, 105, 120);
const PRIMARY: Color32 = Color32::from_rgb(25, 118, 90);
const PRIMARY_HOVER: Color32 = Color32::from_rgb(19, 96, 73);
const DANGER: Color32 = Color32::from_rgb(190, 48, 48);

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct CleanCounts {
    pub explicit_invisible: usize,
    pub format_chars: usize,
    pub control_chars: usize,
    pub variation_selectors: usize,
    pub em_dashes_replaced: usize,
}

impl CleanCounts {
    pub fn total(self) -> usize {
        self.explicit_invisible + self.format_chars + self.control_chars + self.variation_selectors
    }

    pub fn affected_total(self) -> usize {
        self.total() + self.em_dashes_replaced
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CleanOptions {
    pub remove_format_chars: bool,
    pub remove_controls: bool,
    pub normalize_unicode: bool,
    pub replace_em_dashes: bool,
}

impl Default for CleanOptions {
    fn default() -> Self {
        Self {
            remove_format_chars: true,
            remove_controls: true,
            normalize_unicode: true,
            replace_em_dashes: false,
        }
    }
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct CleanResult {
    pub text: String,
    pub counts: CleanCounts,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum OutputFormat {
    Txt,
    Docx,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RemovalKind {
    ExplicitInvisible,
    FormatCharacter,
    ControlCharacter,
    VariationSelector,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CleanAction {
    Remove(RemovalKind),
    ReplaceEmDash(&'static str),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Language {
    English,
    German,
}

impl Language {
    fn detect() -> Self {
        let Some(locale) = sys_locale::get_locale() else {
            return Self::English;
        };

        Self::from_locale(&locale)
    }

    fn from_locale(locale: &str) -> Self {
        let language = locale
            .split(['-', '_', '.'])
            .next()
            .unwrap_or_default()
            .to_ascii_lowercase();

        match language.as_str() {
            "de" => Self::German,
            _ => Self::English,
        }
    }

    fn label(self) -> &'static str {
        match self {
            Self::English => "English",
            Self::German => "Deutsch",
        }
    }
}

#[derive(Debug, Clone, Copy)]
enum TextKey {
    AppTitle,
    Subtitle,
    Clean,
    CopyCleaned,
    CopyOpenGptzero,
    SaveTxt,
    SaveDocx,
    Clear,
    FormatChars,
    Controls,
    NormalizeUnicode,
    ReplaceEmDashes,
    OriginalText,
    PasteSource,
    MarkedOriginal,
    CleanedText,
    ReadyToExport,
    Status,
    InitialStatus,
    Copied,
    GptzeroOpened,
    Cleared,
    SaveDialogTitle,
    TextFiles,
    WordDocuments,
    Language,
    Marked,
    ChangeableCharacters,
}

fn tr(language: Language, key: TextKey) -> &'static str {
    match language {
        Language::English => match key {
            TextKey::AppTitle => "Hidden Character Cleaner",
            TextKey::Subtitle => "Clean pasted text, inspect removals, and export results.",
            TextKey::Clean => "Clean",
            TextKey::CopyCleaned => "Copy Cleaned",
            TextKey::CopyOpenGptzero => "Copy + Open GPTZero",
            TextKey::SaveTxt => "Save TXT",
            TextKey::SaveDocx => "Save DOCX",
            TextKey::Clear => "Clear",
            TextKey::FormatChars => "Format chars",
            TextKey::Controls => "Controls",
            TextKey::NormalizeUnicode => "Normalize Unicode",
            TextKey::ReplaceEmDashes => "Replace em dashes",
            TextKey::OriginalText => "Original Text",
            TextKey::PasteSource => "Paste source text here",
            TextKey::MarkedOriginal => "Marked Original",
            TextKey::CleanedText => "Cleaned Text",
            TextKey::ReadyToExport => "Ready to copy or export",
            TextKey::Status => "Status",
            TextKey::InitialStatus => "Paste text on the left. Cleaning runs automatically.",
            TextKey::Copied => "Cleaned text copied to clipboard.",
            TextKey::GptzeroOpened => {
                "Cleaned text copied. GPTZero opened; paste manually to check."
            }
            TextKey::Cleared => "Cleared.",
            TextKey::SaveDialogTitle => "Save cleaned text",
            TextKey::TextFiles => "Text files",
            TextKey::WordDocuments => "Microsoft Word documents",
            TextKey::Language => "Language",
            TextKey::Marked => "marked",
            TextKey::ChangeableCharacters => "changeable character(s)",
        },
        Language::German => match key {
            TextKey::AppTitle => "Cleaner fuer versteckte Zeichen",
            TextKey::Subtitle => "Text bereinigen, Aenderungen pruefen und Ergebnisse exportieren.",
            TextKey::Clean => "Bereinigen",
            TextKey::CopyCleaned => "Bereinigten Text kopieren",
            TextKey::CopyOpenGptzero => "Kopieren + GPTZero oeffnen",
            TextKey::SaveTxt => "TXT speichern",
            TextKey::SaveDocx => "DOCX speichern",
            TextKey::Clear => "Leeren",
            TextKey::FormatChars => "Formatzeichen",
            TextKey::Controls => "Steuerzeichen",
            TextKey::NormalizeUnicode => "Unicode normalisieren",
            TextKey::ReplaceEmDashes => "Geviertstriche ersetzen",
            TextKey::OriginalText => "Originaltext",
            TextKey::PasteSource => "Quelltext hier einfuegen",
            TextKey::MarkedOriginal => "Markiertes Original",
            TextKey::CleanedText => "Bereinigter Text",
            TextKey::ReadyToExport => "Bereit zum Kopieren oder Exportieren",
            TextKey::Status => "Status",
            TextKey::InitialStatus => "Text links einfuegen. Die Bereinigung laeuft automatisch.",
            TextKey::Copied => "Bereinigter Text wurde in die Zwischenablage kopiert.",
            TextKey::GptzeroOpened => {
                "Bereinigter Text kopiert. GPTZero wurde geoeffnet; bitte manuell einfuegen."
            }
            TextKey::Cleared => "Geleert.",
            TextKey::SaveDialogTitle => "Bereinigten Text speichern",
            TextKey::TextFiles => "Textdateien",
            TextKey::WordDocuments => "Microsoft Word-Dokumente",
            TextKey::Language => "Sprache",
            TextKey::Marked => "markiert",
            TextKey::ChangeableCharacters => "aenderbare Zeichen",
        },
    }
}

pub fn clean_text(input: &str, options: &CleanOptions) -> CleanResult {
    let normalized;
    let source = if options.normalize_unicode {
        normalized = input.nfc().collect::<String>();
        normalized.as_str()
    } else {
        input
    };

    let mut result = CleanResult::default();

    for character in source.chars() {
        match clean_action(character, options) {
            Some(CleanAction::Remove(kind)) => result.counts.add(kind),
            Some(CleanAction::ReplaceEmDash(replacement)) => {
                result.counts.em_dashes_replaced += 1;
                result.text.push_str(replacement);
            }
            None => result.text.push(character),
        }
    }

    result
}

impl CleanCounts {
    fn add(&mut self, kind: RemovalKind) {
        match kind {
            RemovalKind::ExplicitInvisible => self.explicit_invisible += 1,
            RemovalKind::FormatCharacter => self.format_chars += 1,
            RemovalKind::ControlCharacter => self.control_chars += 1,
            RemovalKind::VariationSelector => self.variation_selectors += 1,
        }
    }
}

fn clean_action(character: char, options: &CleanOptions) -> Option<CleanAction> {
    if INVISIBLE_CODEPOINTS.contains(&character) {
        return Some(CleanAction::Remove(RemovalKind::ExplicitInvisible));
    }

    if is_variation_selector(character) {
        return Some(CleanAction::Remove(RemovalKind::VariationSelector));
    }

    if options.replace_em_dashes {
        if let Some(replacement) = em_dash_replacement(character) {
            return Some(CleanAction::ReplaceEmDash(replacement));
        }
    }

    match get_general_category(character) {
        GeneralCategory::Format if options.remove_format_chars => {
            Some(CleanAction::Remove(RemovalKind::FormatCharacter))
        }
        GeneralCategory::Control if options.remove_controls && !is_allowed_control(character) => {
            Some(CleanAction::Remove(RemovalKind::ControlCharacter))
        }
        _ => None,
    }
}

fn em_dash_replacement(character: char) -> Option<&'static str> {
    match character {
        '—' => Some("-"),
        '⸺' => Some("--"),
        '⸻' => Some("---"),
        _ => None,
    }
}

fn is_allowed_control(character: char) -> bool {
    matches!(character, '\n' | '\r' | '\t')
}

fn is_variation_selector(character: char) -> bool {
    matches!(
        character as u32,
        0xFE00..=0xFE0F | 0xE0100..=0xE01EF
    )
}

fn path_with_extension(path: PathBuf, extension: &str) -> PathBuf {
    if path.extension().is_some() {
        path
    } else {
        path.with_extension(extension)
    }
}

fn write_output(path: &Path, text: &str, format: OutputFormat) -> std::io::Result<()> {
    match format {
        OutputFormat::Txt => std::fs::write(path, text),
        OutputFormat::Docx => std::fs::write(path, build_docx(text)?),
    }
}

fn build_docx(text: &str) -> std::io::Result<Vec<u8>> {
    let mut buffer = Cursor::new(Vec::new());
    let mut zip = ZipWriter::new(&mut buffer);
    let options = SimpleFileOptions::default().compression_method(CompressionMethod::Deflated);

    zip.start_file("[Content_Types].xml", options)?;
    zip.write_all(
        br#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types">
  <Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/>
  <Default Extension="xml" ContentType="application/xml"/>
  <Override PartName="/word/document.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.document.main+xml"/>
</Types>"#,
    )?;

    zip.add_directory("_rels/", options)?;
    zip.start_file("_rels/.rels", options)?;
    zip.write_all(
        br#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
  <Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="word/document.xml"/>
</Relationships>"#,
    )?;

    zip.add_directory("word/", options)?;
    zip.start_file("word/document.xml", options)?;
    zip.write_all(build_document_xml(text).as_bytes())?;
    zip.finish()?;

    Ok(buffer.into_inner())
}

fn build_document_xml(text: &str) -> String {
    let normalized = text.replace("\r\n", "\n").replace('\r', "\n");
    let mut xml = String::from(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
  <w:body>"#,
    );

    for line in normalized.split('\n') {
        xml.push_str("\n    <w:p>");
        if !line.is_empty() {
            xml.push_str("<w:r>");
            append_docx_text_run(&mut xml, line);
            xml.push_str("</w:r>");
        }
        xml.push_str("</w:p>");
    }

    xml.push_str(
        r#"
    <w:sectPr>
      <w:pgSz w:w="12240" w:h="15840"/>
      <w:pgMar w:top="1440" w:right="1440" w:bottom="1440" w:left="1440" w:header="720" w:footer="720" w:gutter="0"/>
    </w:sectPr>
  </w:body>
</w:document>"#,
    );

    xml
}

fn append_docx_text_run(xml: &mut String, text: &str) {
    let mut chunk = String::new();

    for character in text.chars() {
        if character == '\t' {
            append_text_chunk(xml, &chunk);
            chunk.clear();
            xml.push_str("<w:tab/>");
        } else {
            chunk.push(character);
        }
    }

    append_text_chunk(xml, &chunk);
}

fn append_text_chunk(xml: &mut String, text: &str) {
    if text.is_empty() {
        return;
    }

    xml.push_str(r#"<w:t xml:space="preserve">"#);
    xml.push_str(&escape_xml(text));
    xml.push_str("</w:t>");
}

fn escape_xml(text: &str) -> String {
    let mut escaped = String::with_capacity(text.len());

    for character in text.chars() {
        match character {
            '&' => escaped.push_str("&amp;"),
            '<' => escaped.push_str("&lt;"),
            '>' => escaped.push_str("&gt;"),
            '"' => escaped.push_str("&quot;"),
            '\'' => escaped.push_str("&apos;"),
            _ => escaped.push(character),
        }
    }

    escaped
}

fn removal_count(input: &str, options: &CleanOptions) -> usize {
    input
        .chars()
        .filter(|character| clean_action(*character, options).is_some())
        .count()
}

fn highlighted_preview_job(input: &str, options: &CleanOptions) -> LayoutJob {
    let normal_format = TextFormat {
        font_id: FontId::monospace(14.0),
        color: TEXT,
        ..Default::default()
    };
    let marker_format = TextFormat {
        font_id: FontId::monospace(13.0),
        color: Color32::WHITE,
        background: DANGER,
        ..Default::default()
    };

    let mut job = LayoutJob::default();

    for character in input.chars() {
        match clean_action(character, options) {
            Some(CleanAction::Remove(_)) => {
                job.append(
                    &format!("[{}]", codepoint_label(character)),
                    0.0,
                    marker_format.clone(),
                );
            }
            Some(CleanAction::ReplaceEmDash(replacement)) => {
                job.append(
                    &format!("[{}->{}]", character, replacement),
                    0.0,
                    marker_format.clone(),
                );
            }
            None => job.append(&character.to_string(), 0.0, normal_format.clone()),
        }
    }

    if input.is_empty() {
        job.append("", 0.0, normal_format);
    }

    job
}

fn codepoint_label(character: char) -> String {
    match character {
        '\u{200B}' => "ZWSP".to_owned(),
        '\u{200C}' => "ZWNJ".to_owned(),
        '\u{200D}' => "ZWJ".to_owned(),
        '\u{FEFF}' => "BOM".to_owned(),
        '\u{00AD}' => "SHY".to_owned(),
        _ => format!("U+{:04X}", character as u32),
    }
}

struct CleanerApp {
    original_text: String,
    cleaned_text: String,
    options: CleanOptions,
    status: String,
    last_cleaned_signature: Option<String>,
    language: Language,
}

impl Default for CleanerApp {
    fn default() -> Self {
        Self {
            original_text: String::new(),
            cleaned_text: String::new(),
            options: CleanOptions::default(),
            status: String::new(),
            last_cleaned_signature: None,
            language: Language::detect(),
        }
    }
}

impl CleanerApp {
    fn clean(&mut self) {
        let result = clean_text(&self.original_text, &self.options);
        let removed = result.counts.total();
        let replaced = result.counts.em_dashes_replaced;
        self.cleaned_text = result.text;
        self.status = clean_status(self.language, removed, replaced);
        self.last_cleaned_signature = Some(self.clean_signature());
    }

    fn clean_if_needed(&mut self) {
        let signature = self.clean_signature();
        if self.last_cleaned_signature.as_deref() != Some(signature.as_str()) {
            self.clean();
        }
    }

    fn clean_signature(&self) -> String {
        format!(
            "{}\u{1F}{}\u{1F}{}\u{1F}{}\u{1F}{}",
            self.original_text,
            self.options.remove_format_chars,
            self.options.remove_controls,
            self.options.normalize_unicode,
            self.options.replace_em_dashes
        )
    }

    fn save_cleaned(&mut self, format: OutputFormat) {
        let (extension, description, default_name) = match format {
            OutputFormat::Txt => ("txt", tr(self.language, TextKey::TextFiles), "cleaned.txt"),
            OutputFormat::Docx => (
                "docx",
                tr(self.language, TextKey::WordDocuments),
                "cleaned.docx",
            ),
        };

        let Some(path) = rfd::FileDialog::new()
            .set_title(tr(self.language, TextKey::SaveDialogTitle))
            .set_file_name(default_name)
            .add_filter(description, &[extension])
            .save_file()
        else {
            return;
        };

        let path = path_with_extension(path, extension);
        match write_output(&path, &self.cleaned_text, format) {
            Ok(()) => self.status = saved_status(self.language, &path),
            Err(error) => self.status = save_failed_status(self.language, &error.to_string()),
        }
    }

    fn copy_and_open_gptzero(&mut self, ctx: &egui::Context) {
        ctx.copy_text(self.cleaned_text.clone());

        match webbrowser::open("https://gptzero.me/") {
            Ok(()) => {
                self.status = tr(self.language, TextKey::GptzeroOpened).to_owned();
            }
            Err(error) => {
                self.status = gptzero_open_failed_status(self.language, &error.to_string());
            }
        }
    }
}

fn clean_status(language: Language, removed: usize, replaced: usize) -> String {
    match language {
        Language::English if replaced == 0 => format!("Removed {removed} character(s)."),
        Language::English => {
            format!("Removed {removed} character(s). Replaced {replaced} dash(es).")
        }
        Language::German if replaced == 0 => format!("{removed} Zeichen entfernt."),
        Language::German => format!("{removed} Zeichen entfernt. {replaced} Strich(e) ersetzt."),
    }
}

fn saved_status(language: Language, path: &Path) -> String {
    match language {
        Language::English => format!("Saved cleaned text to {}.", path.display()),
        Language::German => format!("Bereinigter Text gespeichert unter {}.", path.display()),
    }
}

fn save_failed_status(language: Language, error: &str) -> String {
    match language {
        Language::English => format!("Save failed: {error}"),
        Language::German => format!("Speichern fehlgeschlagen: {error}"),
    }
}

fn gptzero_open_failed_status(language: Language, error: &str) -> String {
    match language {
        Language::English => {
            format!("Cleaned text copied, but GPTZero could not be opened: {error}")
        }
        Language::German => {
            format!("Bereinigter Text kopiert, aber GPTZero konnte nicht geoeffnet werden: {error}")
        }
    }
}

impl eframe::App for CleanerApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        egui::Frame::default()
            .fill(APP_BG)
            .inner_margin(Margin::same(18))
            .show(ui, |ui| {
                self.header(ui);
                ui.add_space(14.0);
                self.toolbar(ui);
                ui.add_space(14.0);
                egui::ScrollArea::vertical()
                    .auto_shrink([false, false])
                    .show(ui, |ui| self.editor_grid(ui));
                ui.add_space(12.0);
                self.status_bar(ui);
            });
    }
}

impl CleanerApp {
    fn header(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                ui.label(
                    RichText::new(tr(self.language, TextKey::AppTitle))
                        .size(26.0)
                        .strong()
                        .color(TEXT),
                );
                ui.label(
                    RichText::new(tr(self.language, TextKey::Subtitle))
                        .size(13.0)
                        .color(MUTED_TEXT),
                );
            });

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                count_badge(
                    ui,
                    removal_count(&self.original_text, &self.options),
                    self.language,
                );
            });
        });
    }

    fn toolbar(&mut self, ui: &mut egui::Ui) {
        panel_frame(SURFACE).show(ui, |ui| {
            ui.horizontal_wrapped(|ui| {
                if primary_button(ui, tr(self.language, TextKey::Clean)).clicked() {
                    self.clean();
                }

                if secondary_button(ui, tr(self.language, TextKey::CopyCleaned)).clicked() {
                    ui.ctx().copy_text(self.cleaned_text.clone());
                    self.status = tr(self.language, TextKey::Copied).to_owned();
                }

                if secondary_button(ui, tr(self.language, TextKey::CopyOpenGptzero)).clicked() {
                    self.copy_and_open_gptzero(ui.ctx());
                }

                ui.separator();

                if secondary_button(ui, tr(self.language, TextKey::SaveTxt)).clicked() {
                    self.save_cleaned(OutputFormat::Txt);
                }

                if secondary_button(ui, tr(self.language, TextKey::SaveDocx)).clicked() {
                    self.save_cleaned(OutputFormat::Docx);
                }

                if quiet_button(ui, tr(self.language, TextKey::Clear)).clicked() {
                    self.original_text.clear();
                    self.cleaned_text.clear();
                    self.status = tr(self.language, TextKey::Cleared).to_owned();
                    self.last_cleaned_signature = Some(self.clean_signature());
                }
            });

            ui.add_space(8.0);
            ui.horizontal_wrapped(|ui| {
                let mut options_changed = false;
                options_changed |= ui
                    .checkbox(
                        &mut self.options.remove_format_chars,
                        tr(self.language, TextKey::FormatChars),
                    )
                    .changed();
                options_changed |= ui
                    .checkbox(
                        &mut self.options.remove_controls,
                        tr(self.language, TextKey::Controls),
                    )
                    .changed();
                options_changed |= ui
                    .checkbox(
                        &mut self.options.normalize_unicode,
                        tr(self.language, TextKey::NormalizeUnicode),
                    )
                    .changed();
                options_changed |= ui
                    .checkbox(
                        &mut self.options.replace_em_dashes,
                        tr(self.language, TextKey::ReplaceEmDashes),
                    )
                    .changed();

                if options_changed {
                    self.clean_if_needed();
                }

                ui.separator();
                ui.label(RichText::new(tr(self.language, TextKey::Language)).color(MUTED_TEXT));
                egui::ComboBox::from_id_salt("language_selector")
                    .selected_text(self.language.label())
                    .show_ui(ui, |ui| {
                        ui.selectable_value(
                            &mut self.language,
                            Language::English,
                            Language::English.label(),
                        );
                        ui.selectable_value(
                            &mut self.language,
                            Language::German,
                            Language::German.label(),
                        );
                    });
            });
        });
    }

    fn editor_grid(&mut self, ui: &mut egui::Ui) {
        let wide_layout = ui.available_width() >= 1180.0;
        let panel_height = if wide_layout { 430.0 } else { 320.0 };

        if wide_layout {
            ui.columns(3, |columns| {
                self.original_panel(&mut columns[0], panel_height);
                self.marked_panel(&mut columns[1], panel_height);
                self.cleaned_panel(&mut columns[2], panel_height);
            });
        } else {
            ui.columns(2, |columns| {
                self.original_panel(&mut columns[0], panel_height);
                self.cleaned_panel(&mut columns[1], panel_height);
            });
            ui.add_space(12.0);
            self.marked_panel(ui, 220.0);
        }
    }

    fn original_panel(&mut self, ui: &mut egui::Ui, height: f32) {
        panel_frame(SURFACE).show(ui, |ui| {
            panel_title(
                ui,
                tr(self.language, TextKey::OriginalText),
                tr(self.language, TextKey::PasteSource),
            );
            ui.add_space(8.0);
            let response = ui.add_sized(
                Vec2::new(ui.available_width(), height),
                egui::TextEdit::multiline(&mut self.original_text).desired_width(f32::INFINITY),
            );
            if response.changed() {
                self.clean_if_needed();
            }
        });
    }

    fn marked_panel(&self, ui: &mut egui::Ui, height: f32) {
        panel_frame(SURFACE).show(ui, |ui| {
            panel_title(
                ui,
                tr(self.language, TextKey::MarkedOriginal),
                &format!(
                    "{} {}",
                    removal_count(&self.original_text, &self.options),
                    tr(self.language, TextKey::ChangeableCharacters)
                ),
            );
            ui.add_space(8.0);
            egui::Frame::default()
                .fill(SURFACE_ALT)
                .stroke(Stroke::new(1.0, BORDER))
                .corner_radius(6)
                .inner_margin(Margin::same(10))
                .show(ui, |ui| {
                    egui::ScrollArea::vertical()
                        .max_height(height)
                        .show(ui, |ui| {
                            ui.label(highlighted_preview_job(&self.original_text, &self.options));
                        });
                });
        });
    }

    fn cleaned_panel(&mut self, ui: &mut egui::Ui, height: f32) {
        panel_frame(SURFACE).show(ui, |ui| {
            panel_title(
                ui,
                tr(self.language, TextKey::CleanedText),
                tr(self.language, TextKey::ReadyToExport),
            );
            ui.add_space(8.0);
            ui.add_sized(
                Vec2::new(ui.available_width(), height),
                egui::TextEdit::multiline(&mut self.cleaned_text).desired_width(f32::INFINITY),
            );
        });
    }

    fn status_bar(&self, ui: &mut egui::Ui) {
        panel_frame(SURFACE_ALT).show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new(tr(self.language, TextKey::Status))
                        .strong()
                        .color(TEXT),
                );
                ui.separator();
                ui.label(
                    RichText::new(if self.status.is_empty() {
                        tr(self.language, TextKey::InitialStatus)
                    } else {
                        &self.status
                    })
                    .color(MUTED_TEXT),
                );
            });
        });
    }
}

fn panel_frame(fill: Color32) -> egui::Frame {
    egui::Frame::default()
        .fill(fill)
        .stroke(Stroke::new(1.0, BORDER))
        .corner_radius(8)
        .inner_margin(Margin::same(14))
}

fn panel_title(ui: &mut egui::Ui, title: &str, detail: &str) {
    ui.horizontal(|ui| {
        ui.label(RichText::new(title).size(16.0).strong().color(TEXT));
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.label(RichText::new(detail).size(12.0).color(MUTED_TEXT));
        });
    });
}

fn primary_button(ui: &mut egui::Ui, label: &str) -> egui::Response {
    ui.add_sized(
        Vec2::new(button_width(label, 96.0), 34.0),
        egui::Button::new(RichText::new(label).strong().color(Color32::WHITE))
            .fill(PRIMARY)
            .stroke(Stroke::new(1.0, PRIMARY_HOVER))
            .corner_radius(6),
    )
}

fn secondary_button(ui: &mut egui::Ui, label: &str) -> egui::Response {
    ui.add_sized(
        Vec2::new(button_width(label, 132.0), 34.0),
        egui::Button::new(RichText::new(label).color(TEXT))
            .fill(SURFACE_ALT)
            .stroke(Stroke::new(1.0, BORDER))
            .corner_radius(6),
    )
}

fn quiet_button(ui: &mut egui::Ui, label: &str) -> egui::Response {
    ui.add_sized(
        Vec2::new(button_width(label, 76.0), 34.0),
        egui::Button::new(RichText::new(label).color(MUTED_TEXT))
            .fill(SURFACE)
            .stroke(Stroke::new(1.0, BORDER))
            .corner_radius(6),
    )
}

fn button_width(label: &str, minimum: f32) -> f32 {
    (label.chars().count() as f32 * 8.0 + 34.0).clamp(minimum, 230.0)
}

fn count_badge(ui: &mut egui::Ui, count: usize, language: Language) {
    egui::Frame::default()
        .fill(if count == 0 {
            Color32::from_rgb(232, 246, 239)
        } else {
            Color32::from_rgb(253, 237, 237)
        })
        .stroke(Stroke::new(
            1.0,
            if count == 0 {
                Color32::from_rgb(177, 222, 201)
            } else {
                Color32::from_rgb(238, 183, 183)
            },
        ))
        .corner_radius(6)
        .inner_margin(Margin::symmetric(10, 6))
        .show(ui, |ui| {
            ui.label(
                RichText::new(format!("{count} {}", tr(language, TextKey::Marked)))
                    .strong()
                    .color(if count == 0 { PRIMARY } else { DANGER }),
            );
        });
}

fn install_style(ctx: &egui::Context) {
    let mut style = (*ctx.global_style()).clone();
    style.visuals = egui::Visuals::light();
    style.visuals.panel_fill = APP_BG;
    style.visuals.window_fill = SURFACE;
    style.visuals.extreme_bg_color = Color32::from_rgb(238, 241, 245);
    style.visuals.widgets.noninteractive.fg_stroke = Stroke::new(1.0, TEXT);
    style.visuals.widgets.inactive.fg_stroke = Stroke::new(1.0, TEXT);
    style.visuals.widgets.hovered.bg_fill = Color32::from_rgb(241, 246, 244);
    style.visuals.widgets.active.bg_fill = Color32::from_rgb(225, 238, 233);
    style.spacing.item_spacing = Vec2::new(10.0, 8.0);
    style.spacing.button_padding = Vec2::new(12.0, 7.0);
    style.spacing.window_margin = Margin::same(12);
    ctx.set_global_style(style);
}

pub fn run() {
    let language = Language::detect();
    let title = tr(language, TextKey::AppTitle);
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title(title)
            .with_inner_size([1100.0, 760.0])
            .with_min_inner_size([780.0, 520.0]),
        ..Default::default()
    };

    if let Err(error) = eframe::run_native(
        title,
        options,
        Box::new(|creation_context| {
            install_style(&creation_context.egui_ctx);
            Ok(Box::<CleanerApp>::default())
        }),
    ) {
        eprintln!("Failed to start app: {error}");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn removes_common_invisible_characters() {
        let result = clean_text("a\u{200B}b\u{FEFF}c\u{202E}d", &CleanOptions::default());

        assert_eq!(result.text, "abcd");
        assert_eq!(result.counts.explicit_invisible, 3);
        assert_eq!(result.counts.total(), 3);
    }

    #[test]
    fn preserves_tabs_and_newlines() {
        let result = clean_text("a\tb\nc\rd", &CleanOptions::default());

        assert_eq!(result.text, "a\tb\nc\rd");
        assert_eq!(result.counts.total(), 0);
    }

    #[test]
    fn removes_disallowed_control_characters() {
        let result = clean_text("a\u{0000}b\u{001F}c", &CleanOptions::default());

        assert_eq!(result.text, "abc");
        assert_eq!(result.counts.control_chars, 2);
    }

    #[test]
    fn removes_variation_selectors() {
        let result = clean_text("a\u{FE0F}b\u{E0100}c", &CleanOptions::default());

        assert_eq!(result.text, "abc");
        assert_eq!(result.counts.variation_selectors, 2);
    }

    #[test]
    fn escapes_word_xml_text() {
        let xml = build_document_xml("A & B < C\tD");

        assert!(xml.contains("A &amp; B &lt; C"));
        assert!(xml.contains("<w:tab/>"));
    }

    #[test]
    fn builds_docx_package() {
        let docx = build_docx("Hello\nworld").expect("docx should be built");

        assert!(docx.starts_with(b"PK"));
        assert!(docx.len() > 500);
    }

    #[test]
    fn counts_preview_removals() {
        let options = CleanOptions::default();

        assert_eq!(removal_count("a\u{200B}b\u{FE0F}c", &options), 2);
        assert!(codepoint_label('\u{200B}').contains("ZWSP"));
    }

    #[test]
    fn replaces_em_dashes_when_enabled() {
        let options = CleanOptions {
            replace_em_dashes: true,
            ..CleanOptions::default()
        };
        let result = clean_text("a—b ⸺ c ⸻ d", &options);

        assert_eq!(result.text, "a-b -- c --- d");
        assert_eq!(result.counts.em_dashes_replaced, 3);
        assert_eq!(result.counts.total(), 0);
        assert_eq!(result.counts.affected_total(), 3);
    }

    #[test]
    fn keeps_em_dashes_when_disabled() {
        let result = clean_text("a—b", &CleanOptions::default());

        assert_eq!(result.text, "a—b");
        assert_eq!(result.counts.em_dashes_replaced, 0);
    }

    #[test]
    fn detects_supported_languages() {
        assert_eq!(Language::from_locale("de-DE"), Language::German);
        assert_eq!(Language::from_locale("de_AT.UTF-8"), Language::German);
        assert_eq!(Language::from_locale("fr-FR"), Language::English);
    }
}
