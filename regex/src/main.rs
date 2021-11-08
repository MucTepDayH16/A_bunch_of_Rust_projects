use std::{
    collections::VecDeque,
    fmt,
};

#[derive(Clone, Debug)]
enum Match {
    EndLn,
    Char(char),
    Range(char, char),
    Any,
    Star(Box<Match>),
    Plus(Box<Match>),
    Ques(Box<Match>),
    Group(VecDeque<Match>),
    Or(VecDeque<Match>),
    NotOr(VecDeque<Match>),
}

impl Into<Match> for char {
    fn into(self) -> Match {
        Match::Char(self)
    }
}

impl<const N: usize> Into<Match> for [Match; N] {
    fn into(self) -> Match {
        Match::Or(self.into_iter().collect())
    }
}

impl Into<Match> for (char, char) {
    fn into(self) -> Match {
        Match::Range(self.0, self.1)
    }
}

impl Match {
    fn star(self) -> Self {
        Match::Star(Box::new(self))
    }

    fn plus(self) -> Self {
        Match::Plus(Box::new(self))
    }

    fn ques(self) -> Self {
        Match::Ques(Box::new(self))
    }
}

#[derive(Debug)]
struct Regex(VecDeque<Match>, String);

impl fmt::Display for Regex {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self.1)
    }
}

