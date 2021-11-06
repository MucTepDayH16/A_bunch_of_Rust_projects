use std::{
    collections::VecDeque,
	iter::Peekable,
};

#[derive(Clone, Debug, PartialEq, Eq)]
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
struct Regex(VecDeque<Match>);

impl Regex {
    fn new<S: ToString>(pat: S) -> Option<Self> {
        let pat = pat.to_string();
        let mut pat = pat.chars();

        let mut range = None;
        let mut brackets = VecDeque::from(['(']);
        let mut to_push = VecDeque::from([VecDeque::new()]);

        while let Some(c) = pat.next() {
            match c {
                ' ' => {}
                '\\' => {
                    match pat.next()? {
                        ' ' => return None,
                        's' => to_push[0].push_back(Match::Char(' ')),
                        c => to_push[0].push_back(Match::Char(c)),
                    }
                }
                '-' if range.is_some() => {
                    if let Match::Char(c0) = to_push[0].pop_back()? {
                        let c1 = pat.next()?;
                        if c0 > c1 {
                            return None;
                        }
                        to_push[0].push_back(Match::Range(c0, c1));
                        range = None;
                    } else {
                        return None;
                    }
                }
                '|' => {
                    if brackets[0] == '|' {
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
                c @ ('(' | '[') => {
                    brackets.push_front(c);
                    to_push.push_front(VecDeque::new());
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
                        let group = to_push.pop_front()?;
                        to_push[0].push_back(Match::Or(group));
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
                    to_push[0].push_back(Match::Char(c))
                }
            }
        }

        if to_push.len() != 1 {
            None
        } else {
            let mut regex = to_push.pop_front().unwrap();
            regex.push_back(Match::EndLn);
            Some(Regex(regex))
        }
    }

    fn is_match<S: ToString>(&self, src: S) -> bool {
        fn inner_is_match<'a, M, S>(mut regex: M, src: &mut Peekable<S>) -> bool
        where M: Iterator<Item=&'a Match> + Clone, S: Iterator<Item=char> + Clone {
            if let Some(m) = regex.next() {
                match m {
                    Match::EndLn => src.peek().is_none(),
                    Match::Char(c) => {
                        src.next_if_eq(c).is_some()
                            && inner_is_match(regex, src)
                    }
                    Match::Range(c0, c1) => {
                        let range = (*c0)..=(*c1);
                        src.next_if(|s| range.contains(s)).is_some()
                            && inner_is_match(regex, src)
                    }
                    Match::Any => {
                        src.next().is_some()
                            && inner_is_match(regex, src)
                    }
                    Match::Star(m) => {
                        let mut group = VecDeque::new();
                        group.extend(regex.clone());

                        let mut src_x = src.clone();
                        loop {
                            if inner_is_match(group.clone().into_iter(), &mut src.clone()) {
                                return true;
                            } else {
                                group.push_front(m.as_ref());
                            }
                            if !inner_is_match(Some(m.as_ref().clone()).iter(), &mut src_x) {
                                return false;
                            }
                        }
                    }
                    Match::Plus(m) => {
                        let mut group = VecDeque::from([m.as_ref()]);
                        group.extend(regex.clone());

                        let mut src_x = src.clone();
                        while inner_is_match(Some(m.as_ref().clone()).iter(), &mut src_x) {
                            if inner_is_match(group.clone().into_iter(), &mut src.clone()) {
                                return true;
                            } else {
                                group.push_front(m.as_ref());
                            }
                        }
                        false
                    }
                    Match::Ques(m) => {
                        let mut group = VecDeque::from([m.as_ref()]);
                        group.extend(regex.clone());

                        let mut src_x = src.clone();
                        if inner_is_match(group.into_iter(), &mut src_x) {
                            *src = src_x;
                            true
                        } else {
                            inner_is_match(regex, src)
                        }
                    }
                    Match::Group(g) => {
                        let mut group = g.iter().collect::<VecDeque<&Match>>();
                        group.extend(regex);
                        inner_is_match(group.into_iter(), src)
                    }
                    Match::Or(g) => {
                        for m in g {
                            let mut src_x = src.clone();
                            if inner_is_match(Some(m.clone()).iter(), &mut src_x) {
                                *src = src_x;
                                return inner_is_match(regex, src);
                            }
                        }
                        false
                    }
                }
            } else {
                true
            }
        }

        let regex = self.0.iter();
        let src = src.to_string();
        let src = &mut src.chars().peekable();
        inner_is_match(regex, src)
    }
}

fn main() {
	let pattern = r"[_a-zA-Z0-9]*(\.[a-z]+)?";
	let regex = Regex::new(pattern).unwrap();
    println!("{:?}", pattern);
	
	let sources = vec![
		"Denis_Dr0zhzhin.gif",
		"not_a_file_.org",
		".net",
		".NET",
		"0_.",
		"0_0",
	];
	for source in sources {
		println!(" {:?} -> {}", source, regex.is_match(source));
	}
}
