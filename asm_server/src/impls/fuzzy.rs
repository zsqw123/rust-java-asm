use java_asm::StrRef;
use nucleo_matcher::pattern::{CaseMatching, Normalization, Pattern};
use nucleo_matcher::{Config, Matcher, Utf32Str};
use std::cmp::{min, Reverse};
use std::sync::Arc;

pub struct FuzzyMatchModel {
    // the query string
    input: StrRef,
    items: Vec<StrRef>,
    // how many results to return
    top_n: usize,
    matcher: Matcher,
    pattern: Pattern,
    // length is the same as input
    inc_infos: IncrementalInfos,
}

type IncrementalInfos = Vec<Option<IncrementalInfo>>;

// if change `a` to `ab`, just search things from (previous matched items + remaining items).
// if change `ab` to `a`, reset incremental info to the specific index.
// in other cases, which means totally different, clear all incremental info.
#[derive(Debug, Clone)]
struct IncrementalInfo {
    // previous last index when search stops
    stop_index: usize,
    // previous matched items
    items: Vec<StrRef>,
}

impl FuzzyMatchModel {
    pub fn new(
        input: StrRef, items: &[StrRef], top_n: usize,
    ) -> Self {
        let config = Config::DEFAULT.match_paths();
        let matcher = Matcher::new(config);
        let pattern = Pattern::parse(&input, CaseMatching::Ignore, Normalization::Never);
        let items = items.to_vec();
        let inc_info = vec![None; items.len() + 1];
        FuzzyMatchModel {
            input,
            items,
            top_n,
            matcher,
            pattern,
            inc_infos: inc_info,
        }
    }

    pub fn search_with_new_input(&mut self, new_input: StrRef) -> Vec<StrRef> {
        let old_input = self.input.clone();
        let old_len = old_input.len();
        let new_len = new_input.len();
        // 1. check if same input
        let mut previous_info: Option<IncrementalInfo> = None;
        if let Some(Some(previous)) = self.inc_infos.get(old_len) {
            previous_info = Some(previous.clone());
        }
        if old_len == new_len && new_input == old_input {
            if let Some(IncrementalInfo { items, .. }) = previous_info {
                // same input, skip search
                return items.clone();
            }
        } else {
            // change pattern
            self.input = new_input.clone();
            self.pattern.reparse(&new_input, CaseMatching::Ignore, Normalization::Never);
        }
        // 2. do search by incremental info
        let result: Vec<StrRef> = previous_info
            .map(|inc_info| {
                // if has inc info
                self.inc_case_1(&old_input, &new_input, &inc_info)
                    .or_else(|| self.inc_case_2(&old_input, &new_input))
                    .unwrap_or_else(|| self.full_search())
            })
            .unwrap_or_else(|| self.full_search());
        result
    }

    // case 1, change `a` to `ab`, just search things from (previous matched items + remaining items).
    // returns `None` means not applicable for this case.
    fn inc_case_1(
        &mut self, old_input: &str, new_input: &str,
        old_inc_info: &IncrementalInfo,
    ) -> Option<Vec<StrRef>> {
        let old_len = old_input.len();
        let new_len = new_input.len();
        if new_len < old_len || !new_input.starts_with(old_input) { return None; };

        let IncrementalInfo { stop_index, items: inc_items } = old_inc_info;
        let search_result = self.search_in_ranges(&inc_items, *stop_index);
        let new_inc_info = IncrementalInfo {
            stop_index: search_result.stop_idx,
            items: search_result.items.clone(),
        };
        self.inc_infos.resize(new_len + 1, None);
        self.inc_infos[new_len] = Some(new_inc_info);
        Some(search_result.items)
    }

    // case 2, if change `ab` to `a`, reset incremental info to the specific index.
    fn inc_case_2(
        &mut self, old_input: &str, new_input: &str,
    ) -> Option<Vec<StrRef>> {
        // 1. find common prefix len
        let mut common_prefix_len = 0usize;
        let old_bytes = old_input.as_bytes();
        let new_bytes = new_input.as_bytes();
        let min_len = min(old_bytes.len(), new_bytes.len());
        for i in 0..min_len {
            if old_bytes[i] == new_bytes[i] {
                common_prefix_len = i + 1;
            } else {
                break;
            }
        }

        // 2. if the inc info in common prefix idx not exists, means not applicable for this case.
        let Some(target_inc_info) = self.inc_infos.get(common_prefix_len)? else {
            return None;
        };

        let result = target_inc_info.items.clone();
        // 3. resize incremental infos to new length to fit new query.
        self.inc_infos.resize(new_input.len() + 1, None);
        Some(result)
    }

