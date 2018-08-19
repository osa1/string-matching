use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::collections::HashSet;
use std::collections::VecDeque; // queue

// TODO: Visualize the graph with graphiz
// TODO: Implement a variant that can search a text with just a immutable borrow (currently we need
// &mut self).
// TODO: Maybe provide a few IntoIter impls.

pub struct AhoCorasick {
    keywords: Vec<String>,
    states: Vec<HashMap<char, usize>>,

    // NOTE For state N fails state is fails[N-1]. Fail for state 0 is not defined.
    fails: Option<Vec<usize>>,

    // outputs[n] = outputs of state n
    outputs: Vec<HashSet<usize>>,
}

impl AhoCorasick {
    pub fn new() -> AhoCorasick {
        AhoCorasick {
            keywords: vec![],
            states: vec![HashMap::new()],
            fails: None,
            outputs: vec![HashSet::new()],
        }
    }

    pub fn add_keyword(&mut self, s: &str) {
        // TODO maybe provide a HashSet<String> API to avoid adding same keyword multiple times
        self.keywords.push(s.to_owned());

        let mut state = 0;
        for c in s.chars() {
            let n_states = self.states.len();
            match self.states.get_mut(state).unwrap().entry(c) {
                Entry::Occupied(entry) => {
                    state = entry.get().clone();
                }
                Entry::Vacant(entry) => {
                    entry.insert(n_states);
                    self.states.push(HashMap::new());
                    self.outputs.push(HashSet::new());
                    state = n_states;
                }
            }
        }

        self.outputs[state].insert(self.keywords.len() - 1);
        self.fails = None; // TODO can we update fails incrementally?
    }

    fn make_fails(&mut self) {
        if self.fails.is_some() {
            return;
        }

        let mut fails = vec![0; self.states.len() - 1];
        // -1 because state 0 doesn't have a fail state
        // (perhaps define f(0) = 0 and simplify this?

        // Breadth-first traversal of the trie
        // Note that we keep indices of states here instead of references to states, to avoid
        // borrowchk issues
        let mut work_list: VecDeque<usize> = VecDeque::new();

        // Start calculating from depth 1
        {
            let init_state = &self.states[0];
            for (_ch, next) in init_state {
                work_list.push_back(*next);
                assert!(*next != 0);
                fails[*next - 1] = 0;
            }
        }

        while !work_list.is_empty() {
            let work = work_list.pop_front().unwrap();
            assert!(work != 0);
            let work_state = &self.states[work];
            for (ch, next) in work_state {
                work_list.push_back(*next);
                // Note that when processing state S we should already have a f(S) defined
                // So fails[S-1] is defined
                let mut fail_state = fails[work - 1];
                while fail_state != 0 && self.states[fail_state].get(ch).is_none() {
                    fail_state = fails[fail_state - 1];
                }

                fails[*next - 1] = self.states[fail_state].get(ch).cloned().unwrap_or(0);

                // Otherwise unsafe below is really unsafe
                assert!(*next != fails[*next - 1]);
                let output_vec: *mut HashSet<usize> =
                    (&mut self.outputs[*next]) as *mut HashSet<usize>;
                for out in &self.outputs[fails[*next - 1]] {
                    unsafe {
                        (*output_vec).insert(*out);
                    }
                }
            }
        }

        self.fails = Some(fails);
    }

    pub fn match_(&mut self, text: &str) -> Vec<(usize, &str)> {
        let mut ret = vec![];

        // Ideally make_fails() would return a reference to the fail vector but that causes
        // borrowchk issues
        self.make_fails();
        let fails = self.fails.as_ref().unwrap();

        let mut state = 0;
        for (ch_idx, ch) in text.chars().enumerate() {
            while state != 0 && self.states[state].get(&ch).is_none() {
                state = fails[state - 1];
            }
            state = self.states[state].get(&ch).cloned().unwrap_or(0);

            for output in &self.outputs[state] {
                let kw = self.keywords[*output].as_str();
                // println!("check_output idx: {}, state: {}, kw: {}", idx, state, kw);
                ret.push((
                    ch_idx - (kw.len() - 1), /* FIXME not correct for unicode */
                    kw,
                ));
            }
        }

        ret
    }
}

pub struct MatchIter<'a, 'b> {
    ac: &'a AhoCorasick,
    state: usize,
    loc: usize,
    chars: ::std::iter::Enumerate<::std::str::Chars<'b>>,

