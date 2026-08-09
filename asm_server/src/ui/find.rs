#[derive(Clone, Debug, Default)]
pub struct FindState {
    pub open: bool,
    pub query: String,
    pub case_sensitive: bool,
    pub matches: Vec<FindMatch>,
    pub current: usize,
    indexed_query: String,
    indexed_case_sensitive: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FindMatch {
    pub line: usize,
    pub start_byte: usize,
    pub end_byte: usize,
}

impl FindState {
    pub fn update_matches<I, S>(&mut self, lines: I) -> bool
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        if self.query == self.indexed_query
            && self.case_sensitive == self.indexed_case_sensitive
        {
            return false;
        }

        self.indexed_query.clone_from(&self.query);
        self.indexed_case_sensitive = self.case_sensitive;
        self.matches.clear();
        self.current = 0;

        if !self.query.is_empty() {
            for (line, text) in lines.into_iter().enumerate() {
                self.matches.extend(
                    find_line_matches(text.as_ref(), &self.query, self.case_sensitive)
                        .into_iter()
                        .map(|(start_byte, end_byte)| FindMatch {
                            line,
                            start_byte,
                            end_byte,
                        }),
                );
            }
        }
        true
    }

    pub fn next(&mut self) -> Option<usize> {
        if self.matches.is_empty() {
            return None;
        }
        self.current = (self.current + 1) % self.matches.len();
        self.current_match().map(|matched| matched.line)
    }

    pub fn previous(&mut self) -> Option<usize> {
        if self.matches.is_empty() {
            return None;
        }
        self.current = (self.current + self.matches.len() - 1) % self.matches.len();
        self.current_match().map(|matched| matched.line)
    }

    pub fn current_match(&self) -> Option<&FindMatch> {
        self.matches.get(self.current)
    }

}

fn find_line_matches(
    line: &str, query: &str, case_sensitive: bool,
) -> Vec<(usize, usize)> {
    if case_sensitive {
        return line
            .match_indices(query)
            .map(|(start, matched)| (start, start + matched.len()))
            .collect();
    }

    // Keep a mapping from the folded string back to byte ranges in the original
    // line. This preserves valid UTF-8 boundaries even when lowercasing expands
    // a character into more than one code point.
    let mut folded = String::new();
    let mut folded_to_original = Vec::new();
    for (start, ch) in line.char_indices() {
        let end = start + ch.len_utf8();
        for folded_char in ch.to_lowercase() {
            folded.push(folded_char);
            folded_to_original.push((start, end));
        }
    }
    let folded_query = query.to_lowercase();

    folded
        .match_indices(&folded_query)
        .filter_map(|(start, matched)| {
            let start_char = folded[..start].chars().count();
            let end_char = folded[..start + matched.len()].chars().count();
            let original_start = folded_to_original.get(start_char)?.0;
            let original_end = folded_to_original.get(end_char.checked_sub(1)?)?.1;
            Some((original_start, original_end))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::FindState;

    fn lines() -> Vec<String> {
        vec![
            "invoke-virtual {v0}, Lfoo/Bar;->run()V".into(),
            "const-string v0, \"Hello\"".into(),
            "hello again".into(),
        ]
    }

    #[test]
    fn finds_all_matches_and_navigates_with_wraparound() {
        let mut state = FindState { query: "hello".into(), ..Default::default() };
        state.update_matches(&lines());

        assert_eq!(state.matches.len(), 2);
        assert_eq!(state.matches[0].line, 1);
        assert_eq!(state.matches[1].line, 2);

        state.next();
        assert_eq!(state.current, 1);
        state.next();
        assert_eq!(state.current, 0);
        state.previous();
        assert_eq!(state.current, 1);
    }

    #[test]
    fn case_sensitive_search_can_be_enabled() {
        let mut state = FindState {
            query: "Hello".into(),
            case_sensitive: true,
            ..Default::default()
        };
        state.update_matches(&lines());

        assert_eq!(state.matches.len(), 1);
        assert_eq!(state.matches[0].line, 1);
    }

    #[test]
    fn match_ranges_are_byte_ranges() {
        let mut state = FindState { query: "世界".into(), ..Default::default() };
        state.update_matches(&["你好，世界".to_string()]);

        let matched = &state.matches[0];
        assert_eq!(&"你好，世界"[matched.start_byte..matched.end_byte], "世界");
    }
}
