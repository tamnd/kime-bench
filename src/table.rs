//! The committed data files: tab separated, `#` comments, one header line.
//!
//! Plain TSV rather than a format that needs a parser crate, because the whole point of these
//! files is that someone who does not trust the harness can read them with `cut`.

use std::path::Path;

/// A parsed TSV file. Every row has exactly as many fields as the header.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Table {
    pub header: Vec<String>,
    pub rows: Vec<Vec<String>>,
}

impl Table {
    /// Parses TSV text, skipping blank lines and lines that start with `#`.
    pub fn parse(text: &str) -> Result<Self, String> {
        let mut lines =
            text.lines().enumerate().filter(|(_, l)| !l.trim().is_empty() && !l.starts_with('#'));
        let Some((_, head)) = lines.next() else {
            return Err("no header line".into());
        };
        let header: Vec<String> = head.split('\t').map(str::to_string).collect();
        let mut rows = Vec::new();
        for (i, line) in lines {
            let row: Vec<String> = line.split('\t').map(str::to_string).collect();
            if row.len() != header.len() {
                return Err(format!(
                    "line {}: {} fields where the header has {}",
                    i + 1,
                    row.len(),
                    header.len()
                ));
            }
            rows.push(row);
        }
        Ok(Self { header, rows })
    }

    /// Reads and parses a file.
    pub fn load(path: &Path) -> Result<Self, String> {
        let text = std::fs::read_to_string(path)
            .map_err(|e| format!("could not read {}: {e}", path.display()))?;
        Self::parse(&text).map_err(|e| format!("{}: {e}", path.display()))
    }

    /// The index of a column by name.
    pub fn column(&self, name: &str) -> Option<usize> {
        self.header.iter().position(|h| h == name)
    }

    /// The values of one column, in row order.
    pub fn values<'a>(&'a self, name: &str) -> Option<impl Iterator<Item = &'a str> + 'a> {
        let c = self.column(name)?;
        Some(self.rows.iter().map(move |r| r[c].as_str()))
    }

    /// Renders the table as Markdown, which is what a report embeds.
    pub fn markdown(&self) -> String {
        let mut out = String::new();
        out.push_str(&format!("| {} |\n", self.header.join(" | ")));
        out.push_str(&format!("|{}\n", "---|".repeat(self.header.len())));
        for row in &self.rows {
            out.push_str(&format!("| {} |\n", row.join(" | ")));
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn comments_and_blank_lines_are_skipped() {
        let t = Table::parse("# note\n\na\tb\n1\t2\n# again\n3\t4\n").unwrap();
        assert_eq!(t.header, ["a", "b"]);
        assert_eq!(t.rows.len(), 2);
        assert_eq!(t.values("b").unwrap().collect::<Vec<_>>(), ["2", "4"]);
    }

    #[test]
    fn a_short_row_is_an_error_with_its_line() {
        let e = Table::parse("a\tb\n1\n").unwrap_err();
        assert!(e.contains("line 2"), "{e}");
    }

    #[test]
    fn markdown_has_a_separator_per_column() {
        let t = Table::parse("a\tb\n1\t2\n").unwrap();
        assert_eq!(t.markdown(), "| a | b |\n|---|---|\n| 1 | 2 |\n");
    }
}
