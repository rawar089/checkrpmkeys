/// Supported UI languages. Add a new variant + a `Translations` arm in
/// `for_lang` below to support another language.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Lang {
    En,
    De,
    Fr,
}

impl Lang {
    /// Cycles to the next language (used by the `l` keybinding).
    pub fn cycle(self) -> Lang {
        match self {
            Lang::En => Lang::De,
            Lang::De => Lang::Fr,
            Lang::Fr => Lang::En,
        }
    }

    /// Short badge shown in the title bar (EN / DE / FR).
    pub fn code(self) -> &'static str {
        match self {
            Lang::En => "EN",
            Lang::De => "DE",
            Lang::Fr => "FR",
        }
    }

    /// Best-effort detection from the environment (`LC_ALL`, `LC_MESSAGES`,
    /// `LANG`, `LANGUAGE`), falling back to English. No extra crate needed:
    /// these are the standard POSIX locale variables (e.g. `de_DE.UTF-8`,
    /// `fr_FR.UTF-8`).
    pub fn detect_from_env() -> Lang {
        for var in ["LC_ALL", "LC_MESSAGES", "LANG", "LANGUAGE"] {
            if let Ok(val) = std::env::var(var) {
                let lower = val.to_lowercase();
                if lower.starts_with("de") {
                    return Lang::De;
                }
                if lower.starts_with("fr") {
                    return Lang::Fr;
                }
                if lower.starts_with("en") {
                    return Lang::En;
                }
            }
        }
        Lang::En
    }
}

/// All user-facing UI strings (chrome, not data). Record fields themselves
/// (package names, UIDs, dates) are real-world data and intentionally never
/// translated.
pub struct Translations {
    pub app_title: &'static str,
    pub valid: &'static str,
    pub expired: &'static str,
    pub invalid: &'static str,
    pub total: &'static str,

    pub col_package: &'static str,
    pub col_key_type: &'static str,
    pub col_owner: &'static str,
    pub col_expires: &'static str,
    pub col_fingerprint: &'static str,
    pub col_status: &'static str,

    pub status_valid: &'static str,
    pub status_expired: &'static str,
    pub status_invalid: &'static str,
    pub status_legacy: &'static str,

    pub keys_title: &'static str,
    pub sorted_by: &'static str,
    pub filter_label: &'static str,
    pub active_filter: &'static str,
    pub clear_filter_hint: &'static str,

    pub help_normal: &'static str,
    pub help_filtering: &'static str,

    pub details_title: &'static str,
    pub detail_package: &'static str,
    pub detail_key_type: &'static str,
    pub detail_owner: &'static str,
    pub detail_expires: &'static str,
    pub detail_status: &'static str,
    pub detail_fingerprint: &'static str,

    pub help_title: &'static str,
    pub kb_move: &'static str,
    pub kb_top_bottom: &'static str,
    pub kb_page: &'static str,
    pub kb_filter: &'static str,
    pub kb_sort_column: &'static str,
    pub kb_reverse_sort: &'static str,
    pub kb_details: &'static str,
    pub kb_clear_filter: &'static str,
    pub kb_language: &'static str,
    pub kb_help: &'static str,
    pub kb_close: &'static str,
    pub kb_quit: &'static str,
    pub filter_mode_note: &'static str,
}