impl Regex {
    fn new<S: ToString>(pat: S) -> Option<Self> {
        let source = pat.to_string();
        let mut pat = source.chars().peekable();

        let mut range = None;
        let mut brackets = VecDeque::from(['(']);
        let mut to_push = VecDeque::from([VecDeque::new()]);

        const UNESCAPES: [(char, char); 8] = [
            ('a', '\x07'),
            ('b', '\x08'),
            ('t', '\x09'),
            ('n', '\x0A'),
            ('v', '\x0B'),
            ('f', '\x0C'),
            ('r', '\x0D'),
            ('e', '\x1B'),
        ];

        const UPPER: Match = Match::Range('A', 'Z');
        const LOWER: Match = Match::Range('a', 'z');
        const DIGIT: Match = Match::Range('0', '9');
        const CNTRL: (Match, Match) = (
            Match::Range('\x00', '\x1F'),
            Match::Char('\x7F'),
        );
        const GRAPH: Match = Match::Range('\x21', '\x7E');
        const PRINT: Match = Match::Range('\x20', '\x7E');

        while let Some(c) = pat.next() {
            match c {
                '\\' => {
                    match pat.next()? {
                        c @ ('\\' | '-' | '|' | '(' | '[' | ')' | ']' | '*' | '+' | '?' | '.' | ':') =>
                            to_push[0].push_back(c.into()),
                        c @ ('a' | 'b' | 't' | 'n' | 'v' | 'f' | 'r' | 'e') => {
                            let esc = UNESCAPES.iter()
                                .find(|i| i.0 == c)
                                .unwrap().1;
                            to_push[0].push_back(esc.into());
                        }
                        'd' => to_push[0].push_back(DIGIT),
                        'w' => to_push[0].push_back([
                                UPPER, LOWER, DIGIT, '_'.into(),
                            ].into()),
                        's' => to_push[0].push_back([
                                ' '.into(), '\t'.into(), '\x0B'.into(),
                                '\r'.into(), '\n'.into(), '\x0C'.into(),
                            ].into()),
                        'c' => {
                            match pat.next()? {
                                c @ 'A'..='Z' => {
                                    let ctrl = ('A'..='Z').zip('\x01'..='\x1A')
                                        .find(|i| i.0 == c)
                                        .unwrap().1;
                                    to_push[0].push_back(Match::Char(ctrl));
                                },
                                _ => return None,
                            }
                        }
                        _ => return None,
                    }
                }
                '-' if range.is_some() => {
                    if let Match::Char(c0) = to_push[0].pop_back()? {
                        let c1 = pat.next()?;
                        if c0 > c1 {
                            return None;
                        }
                        to_push[0].push_back((c0, c1).into());
                        range = None;
                    } else {
                        return None;
                    }
                }
                '|' => {
                    if brackets.front()? == &'|' {
                        let circle = Match::Group(to_push.pop_front()?);
                        to_push[0].push_back(circle);
                        to_push.push_front(VecDeque::new());
                    } else {
                        brackets.push_front('|');
                        let circle = Match::Group(to_push.pop_front()?);
                        to_push.push_front(VecDeque::from([circle]));
                        to_push.push_front(VecDeque::new());
                    }
                }
                '(' => {
                    brackets.push_front(c);
                    to_push.push_front(VecDeque::new());
                }
                '[' => {
                    if pat.peek()? == &':' {
                        let _ = pat.next();
                        if brackets.front()? == &'[' {
                            let mut class = String::new();
                            while let Some(c) = pat.next_if(|&c| c != ':') {
                                class.push(c);
                            }
                            let _ = pat.next();
                            if let Some(']') = pat.next() {
                                match class.as_str() {
                                    "upper" => to_push[0].push_back(UPPER),
                                    "lower" => to_push[0].push_back(LOWER),
                                    "alpha" =>
                                        to_push[0].push_back([
                                            UPPER, LOWER,
                                        ].into()),
                                    "digit" => to_push[0].push_back(DIGIT),
                                    "xdigit" =>
                                        to_push[1].push_back([
                                            DIGIT, ('A', 'F').into(), ('a', 'f').into(),
                                        ].into()),
                                    "alnum" =>
                                        to_push[0].push_back([
                                            UPPER, LOWER, DIGIT,
                                        ].into()),
                                    "ascii" => to_push[0].push_back(('\x00', '\x7F').into()),
                                    "word" =>
                                        to_push[0].push_back([
                                            UPPER, LOWER, DIGIT, '_'.into(),
                                        ].into()),
                                    "punct" => {
                                        let or = Match::Or(
                                            "-!\"#$%&'()*+,./:;<=>?@[\\]_`{|}~".chars()
                                                .map(char::into)
                                                .collect()
                                        );
                                        to_push[0].push_back(or);
                                    }
                                    "blank" =>
                                        to_push[0].push_back([
                                            ' '.into(), '\t'.into(),
                                        ].into()),
                                    "space" =>
                                        to_push[0].push_back([
                                            ' '.into(), '\t'.into(), '\x0B'.into(),
                                            '\r'.into(), '\n'.into(), '\x0C'.into(),
                                        ].into()),
                                    "cntrl" =>
                                        to_push[0].push_back([
                                            CNTRL.0, CNTRL.1,
                                        ].into()),
                                    "graph" => to_push[0].push_back(GRAPH),
                                    "print" => to_push[0].push_back(PRINT),
                                    _ => return None,
                                }
                            } else {
                                return None;
                            }
                        } else {
                            return None;
                        }
                    } else {
                        if let Some(_) = pat.next_if_eq(&'^') {
                            brackets.push_front('^');
                        }
                        brackets.push_front(c);
                        to_push.push_front(VecDeque::new());
                    }
                }
                ')' => {
                    if let Some(c) = brackets.pop_front() {
                        if c == '(' {
                            let group = to_push.pop_front()?;
                            to_push[0].push_back(Match::Group(group));
                        } else if c == '|' {
                            brackets.pop_front();
                            let group = to_push.pop_front()?;
                            to_push[0].push_back(Match::Group(group));
                            let group = to_push.pop_front()?;
                            let groups = VecDeque::from([Match::Or(group)]);
                            to_push[0].push_back(Match::Group(groups));
                        } else {
                            return None;
                        }
                    } else {
                        return None;
                    }
                }
                ']' => {
                    if let Some('[') = brackets.pop_front() {
                        let group = to_push
                            .pop_front()?
                            .into_iter()
                            .flat_map(|m| match m {
                                Match::Or(v) => v,
                                m => VecDeque::from([m]),
                            }).collect::<VecDeque<_>>();
                        if let Some('^') = brackets.front() {
                            group.iter().try_for_each(
                                |m| match m {
                                    Match::Char(_) | Match::Range(_, _) => Some(()),
                                    _ => None,
                                }
                            )?;
                            to_push[0].push_back(Match::NotOr(group));
                        } else {
                            to_push[0].push_back(Match::Or(group));
                        }
                    } else {
                        return None;
                    }
                }
                '*' => {
                    let m = to_push[0].pop_back()?.star();
                    to_push[0].push_back(m);
                }
                '+' => {
                    let m = to_push[0].pop_back()?.plus();
                    to_push[0].push_back(m);
                }
                '?' => {
                    let m = to_push[0].pop_back()?.ques();
                    to_push[0].push_back(m);
                }
                '.' => {
                    to_push[0].push_back(Match::Any)
                }
                c => {
                    range = Some(c);
                    to_push[0].push_back(c.into())
                }
            }
        }

        if to_push.len() != 1 {
            None
        } else {
            let mut regex = to_push.pop_front().unwrap();
            regex.push_back(Match::EndLn);
            Some(Regex(regex, source))
        }
    }

