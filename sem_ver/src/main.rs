use num_integer::Integer;
use regex::Regex;
use std::{cmp::*, fmt, str::FromStr};

#[derive(Clone, Debug)]
struct SemVer<Major: Integer = u16, Minor: Integer = u16, Patch: Integer = u32> {
    major: Major,
    minor: Minor,
    patch: Patch,
    prerelease: Option<Box<str>>,
    buildmetadata: Option<Box<str>>,
}

impl<Major, Minor, Patch> fmt::Display for SemVer<Major, Minor, Patch>
where
    Major: Integer + fmt::Display,
    Minor: Integer + fmt::Display,
    Patch: Integer + fmt::Display,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)?;
        if let Some(prerelease) = &self.prerelease {
            write!(f, "-{}", &*prerelease)?;
        }
        if let Some(buildmetadata) = &self.buildmetadata {
            write!(f, "+{}", &*buildmetadata)?;
        }
        Ok(())
    }
}

impl<Major, Minor, Patch> Eq for SemVer<Major, Minor, Patch>
where
    Major: Integer,
    Minor: Integer,
    Patch: Integer,
{
}

impl<Major, Minor, Patch> PartialEq for SemVer<Major, Minor, Patch>
where
    Major: Integer,
    Minor: Integer,
    Patch: Integer,
{
    fn eq(&self, other: &Self) -> bool {
        self.cmp(other).is_eq()
    }
}

impl<Major, Minor, Patch> Ord for SemVer<Major, Minor, Patch>
where
    Major: Integer,
    Minor: Integer,
    Patch: Integer,
{
    fn cmp(&self, other: &Self) -> Ordering {
        fn cmp_meta(first: &Option<Box<str>>, second: &Option<Box<str>>) -> Ordering {
            match (first.as_ref(), second.as_ref()) {
                (None, None) => Ordering::Equal,
                (None, Some(_)) => Ordering::Greater,
                (Some(_), None) => Ordering::Less,
                (Some(first), Some(second)) => first.cmp(&*second),
            }
        }

        self.major
            .cmp(&other.major)
            .then_with(|| self.minor.cmp(&other.minor))
            .then_with(|| self.patch.cmp(&other.patch))
            .then_with(|| cmp_meta(&self.prerelease, &other.prerelease))
            .then_with(|| cmp_meta(&self.buildmetadata, &other.buildmetadata))
    }
}

impl<Major, Minor, Patch> PartialOrd for SemVer<Major, Minor, Patch>
where
    Major: Integer,
    Minor: Integer,
    Patch: Integer,
{
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[derive(Clone, Debug)]
enum SemVerParseError {
    NotASemVer,
    NotSingleSemVer,
}

impl<'t, Major, Minor, Patch> FromStr for SemVer<Major, Minor, Patch>
where
    Major: Integer,
    Minor: Integer,
    Patch: Integer,
{
    type Err = SemVerParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let regex = Regex::new(concat!(
            r"^[vV]?",
            r"(?P<major>0|[1-9]\d*)",
            r"\.(?P<minor>0|[1-9]\d*)",
            r"\.(?P<patch>0|[1-9]\d*)",
            r"(?:-(?P<prerelease>(?:0|[1-9]\d*|\d*[a-zA-Z-][0-9a-zA-Z-]*)(?:\.(?:0|[1-9]\d*|\d*[a-zA-Z-][0-9a-zA-Z-]*))*))?",
            r"(?:\+(?P<buildmetadata>[0-9a-zA-Z-]+(?:\.[0-9a-zA-Z-]+)*))?",
            r"$",
        )).unwrap();
        let mut captures = regex.captures_iter(s).into_iter();

        match (captures.next(), captures.next()) {
            (Some(capture), None) => unsafe {
                // SAFETY: obligatory regex groups
                let major = capture.name("major").unwrap_unchecked().as_str();
                let minor = capture.name("minor").unwrap_unchecked().as_str();
                let patch = capture.name("patch").unwrap_unchecked().as_str();
                Ok(SemVer {
                    // SAFETY: regex contains only decimal digits
                    major: Major::from_str_radix(major, 10).unwrap_unchecked(),
                    minor: Minor::from_str_radix(minor, 10).unwrap_unchecked(),
                    patch: Patch::from_str_radix(patch, 10).unwrap_unchecked(),
                    prerelease: capture.name("prerelease").map(|m| Box::from(m.as_str())),
                    buildmetadata: capture.name("buildmetadata").map(|m| Box::from(m.as_str())),
                })
            },
            (Some(_), Some(_)) => Err(SemVerParseError::NotSingleSemVer),
            _ => Err(SemVerParseError::NotASemVer),
        }
    }
}

fn main() {
    let sem_ver = vec!["0.0.1", "2.2.0", "v3.0.0-rc", "3.0.0"];
    let sem_ver = sem_ver
        .into_iter()
        .map(|v| v.parse::<SemVer>())
        .collect::<Result<Vec<_>, _>>()
        .unwrap();

    let (start, end) = (sem_ver[1].clone(), sem_ver[3].clone());
    let range = start..=end;

    for sv in sem_ver.iter() {
        println!(
            "{} {} in range",
            sv,
            if range.contains(sv) { "is" } else { "isn't" }
        );
    }
}