impl Translations {
    pub fn for_lang(lang: Lang) -> Self {
        match lang {
            Lang::En => Self {
                app_title: "GPG Key Inspector",
                valid: "valid",
                expired: "expired",
                invalid: "invalid",
                total: "total",

                col_package: "Package",
                col_key_type: "Type",
                col_owner: "Owner (UID)",
                col_expires: "Expires",
                col_fingerprint: "Fingerprint",
                col_status: "Status",

                status_valid: "Valid",
                status_expired: "Expired",
                status_invalid: "Invalid",
                status_legacy: "Legacy",

                keys_title: "Keys",
                sorted_by: "Sorted by",
                filter_label: "Filter: ",
                active_filter: "Active filter",
                clear_filter_hint: "c to clear",

                help_normal: "j/k \u{2191}/\u{2193} move \u{00B7} g/G top/bottom \u{00B7} /: filter \u{00B7} s: sort column \u{00B7} r: reverse \u{00B7} Enter: details \u{00B7} c: clear filter \u{00B7} l: language \u{00B7} h: help \u{00B7} q: quit",
                help_filtering: "Type to filter \u{00B7} Enter/Esc: back to navigation",

                details_title: "Key Details (Enter/Esc to close)",
                detail_package: "Package:",
                detail_key_type: "Key Type:",
                detail_owner: "Owner (UID):",
                detail_expires: "Expires:",
                detail_status: "Status:",
                detail_fingerprint: "Fingerprint:",

                help_title: "Keyboard Shortcuts (h/Esc to close)",
                kb_move: "Move selection up / down",
                kb_top_bottom: "Jump to top / bottom",
                kb_page: "Move by 10 rows",
                kb_filter: "Search / filter the list",
                kb_sort_column: "Cycle sort column",
                kb_reverse_sort: "Reverse sort direction",
                kb_details: "Toggle key details",
                kb_clear_filter: "Clear active filter",
                kb_language: "Cycle UI language",
                kb_help: "Toggle this help",
                kb_close: "Close popup / clear filter / quit",
                kb_quit: "Quit",
                filter_mode_note: "While filtering: type to search, Enter/Esc returns to navigation.",
            },
            Lang::De => Self {
                app_title: "GPG-Schl\u{00FC}ssel-Inspektor",
                valid: "g\u{00FC}ltig",
                expired: "abglaufen",
                invalid: "ung\u{00FC}ltig",
                total: "gesamt",

                col_package: "Paket",
                col_key_type: "Typ",
                col_owner: "Eigent\u{00FC}mer (UID)",
                col_expires: "L\u{00E4}uft ab",
                col_fingerprint: "Fingerabdruck",
                col_status: "Status",

                status_valid: "G\u{00FC}ltig",
                status_expired: "Abgelaufen",
                status_invalid: "Ung\u{00FC}ltig",
                status_legacy: "Missbiligt",

                keys_title: "Schl\u{00FC}ssel",
                sorted_by: "Sortiert nach",
                filter_label: "Filter: ",
                active_filter: "Aktiver Filter",
                clear_filter_hint: "c zum L\u{00F6}schen",

                help_normal: "j/k \u{2191}/\u{2193} bewegen \u{00B7} g/G Anfang/Ende \u{00B7} /: filtern \u{00B7} s: Spalte sortieren \u{00B7} r: Reihenfolge umkehren \u{00B7} Enter: Details \u{00B7} c: Filter l\u{00F6}schen \u{00B7} l: Sprache \u{00B7} h: Hilfe \u{00B7} q: Beenden",
                help_filtering: "Tippen zum Filtern \u{00B7} Enter/Esc: zur\u{00FC}ck zur Navigation",

                details_title: "Schl\u{00FC}sseldetails (Enter/Esc zum Schlie\u{00DF}en)",
                detail_package: "Paket:",
                detail_key_type: "Schl\u{00FC}sseltyp:",
                detail_owner: "Eigent\u{00FC}mer (UID):",
                detail_expires: "L\u{00E4}uft ab:",
                detail_status: "Status:",
                detail_fingerprint: "Fingerabdruck:",

                help_title: "Tastenk\u{00FC}rzel (h/Esc zum Schlie\u{00DF}en)",
                kb_move: "Auswahl nach oben / unten bewegen",
                kb_top_bottom: "Zum Anfang / Ende springen",
                kb_page: "Um 10 Zeilen bewegen",
                kb_filter: "Liste durchsuchen / filtern",
                kb_sort_column: "Sortierspalte wechseln",
                kb_reverse_sort: "Sortierreihenfolge umkehren",
                kb_details: "Schl\u{00FC}sseldetails ein-/ausblenden",
                kb_clear_filter: "Aktiven Filter l\u{00F6}schen",
                kb_language: "Sprache wechseln",
                kb_help: "Diese Hilfe ein-/ausblenden",
                kb_close: "Popup schlie\u{00DF}en / Filter l\u{00F6}schen / beenden",
                kb_quit: "Beenden",
                filter_mode_note: "Beim Filtern: Tippen zum Suchen, Enter/Esc kehrt zur Navigation zur\u{00FC}ck.",
            },
            Lang::Fr => Self {
                app_title: "Inspecteur de cl\u{00E9}s GPG",
                valid: "valide",
                expired: "expir\u{00E9}e",
                invalid: "invalide",
                total: "total",

                col_package: "Paquet",
                col_key_type: "Type",
                col_owner: "Propri\u{00E9}taire (UID)",
                col_expires: "Expire le",
                col_fingerprint: "Empreinte",
                col_status: "Statut",

                status_valid: "Valide",
                status_expired: "Expir\u{00E9}e",
                status_invalid: "Invalide",
                status_legacy: "Ver\u{00E9}d\u{00E9}e",

                keys_title: "Cl\u{00E9}s",
                sorted_by: "Tri\u{00E9} par",
                filter_label: "Filtre : ",
                active_filter: "Filtre actif",
                clear_filter_hint: "c pour effacer",

                help_normal: "j/k \u{2191}/\u{2193} d\u{00E9}placer \u{00B7} g/G d\u{00E9}but/fin \u{00B7} /: filtrer \u{00B7} s: colonne de tri \u{00B7} r: inverser \u{00B7} Entr\u{00E9}e: d\u{00E9}tails \u{00B7} c: effacer le filtre \u{00B7} l: langue \u{00B7} h: aide \u{00B7} q: quitter",
                help_filtering: "Tapez pour filtrer \u{00B7} Entr\u{00E9}e/\u{00C9}chap : retour \u{00E0} la navigation",

                details_title: "D\u{00E9}tails de la cl\u{00E9} (Entr\u{00E9}e/\u{00C9}chap pour fermer)",
                detail_package: "Paquet :",
                detail_key_type: "Type de cl\u{00E9} :",
                detail_owner: "Propri\u{00E9}taire (UID) :",
                detail_expires: "Expire le :",
                detail_status: "Statut :",
                detail_fingerprint: "Empreinte :",

                help_title: "Raccourcis clavier (h/\u{00C9}chap pour fermer)",
                kb_move: "D\u{00E9}placer la s\u{00E9}lection haut / bas",
                kb_top_bottom: "Aller au d\u{00E9}but / \u{00E0} la fin",
                kb_page: "D\u{00E9}placer de 10 lignes",
                kb_filter: "Rechercher / filtrer la liste",
                kb_sort_column: "Changer la colonne de tri",
                kb_reverse_sort: "Inverser l'ordre de tri",
                kb_details: "Afficher/masquer les d\u{00E9}tails",
                kb_clear_filter: "Effacer le filtre actif",
                kb_language: "Changer la langue",
                kb_help: "Afficher/masquer cette aide",
                kb_close: "Fermer la fen\u{00EA}tre / effacer le filtre / quitter",
                kb_quit: "Quitter",
                filter_mode_note: "En mode filtre : tapez pour rechercher, Entr\u{00E9}e/\u{00C9}chap revient \u{00E0} la navigation.",
            },
        }
    }
}
