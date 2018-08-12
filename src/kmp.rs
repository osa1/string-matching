#[derive(Debug)]
pub struct KMP<C> {
    pat: Vec<C>,

    // See `mk_pfx`
    pfx: Vec<usize>,
}

// Given size of the current match (should be at least 1) n, pfx[n-1] gives the longest proper
// prefix of the pattern that is also a suffix. Examples:
//
// - Pattern: ababaca
//   Pfx:
//      match size  | longest prefix
//      1 "a"       | 0 ""
//      2 "ab"      | 0 ""
//      3 "aba"     | 1 "a"
//      4 "abab"    | 2 "ab"
//      5 "ababa"   | 3 "aba"
//      6 "ababac"  | 0 ""
//      7 "ababaca" | 1 "a"
//
// - Pattern: "aaba"
//   Pfx:
//      match size  | longest prefix
//      1 "a"       | 0 ""
//      2 "aa"      | 1 "a"
//      3 "aab"     | 0 ""
//      4 "aaba"    | 1 "a"
//
// - Pattern: "aaaa"
//   Pfx:
//      match size  | longest prefix
//      1 "a"       | 0 ""
//      2 "aa"      | 1 "a"
//      3 "aaa"     | 2 "aa"
//      4 "aaaa"    | 3 "aaa"
//
fn mk_pfx<C: Eq>(pat: &[C]) -> Vec<usize> {
    let mut ret = vec![0; pat.len()];

    // Number of characters matched
    let mut n_matched = 0;

    for i in 1..pat.len() {
        let next = &pat[i];

        while n_matched > 0 && pat[n_matched] != *next {
            // Next character does not match, backtrack. Similar to the backtracking code in
            // `match_`.
            n_matched = ret[n_matched - 1];
        }
        if pat[n_matched] == *next {
            // Next character matches
            n_matched += 1;
        }

        ret[i] = n_matched;
    }

    ret
}

#[test]
fn test_mk_pfx() {
    assert_eq!(
        mk_pfx(&("ababaca".chars().collect::<Vec<_>>())),
        vec![0, 0, 1, 2, 3, 0, 1],
    );

    assert_eq!(
        mk_pfx(&("aaba".chars().collect::<Vec<_>>())),
        vec![0, 1, 0, 1],
    );

    assert_eq!(
        mk_pfx(&("aaaa".chars().collect::<Vec<_>>())),
        vec![0, 1, 2, 3],
    );
}

use std::iter::Enumerate;

pub struct MatchIterator<'a, 'b, C: Eq + 'a, I: Iterator<Item = C>> {
    iter: Enumerate<I>,
    n_matched: usize,
    pat: &'a [C],
    pfx: &'b [usize],
}

impl<C: Eq> KMP<C> {
    pub fn new(pat: Vec<C>) -> KMP<C> {
        assert!(!pat.is_empty());
        let pfx = mk_pfx(&pat);
        KMP { pat, pfx }
    }

    pub fn match_<I: Iterator<Item = C>>(&self, t: I) -> Vec<usize> {
        let mut ret = vec![];

        // Number of characters matched
        let mut n_matched = 0;

        for (i, c) in t.enumerate() {
            while n_matched > 0 && self.pat[n_matched] != c {
                // Next character does not match, backtrack so that next n_matched will be the
                // longest prefix of current match that's also a suffix of the current match.
                n_matched = self.pfx[n_matched];
            }
            if self.pat[n_matched] == c {
                // Next character matches
                n_matched += 1;
            }
            if n_matched == self.pat.len() {
                // All of P matched
                ret.push(i + 1 - self.pat.len());
                // Loop for the next match
                n_matched = self.pfx[self.pat.len() - 1];
            }
        }

        ret
    }

    pub fn match_iter<I: Iterator<Item = C>>(&self, t: I) -> MatchIterator<C, I> {
        MatchIterator {
            iter: t.enumerate(),
            n_matched: 0,
            pat: &self.pat,
            pfx: &self.pfx,
        }
    }
}

impl<'a, 'b, C: Eq, I: Iterator<Item = C>> Iterator for MatchIterator<'a, 'b, C, I> {
    type Item = usize;

    fn next(&mut self) -> Option<usize> {
        for (i, c) in &mut self.iter {
            while self.n_matched > 0 && self.pat[self.n_matched] != c {
                // Next character does not match, backtrack so that next n_matched will be the
                // longest prefix of current match that's also a suffix of the current match.
                self.n_matched = self.pfx[self.n_matched];
            }
            if self.pat[self.n_matched] == c {
                // Next character matches
                self.n_matched += 1;
            }
            if self.n_matched == self.pat.len() {
                // Loop for the next match
                self.n_matched = self.pfx[self.pat.len() - 1];
                // All of P matched
                return Some(i + 1 - self.pat.len());
            }
        }

        None
    }
}

#[test]
fn test_kmp() {
    fn kmp(pat: &str, text: &str) -> Vec<usize> {
        KMP::new(pat.chars().collect()).match_(text.chars())
    }

    assert_eq!(kmp("ababaca", "ababaca"), vec![0]);

    assert_eq!(kmp("ab", "ababaca"), vec![0, 2]);

    assert_eq!(kmp("aaba", "aabaacaadaabaaba"), vec![0, 9, 12]);

    assert_eq!(kmp("foo", ""), vec![]);
}

#[test]
fn test_iterator() {
    let kmp = KMP::new("aaba".chars().collect());
    let mut iter = kmp.match_iter("aabaacaadaabaaba".chars());
    assert_eq!(iter.next(), Some(0));
    assert_eq!(iter.next(), Some(9));
    assert_eq!(iter.next(), Some(12));
    assert_eq!(iter.next(), None);
}

#[test]
fn test_generic() {
    let kmp: KMP<usize> = KMP::new(vec![1, 1]);
    let arr: [usize; 4] = [1, 1, 1, 1];
    let mut iter = kmp.match_iter(arr.iter().cloned());
    assert_eq!(iter.next(), Some(0));
    assert_eq!(iter.next(), Some(1));
    assert_eq!(iter.next(), Some(2));
    assert_eq!(iter.next(), None);
}
