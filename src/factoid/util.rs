use std::iter::Peekable;

// e.g. awoo -> awooooo or meow -> meoooow
pub fn is_extension(base: &str, candidate: &str) -> bool {
    if base.len() == 0 && candidate.len() == 0 {
        return true;
    } else if base.len() == 0 && candidate.len() > 0 {
        return false;
    }

    let my_base = base.to_lowercase().to_string();
    let my_candidate = candidate.to_lowercase().to_string();

    let mut bs = my_base.chars().peekable();
    let mut cs = my_candidate.chars().peekable();

    let mut b = bs.next().unwrap(); // If you pass in an empty base it's
                                    // your problem >:(
    let mut c = match cs.next() {
        Some(chr) => chr,
        None => return false,
    };
    loop {
        // first, fast forward bs to the end of its current "run",
        // while making sure cs moves with us
        co_fast_forward(&mut bs, b, &mut cs, c);

        // We've moved b and c to the end of their *shared* run.  Now,
        // keep moving c forward til it finishes that *entire* run, if
        // its run was longer. If c runs out entirely, move b forward
        // one as well. If it had more left, they didn't match. If
        // it's also done, they did.

        while b == c {
            c = match cs.next() {
                Some(chr) => chr,
                None => return bs.next().is_none(),
            };
        }

        // if we're still here, cs had at least 1 element left. bs
        // must have at least 1 element left as well, or the two don't
        // match.
        b = match bs.next() {
            Some(chr) => chr,
            None => return false,
        };

        // if their next chars don't match, this can't work.
        if b != c {
            return false;
        }
    }
}

#[test]
pub fn test_is_extension() {
    assert!(is_extension(&"awoo", &"awoo"));
    assert!(is_extension(&"awoo", &"awooo"));
    assert!(is_extension(&"awoo", &"aawoo"));
    assert!(is_extension(&"awoo", &"awwoo"));
    assert!(!is_extension(&"awoo", &"awo"));
    assert!(!is_extension(&"awwo", &"awo"));
    assert!(!is_extension(&"awoo", &"ao"));
    assert!(!is_extension(&"awoo", &"aowo"));
    assert!(!is_extension(&"awoo", &"aw0o"));
}

fn co_fast_forward<I, T>(i1: &mut Peekable<I>, h1: T, i2: &mut Peekable<I>, h2: T)
where
    I: std::iter::Iterator<Item = T>,
    T: std::cmp::PartialEq + Copy + Clone,
{
    while let (Some(p1), Some(p2)) = (i1.peek(), i2.peek()) {
        if p1 == &h1 && p2 == &h2 && h1 == h2 {
            i1.next();
            i2.next();
        } else {
            // run is over, break
            return;
        }
    }
}
