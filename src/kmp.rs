#[derive(Debug)]
struct KMP {
    pat: Vec<char>,

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
fn mk_pfx(pat: &[char]) -> Vec<usize> {
    let mut ret = vec![0; pat.len()];

    // Number of characters matched
    let mut n_matched = 0;

    for i in 1..pat.len() {
        let next = pat[i];

        while n_matched > 0 && pat[n_matched] != next {
            // Next character does not match, backtrack. Similar to the backtracking code in
            // `match_`.
            n_matched = ret[n_matched - 1];
        }
        if pat[n_matched] == next {
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

impl KMP {
    pub fn new(pat: Vec<char>) -> KMP {
        assert!(!pat.is_empty());
        let pfx = mk_pfx(&pat);
        KMP { pat, pfx }
    }

    pub fn match_(&self, t: &[char]) -> Vec<usize> {
        let mut ret = vec![];

        // Number of characters matched
        let mut n_matched = 0;

        for i in 0..t.len() {
            while n_matched > 0 && self.pat[n_matched] != t[i] {
                // Next character does not match, backtrack so that next n_matched will be the
                // longest prefix of current match that's also a suffix of the current match.
                n_matched = self.pfx[n_matched];
            }
            if self.pat[n_matched] == t[i] {
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
}

#[test]
fn test_kmp() {
    fn kmp(pat: &str, text: &str) -> Vec<usize> {
        KMP::new(pat.chars().collect()).match_(&(text.chars().collect::<Vec<_>>()))
    }

    assert_eq!(kmp("ababaca", "ababaca"), vec![0]);

    assert_eq!(kmp("ab", "ababaca"), vec![0, 2]);

    assert_eq!(kmp("aaba", "aabaacaadaabaaba"), vec![0, 9, 12]);

    assert_eq!(kmp("foo", ""), vec![]);
}
