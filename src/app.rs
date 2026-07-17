use ratatui::widgets::TableState;

use crate::data::GpgKeyRecord;
use crate::i18n::{Lang, Translations};

#[derive(PartialEq, Eq, Clone, Copy)]
pub enum SortColumn {
    Package,
    KeyType,
    Uid,
    Expires,
    Status,
}

#[derive(PartialEq, Eq, Clone, Copy)]
pub enum InputMode {
    Normal,
    Filtering,
}

pub struct App {
    /// Full, unfiltered dataset. Never reordered so indices stay stable.
    pub all_records: Vec<GpgKeyRecord>,
    /// Indices into `all_records` that currently pass the filter, in
    /// display order (i.e. already sorted).
    pub filtered_indices: Vec<usize>,
    pub table_state: TableState,
    pub input_mode: InputMode,
    pub filter_text: String,
    pub sort_column: SortColumn,
    pub sort_ascending: bool,
    pub show_details: bool,
    pub show_help: bool,
    pub should_quit: bool,
    pub lang: Lang,
    pub i18n: Translations,
}

impl App {
    pub fn new(records: Vec<GpgKeyRecord>, lang: Lang) -> Self {
        let filtered_indices: Vec<usize> = (0..records.len()).collect();
        let mut table_state = TableState::default();
        if !filtered_indices.is_empty() {
            table_state.select(Some(0));
        }

        let mut app = Self {
            all_records: records,
            filtered_indices,
            table_state,
            input_mode: InputMode::Normal,
            filter_text: String::new(),
            sort_column: SortColumn::Package,
            sort_ascending: true,
            show_details: false,
            show_help: false,
            should_quit: false,
            i18n: Translations::for_lang(lang),
            lang,
        };
        app.sort_filtered();
        app
    }

    /// Cycles the UI language (EN -> DE -> FR -> EN) and swaps the active
    /// translation table. Data fields (names, UIDs, dates) are untouched.
    pub fn cycle_lang(&mut self) {
        self.lang = self.lang.cycle();
        self.i18n = Translations::for_lang(self.lang);
    }

    /// Toggles the key-details popup, closing the help popup if it was open
    /// so only one overlay is ever shown at a time.
    pub fn toggle_details(&mut self) {
        self.show_details = !self.show_details;
        if self.show_details {
            self.show_help = false;
        }
    }

    /// Toggles the keybindings help popup, closing the details popup if it
    /// was open so only one overlay is ever shown at a time.
    pub fn toggle_help(&mut self) {
        self.show_help = !self.show_help;
        if self.show_help {
            self.show_details = false;
        }
    }

    /// Recomputes `filtered_indices` from `filter_text` against package
    /// name, owner UID, fingerprint and key type (case-insensitive), then
    /// re-sorts and resets selection.
    pub fn apply_filter(&mut self) {
        let query = self.filter_text.to_lowercase();
        self.filtered_indices = self
            .all_records
            .iter()
            .enumerate()
            .filter(|(_, r)| {
                query.is_empty()
                    || r.package_name.to_lowercase().contains(&query)
                    || r.uid.to_lowercase().contains(&query)
                    || r.fingerprint.to_lowercase().contains(&query)
                    || r.key_type.to_lowercase().contains(&query)
            })
            .map(|(i, _)| i)
            .collect();

        self.sort_filtered();

        if self.filtered_indices.is_empty() {
            self.table_state.select(None);
        } else {
            self.table_state.select(Some(0));
        }
    }

    pub fn sort_filtered(&mut self) {
        let records = &self.all_records;
        let asc = self.sort_ascending;
        let col = self.sort_column;

        self.filtered_indices.sort_by(|&a, &b| {
            let ra = &records[a];
            let rb = &records[b];
            let ord = match col {
                SortColumn::Package => ra.package_name.cmp(&rb.package_name),
                SortColumn::KeyType => ra.key_type.cmp(&rb.key_type),
                SortColumn::Uid => ra.uid.cmp(&rb.uid),
                SortColumn::Expires => ra.expires.cmp(&rb.expires),
                SortColumn::Status => ra.status().cmp(&rb.status()),
            };
            if asc {
                ord
            } else {
                ord.reverse()
            }
        });
    }

    pub fn cycle_sort_column(&mut self) {
        self.sort_column = match self.sort_column {
            SortColumn::Package => SortColumn::KeyType,
            SortColumn::KeyType => SortColumn::Uid,
            SortColumn::Uid => SortColumn::Expires,
            SortColumn::Expires => SortColumn::Status,
            SortColumn::Status => SortColumn::Package,
        };
        self.sort_filtered();
    }

    pub fn toggle_sort_direction(&mut self) {
        self.sort_ascending = !self.sort_ascending;
        self.sort_filtered();
    }

    pub fn sort_column_label(&self) -> &str {
        match self.sort_column {
            SortColumn::Package => self.i18n.col_package,
            SortColumn::KeyType => self.i18n.col_key_type,
            SortColumn::Uid => self.i18n.col_owner,
            SortColumn::Expires => self.i18n.col_expires,
            SortColumn::Status => self.i18n.col_status,
        }
    }

    pub fn selected_record(&self) -> Option<&GpgKeyRecord> {
        self.table_state
            .selected()
            .and_then(|i| self.filtered_indices.get(i))
            .and_then(|&idx| self.all_records.get(idx))
    }

    pub fn next(&mut self) {
        if self.filtered_indices.is_empty() {
            return;
        }
        let i = match self.table_state.selected() {
            Some(i) if i + 1 < self.filtered_indices.len() => i + 1,
            _ => 0,
        };
        self.table_state.select(Some(i));
    }

    pub fn previous(&mut self) {
        if self.filtered_indices.is_empty() {
            return;
        }
        let i = match self.table_state.selected() {
            Some(0) | None => self.filtered_indices.len() - 1,
            Some(i) => i - 1,
        };
        self.table_state.select(Some(i));
    }

    pub fn page_down(&mut self, page: usize) {
        if self.filtered_indices.is_empty() {
            return;
        }
        let len = self.filtered_indices.len();
        let i = self.table_state.selected().unwrap_or(0);
        self.table_state.select(Some((i + page).min(len - 1)));
    }

    pub fn page_up(&mut self, page: usize) {
        if self.filtered_indices.is_empty() {
            return;
        }
        let i = self.table_state.selected().unwrap_or(0);
        self.table_state.select(Some(i.saturating_sub(page)));
    }

    pub fn go_top(&mut self) {
        if !self.filtered_indices.is_empty() {
            self.table_state.select(Some(0));
        }
    }

    pub fn go_bottom(&mut self) {
        if !self.filtered_indices.is_empty() {
            self.table_state.select(Some(self.filtered_indices.len() - 1));
        }
    }

    /// Returns (total, expired, invalid-but-not-expired) counts over the
    /// full dataset, regardless of the current filter.
    pub fn counts(&self) -> (usize, usize, usize) {
        let mut expired = 0;
        let mut invalid = 0;
        for r in &self.all_records {
            if r.expired {
                expired += 1;
            } else if !r.valid {
                invalid += 1;
            }
        }
        (self.all_records.len(), expired, invalid)
    }
}