    // Only available when yielding outputs of a state
    output_iter: Option<::std::collections::hash_set::Iter<'a, usize>>,
}

impl<'a, 'b> Iterator for MatchIter<'a, 'b> {
    type Item = (usize, &'a str);

    fn next(&mut self) -> Option<(usize, &'a str)> {
        if let Some(output_iter) = &mut self.output_iter {
            if let Some(output_idx) = output_iter.next() {
                let kw = self.ac.keywords[*output_idx].as_str();
                return Some((
                        self.loc - (kw.len() - 1), // FIXME
                        kw,
                ));
            } else {
                self.output_iter = None;
            }
        }

        match self.chars.next() {
            None => None,
            Some((ch_idx, ch)) => {
                self.loc = ch_idx;
                while self.state != 0 && self.ac.states[self.state].get(&ch).is_none() {
                    // TODO: what if fails was invalidated? is that even possible?
                    // (can I add a word after building an iterator?)
                    self.state = (self.ac.fails.as_ref().unwrap())[self.state - 1];
                }
                self.state = self.ac.states[self.state].get(&ch).cloned().unwrap_or(0);
                self.output_iter = Some(self.ac.outputs[self.state].iter());
                self.next()
            }
        }
    }
}

impl AhoCorasick {
    pub fn match_iter<'a, 'b>(&'a mut self, str: &'b str) -> MatchIter<'a, 'b> {
        self.make_fails();
        MatchIter {
            ac: self,
            state: 0,
            loc: 0,
            chars: str.chars().enumerate(),
            output_iter: None,
        }
    }
}

#[test]
fn test_trie() {
    let mut ac = AhoCorasick::new();
    ac.add_keyword("hers");
    ac.add_keyword("his");
    ac.add_keyword("she");

    // println!("states: {:?}", ac.states);
    // println!("fails: {:?}", ac.fails);
    // println!("outputs: {:?}", ac.outputs);

    assert_eq!(ac.match_("she"), vec![(0, "she")]);

    assert_eq!(ac.match_("    she"), vec![(4, "she")]);

    assert_eq!(ac.match_("  hers "), vec![(2, "hers")]);

    assert_eq!(ac.match_(" his"), vec![(1, "his")]);

    assert_eq!(
        ac.match_(" she hers his "),
        vec![(1, "she"), (5, "hers"), (10, "his")]
    );

    assert_eq!(ac.match_("hershe"), vec![(0, "hers"), (3, "she")]);

    // overlapping matches
    assert_eq!(ac.match_("hishe"), vec![(0, "his"), (2, "she")]);
}

#[test]
fn test_trie_2() {
    let mut ac = AhoCorasick::new();
    ac.add_keyword("fo");
    ac.add_keyword("xfoo");
    ac.add_keyword("bar");
    ac.add_keyword("bax");

    // We start matching "xfoo", but after "xfo" we fail, and fail state has an output.
    assert_eq!(ac.match_("xfobaxbar"), vec![(1, "fo"), (3, "bax"), (6, "bar")]);
}

#[test]
fn test_trie_iterator() {
    let mut ac = AhoCorasick::new();
    ac.add_keyword("hers");
    ac.add_keyword("his");
    ac.add_keyword("she");

    let mut iter = ac.match_iter(" she hers his ");
    assert_eq!(iter.next(), Some((1, "she")));
    assert_eq!(iter.next(), Some((5, "hers")));
    assert_eq!(iter.next(), Some((10, "his")));
    assert_eq!(iter.next(), None);

    // overlapping matches
    let mut iter = ac.match_iter("hishe");
    assert_eq!(iter.next(), Some((0, "his")));
    assert_eq!(iter.next(), Some((2, "she")));
    assert_eq!(iter.next(), None);

    let mut ac = AhoCorasick::new();
    ac.add_keyword("fo");
    ac.add_keyword("xfoo");
    ac.add_keyword("bar");
    ac.add_keyword("bax");

    let mut iter = ac.match_iter("xfobaxbar");
    assert_eq!(iter.next(), Some((1, "fo")));
    assert_eq!(iter.next(), Some((3, "bax")));
    assert_eq!(iter.next(), Some((6, "bar")));
    assert_eq!(iter.next(), None);
}