    pub fn full_search(&mut self) -> Vec<StrRef> {
        let SearchResult { stop_idx, items } = self.search_in_ranges(&[], 0);
        let new_inc_info = IncrementalInfo {
            stop_index: stop_idx,
            items,
        };
        self.inc_infos.clear();
        self.inc_infos.resize(self.input.len() + 1, None);
        self.inc_infos[self.input.len()] = Some(new_inc_info.clone());
        new_inc_info.items
    }

    fn search_in_ranges(
        &mut self, items_1: &[StrRef], items_2_start_idx: usize,
    ) -> SearchResult {
        let Self { top_n, items: all_items, matcher, pattern, .. } = self;
        let top_n = *top_n;
        let mut buf = Vec::new();
        let mut result: Vec<(&StrRef, u32)> = Vec::with_capacity(top_n);

        // search things in items 1
        for item in items_1 {
            if result.len() >= top_n {
                break;
            }
            let haystack = Utf32Str::new(item.as_ref(), &mut buf);
            let single_result = pattern.score(haystack, matcher)
                .map(|score| (item, score));
            let Some(single_result) = single_result else {
                continue;
            };
            result.push(single_result);
        }

        // search things in items 2 which stored in `all_items` and starts from `items_2_start_idx`.
        let mut stop_idx = items_2_start_idx;
        for (idx, item) in all_items.iter().enumerate().skip(stop_idx) {
            if result.len() >= top_n {
                break;
            }
            stop_idx = idx;
            let haystack = Utf32Str::new(item.as_ref(), &mut buf);
            let single_result = pattern.score(haystack, matcher)
                .map(|score| (item, score));
            let Some(single_result) = single_result else {
                continue;
            };
            result.push(single_result);
        }

        result.sort_by_key(|(_, score)| Reverse(*score));
        let result_items = result.iter().map(|(item, _)| Arc::clone(item)).collect();
        SearchResult {
            stop_idx,
            items: result_items,
        }
    }
}

struct SearchResult {
    stop_idx: usize,
    items: Vec<StrRef>,
}

#[cfg(test)]
mod tests {
    use crate::impls::fuzzy::FuzzyMatchModel;
    use java_asm::{vec_str_ref, StrRef};
    use rand::prelude::SliceRandom;
    use rand::rng;

    #[test]
    fn test_fuzzy_match() {
        // simple input
        let input: StrRef = "abc".into();
        let items: Vec<StrRef> = vec_str_ref![
            "apple/banana",
            "apple/banana/cake",
        ];
        let mut model = FuzzyMatchModel::new(input, &items, 10);
        let expected_result_1 = vec_str_ref![
            "apple/banana/cake"
        ];
        assert_eq!(model.full_search(), expected_result_1);

        // update input
        let real_result = model.search_with_new_input("abn".into());
        let expected_result_2 = vec_str_ref![
            "apple/banana",
            "apple/banana/cake"
        ];
        assert_eq!(real_result, expected_result_2);

        // not exist
        let real_result = model.search_with_new_input("abcd".into());
        let expected_result_3 = vec_str_ref![];
        assert_eq!(real_result, expected_result_3);
    }

    #[test]
    fn test_huge_input() {
        let sample_size = 100_000;
        let input: StrRef = "im2z".into();
        let items: Vec<StrRef> = (0..sample_size).map(|i|
            format!("item/{}/i12k/pc1i1/z", i).into()
        ).collect();
        let start = std::time::Instant::now();
        let mut model = FuzzyMatchModel::new(input, &items, 100_000);
        let result = model.full_search();
        println!("Cost time: {:?}ms for 100K items", start.elapsed().as_millis());
        assert_eq!(result.len(), sample_size);
    }

    #[test]
    fn test_huge_input_take_1000() {
        let sample_size = 50_000;
        let input: StrRef = "im21".into();
        // items 1: always matched
        let items_1 = (0..sample_size).map(|i|
            format!("item/{}/i12k/pc1i1/z", i).into()
        );
        // items 2: not matched
        let items_2 = (0..sample_size).map(|i|
            format!("item/{}", i).into()
        );
        let mut items: Vec<StrRef> = items_1.chain(items_2).collect();
        items.shuffle(&mut rng());

        let start = std::time::Instant::now();
        let mut model = FuzzyMatchModel::new(input, &items, 10000);
        let result = model.full_search();
        println!("Cost time: {:?}ms for take 1000 items", start.elapsed().as_millis());
        assert_eq!(result.len(), 10000);

        let start = std::time::Instant::now();
        let result = model.search_with_new_input("im21z".into());
        println!("Cost time: {:?}ms for take 1000 items next time", start.elapsed().as_millis());
        assert_eq!(result.len(), 10000);
    }
}