    fn is_match<S: ToString>(&self, src: S) -> bool {
        fn iter_match<'a, M, S>(mut regex: M, src: &mut S) -> bool
        where M: Iterator<Item=&'a Match> + Clone, S: Iterator<Item=char> + Clone {
            if let Some(m) = regex.next() {
                match m {
                    Match::EndLn => src.next().is_none(),
                    Match::Char(c) => {
                        src.next().filter(|s| s == c).is_some()
                            && iter_match(regex, src)
                    }
                    Match::Range(c0, c1) => {
                        let range = (*c0)..=(*c1);
                        src.next().filter(|s| range.contains(s)).is_some()
                            && iter_match(regex, src)
                    }
                    Match::Any => {
                        src.next().is_some()
                            && iter_match(regex, src)
                    }
                    Match::Star(m) => {
                        let mut regex = regex.collect::<VecDeque<_>>();

                        let mut src_x = src.clone();
                        loop {
                            let mut src_y = src.clone();
                            if iter_match(regex.clone().into_iter(), &mut src_y) {
                                *src = src_y;
                                return true;
                            }
                            if iter_match(Some(m.as_ref().clone()).iter(), &mut src_x) {
                                regex.push_front(m.as_ref());
                            } else {
                                return false;
                            }
                        }
                    }
                    Match::Plus(m) => {
                        let mut regex = regex.collect::<VecDeque<_>>();

                        let mut src_x = src.clone();
                        loop {
                            if iter_match(Some(m.as_ref().clone()).iter(), &mut src_x) {
                                regex.push_front(m.as_ref());
                            } else {
                                return false;
                            }
                            let mut src_y = src.clone();
                            if iter_match(regex.clone().into_iter(), &mut src_y) {
                                *src = src_y;
                                return true;
                            }
                        }
                    }
                    Match::Ques(m) => {
                        let mut group = VecDeque::from([m.as_ref()]);
                        group.extend(regex.clone());

                        let mut src_x = src.clone();
                        if iter_match(group.into_iter(), &mut src_x) {
                            *src = src_x;
                            true
                        } else {
                            iter_match(regex, src)
                        }
                    }
                    Match::Group(g) => {
                        let mut group = g.iter().collect::<VecDeque<_>>();
                        group.extend(regex);
                        iter_match(group.into_iter(), src)
                    }
                    Match::Or(g) => {
                        let regex = regex.collect::<VecDeque<_>>();

                        for m in g {
                            if iter_match(Some(m.clone()).iter(), &mut src.clone()) {
                                let mut regex: VecDeque<&Match> = regex.clone();
                                regex.push_front(m);
                                let mut src_x = src.clone();
                                if iter_match(regex.into_iter(), &mut src_x) {
                                    *src = src_x;
                                    return true;
                                }
                            }
                        }
                        false
                    }
                    Match::NotOr(g) => {
                        for m in g {
                            if iter_match(Some(m.clone()).iter(), &mut src.clone()) {
                                return false;
                            }
                        }
                        src.next().is_some() && iter_match(regex, src)
                    }
                }
            } else {
                true
            }
        }

        let src = src.to_string();
        iter_match(self.0.iter(), &mut src.chars())
    }
}

fn main() {
	let regex = Regex::new(r"[^[:digit:]]*\.[^[:digit:]]+").unwrap();

	let sources = vec![
		"Denis_Dr0zhzhin.gif",
		"not_a_file_.org",
        "png.image0",
		".net",
		".NET",
        ".,/?",
		"0_.",
		"0_0",
	];
	for source in sources {
		println!(" {:?} -> {}", source, regex.is_match(source));
	}
}
