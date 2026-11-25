//! The command line (`docs/KONZEPT.md` 3.2, plan 2.5): `tasqlet add|open|current|list`. Reading
//! the arguments and carrying out a command on a [`Session`] live here; the shell sends the command
//! to the running app over its socket, or, if none runs, carries it out on the files itself.
//! Opening is the shell's: a command only says which resources to open.

use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use crate::jira::JiraProjects;
use crate::model::{Id, Item, NewResource, State, Target};
use crate::quick::parse_with;
use crate::resources::{classify, normalize, ranked};
use crate::{Action, Session};

/// Where `add` puts the entry; without an option at the end of the focus list (D37).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Place {
    Now,
    #[default]
    Focus,
    Inter,
    Idea,
}

/// A command of the command line.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase", tag = "do")]
pub enum Command {
    /// Capture an entry; the text is read like the quick entry (`@Name`, `!date`, links).
    Add {
        text: String,
        place: Place,
        links: Vec<String>,
    },
    /// Open the resource with this digit of the current task, or all of them.
    Open { digit: Option<u8> },
    /// Show the current task (#1 in focus).
    Current { json: bool },
    /// Show the open entries, or those in one state.
    List { state: Option<State>, json: bool },
}

/// What the arguments ask for.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Parsed {
    /// No command: start the app (or show its popover); `toggle` for `--toggle`.
    App {
        toggle: bool,
    },
    Help,
    Version,
    Run(Command),
}

pub const USAGE: &str = "\
Usage:
  tasqlet                       start Tasqlet in the tray, or show its popover
  tasqlet --toggle              show or hide the popover (for a desktop key)
  tasqlet add TEXT… [--now|--focus|--inter|--idea] [--link URL|PATH]…
                                capture an entry (default: end of the focus list);
                                TEXT is read like the quick entry: @Name, !date, links
  tasqlet open [N]              open resource N (Alt+N) of the current task, or all
  tasqlet current [--json]      the current task and its resources
  tasqlet list [--state S] [--json]
                                open entries; S: focus, intermezzo, idea, delegated, done
  tasqlet --help | --version
";

fn state_from(word: &str) -> Option<State> {
    Some(match word {
        "focus" => State::Focus,
        "intermezzo" | "inter" => State::Intermezzo,
        "idea" | "ideas" => State::Idea,
        "delegated" => State::Delegated,
        "done" => State::Done,
        _ => return None,
    })
}

/// Read the arguments (without the program name).
pub fn parse_args(args: &[String]) -> Result<Parsed, String> {
    let Some(first) = args.first() else {
        return Ok(Parsed::App { toggle: false });
    };
    let rest = &args[1..];
    let mut json = false;
    let mut words: Vec<&str> = Vec::new();
    let mut flags: Vec<(&str, Option<&str>)> = Vec::new();
    let mut i = 0;
    let mut only_words = false;
    while i < rest.len() {
        let a = rest[i].as_str();
        if only_words || !a.starts_with("--") {
            words.push(a);
        } else if a == "--" {
            only_words = true;
        } else if a == "--json" {
            json = true;
        } else if let Some((name, value)) = a.split_once('=') {
            flags.push((name, Some(value)));
        } else if matches!(a, "--link" | "--state") {
            let value = rest.get(i + 1).ok_or(format!("{a} needs a value"))?;
            flags.push((a, Some(value.as_str())));
            i += 1;
        } else {
            flags.push((a, None));
        }
        i += 1;
    }
    let unknown = |flag: &str| Err(format!("unknown option {flag}"));
    match first.as_str() {
        "--toggle" if rest.is_empty() => Ok(Parsed::App { toggle: true }),
        "--help" | "-h" | "help" => Ok(Parsed::Help),
        "--version" | "-V" => Ok(Parsed::Version),
        "add" => {
            let mut place = None;
            let mut links = Vec::new();
            for (flag, value) in flags {
                let p = match flag {
                    "--now" => Place::Now,
                    "--focus" => Place::Focus,
                    "--inter" => Place::Inter,
                    "--idea" => Place::Idea,
                    "--link" => {
                        links.push(value.unwrap_or_default().to_string());
                        continue;
                    }
                    f => return unknown(f),
                };
                if place.replace(p).is_some_and(|q| q != p) {
                    return Err("only one of --now, --focus, --inter, --idea".into());
                }
            }
            if json {
                return unknown("--json");
            }
            let text = words.join(" ");
            if text.trim().is_empty() && links.is_empty() {
                return Err("add needs a text or a link".into());
            }
            Ok(Parsed::Run(Command::Add {
                text,
                place: place.unwrap_or_default(),
                links,
            }))
        }
        "open" => {
            if let Some((f, _)) = flags.first() {
                return unknown(f);
            }
            let digit = match words.as_slice() {
                [] => None,
                [n] => Some(
                    n.parse::<u8>()
                        .ok()
                        .filter(|d| (1..=9).contains(d))
                        .ok_or("N is a digit from 1 to 9")?,
                ),
                _ => return Err("open takes at most one number".into()),
            };
            Ok(Parsed::Run(Command::Open { digit }))
        }
        "current" => match (flags.first(), words.first()) {
            (Some((f, _)), _) => unknown(f),
            (_, Some(w)) => Err(format!("unexpected {w}")),
            _ => Ok(Parsed::Run(Command::Current { json })),
        },
        "list" => {
            let mut state = None;
            for (flag, value) in flags {
                match flag {
                    "--state" => {
                        let v = value.unwrap_or_default();
                        state = Some(state_from(v).ok_or(format!("unknown state {v}"))?);
                    }
                    f => return unknown(f),
                }
            }
            if let Some(w) = words.first() {
                return Err(format!("unexpected {w}"));
            }
            Ok(Parsed::Run(Command::List { state, json }))
        }
        other => Err(format!("unknown command {other}")),
    }
}

/// What a command did: the text to print, the resources to open (item, resource, target) and the
/// links whose Jira projects are to be learned.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Outcome {
    pub out: String,
    pub open: Vec<(Id, Id, String)>,
    pub learn: Vec<String>,
    pub changed: bool,
}

/// An item for `--json`: the data file's shape, resources by rank with their digit and label.
pub fn item_json(it: &Item) -> Value {
    let mut value = serde_json::to_value(it).unwrap_or(Value::Null);
    value["resources"] = ranked(&it.resources)
        .into_iter()
        .map(|r| {
            let mut json = serde_json::to_value(r.resource).unwrap_or(Value::Null);
            json["shown"] = r.resource.shown_label().into();
            json["digit"] = r.digit.into();
            json
        })
        .collect();
    value
}

fn title(it: &Item) -> &str {
    if it.title.is_empty() {
        "(untitled)"
    } else {
        &it.title
    }
}

fn state_name(state: State) -> &'static str {
    match state {
        State::Focus => "focus",
        State::Intermezzo => "intermezzo",
        State::Idea => "idea",
        State::Delegated => "delegated",
        State::Done => "done",
    }
}

/// Carry out a command. Changes go through the session (undo in the app).
pub fn run(
    session: &mut Session,
    jira: &JiraProjects,
    command: &Command,
    today: NaiveDate,
    now: DateTime<Utc>,
) -> Result<Outcome, String> {
    let store = session.store();
    let current = store.list(State::Focus).first().map(|it| (*it).clone());
    match command {
        Command::Add { text, place, links } => {
            let mut entry = parse_with(text, today, jira);
            for link in links {
                let target = normalize(link);
                let kind = classify(&target).ok_or(format!("not a link or path: {link}"))?;
                if !entry.resources.iter().any(|r| r.target == target) {
                    entry.resources.push(NewResource {
                        kind,
                        target,
                        label: None,
                    });
                }
            }
            if entry.is_empty() {
                return Err("nothing to add".into());
            }
            let learn = entry
                .resources
                .iter()
                .filter(|r| r.kind == crate::ResourceType::Url)
                .map(|r| r.target.clone())
                .collect();
            let chosen = match place {
                Place::Now => Target::Now,
                Place::Focus => Target::Focus,
                Place::Inter => Target::Intermezzo,
                Place::Idea => Target::Idea,
            };
            let target = entry.target(chosen);
            let item = entry.into_new_item();
            let id = item.id;
            session
                .apply(
                    Action::Add {
                        item,
                        target: target.clone(),
                    },
                    now,
                )
                .map_err(|e| e.to_string())?;
            let store = session.store();
            let it = store.get(id).ok_or("lost the new entry")?;
            let what = match target {
                Target::Now => "Started on #1".to_string(),
                Target::Focus => {
                    let n = store.list(State::Focus).iter().position(|x| x.id == id);
                    format!("Added to focus #{}", n.map_or(0, |n| n + 1))
                }
                Target::Intermezzo => "Added to intermezzo".into(),
                Target::Idea => "Added to ideas".into(),
                Target::Delegated(name) => format!("Delegated to {name}"),
            };
            Ok(Outcome {
                out: format!("{what}: {}\n", title(it)),
                learn,
                changed: true,
                ..Outcome::default()
            })
        }
        Command::Open { digit } => {
            let it = current.ok_or("there is no current task")?;
            let mut open: Vec<(Id, Id, String)> = ranked(&it.resources)
                .into_iter()
                .filter(|r| digit.is_none() || r.digit == *digit)
                .map(|r| (it.id, r.resource.id, r.resource.target.clone()))
                .collect();
            if open.is_empty() {
                return Err(match digit {
                    Some(d) => format!("the current task has no resource {d}"),
                    None => "the current task has no resources".into(),
                });
            }
            open.truncate(32);
            Ok(Outcome {
                open,
                ..Outcome::default()
            })
        }
        Command::Current { json } => {
            let out = match (&current, json) {
                (None, true) => "null\n".into(),
                (None, false) => return Err("there is no current task".into()),
                (Some(it), true) => format!("{}\n", item_json(it)),
                (Some(it), false) => {
                    let mut out = format!(
                        "{}{}\n",
                        title(it),
                        if store.on_break() { "  (break)" } else { "" }
                    );
                    for r in ranked(&it.resources) {
                        let key = r.digit.map_or(" ".into(), |d| d.to_string());
                        out += &format!(
                            "  {key}  {}  {}\n",
                            r.resource.shown_label(),
                            r.resource.target
                        );
                    }
                    out
                }
            };
            Ok(Outcome {
                out,
                ..Outcome::default()
            })
        }
        Command::List { state, json } => {
            let states = match state {
                Some(s) => vec![*s],
                None => vec![
                    State::Focus,
                    State::Intermezzo,
                    State::Delegated,
                    State::Idea,
                ],
            };
            let out = if *json {
                let items: Vec<Value> = states
                    .iter()
                    .flat_map(|s| store.list(*s))
                    .map(item_json)
                    .collect();
                format!("{}\n", json!(items))
            } else {
                let mut out = String::new();
                for s in &states {
                    let items = store.list(*s);
                    if state.is_none() && items.is_empty() {
                        continue;
                    }
                    if state.is_none() {
                        out += &format!("{}:\n", state_name(*s));
                    }
                    for (n, it) in items.iter().enumerate() {
                        let mark = match s {
                            State::Focus => format!("#{}", n + 1),
                            State::Delegated => {
                                format!("@{}", it.delegated_to.as_deref().unwrap_or("?"))
                            }
                            _ => "-".into(),
                        };
                        out += &format!("  {mark}  {}\n", title(it));
                    }
                }
                out
            };
            Ok(Outcome {
                out,
                ..Outcome::default()
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Store;
    use chrono::TimeZone;

    fn args(line: &str) -> Vec<String> {
        line.split_whitespace().map(String::from).collect()
    }

    fn now() -> DateTime<Utc> {
        Utc.with_ymd_and_hms(2026, 11, 25, 10, 0, 0).unwrap()
    }

    fn today() -> NaiveDate {
        NaiveDate::from_ymd_opt(2026, 11, 25).unwrap()
    }

    fn exec(session: &mut Session, line: &str) -> Result<Outcome, String> {
        let Parsed::Run(cmd) = parse_args(&args(line))? else {
            panic!("not a command: {line}");
        };
        run(session, &JiraProjects::default(), &cmd, today(), now())
    }

    #[test]
    fn arguments() {
        assert_eq!(parse_args(&[]).unwrap(), Parsed::App { toggle: false });
        assert_eq!(
            parse_args(&args("--toggle")).unwrap(),
            Parsed::App { toggle: true }
        );
        assert_eq!(
            parse_args(&args(
                "add Fix the build --now --link https://ci.example.com/1"
            ))
            .unwrap(),
            Parsed::Run(Command::Add {
                text: "Fix the build".into(),
                place: Place::Now,
                links: vec!["https://ci.example.com/1".into()],
            })
        );
        assert_eq!(
            parse_args(&args("add -- --weird title")).unwrap(),
            Parsed::Run(Command::Add {
                text: "--weird title".into(),
                place: Place::Focus,
                links: vec![],
            })
        );
        assert_eq!(
            parse_args(&args("list --state=delegated --json")).unwrap(),
            Parsed::Run(Command::List {
                state: Some(State::Delegated),
                json: true
            })
        );
        assert_eq!(
            parse_args(&args("open 2")).unwrap(),
            Parsed::Run(Command::Open { digit: Some(2) })
        );
        assert!(parse_args(&args("open 0")).is_err());
        assert!(parse_args(&args("add x --now --idea")).is_err());
        assert!(parse_args(&args("add")).is_err());
        assert!(parse_args(&args("list --state soon")).is_err());
        assert!(parse_args(&args("frobnicate")).is_err());
        assert!(parse_args(&args("current --link x")).is_err());
    }

    #[test]
    fn add_lands_at_the_end_of_focus_and_reads_like_the_quick_entry() {
        let mut s = Session::new(Store::new());
        let out = exec(&mut s, "add First").unwrap();
        assert_eq!(out.out, "Added to focus #1: First\n");
        let out = exec(&mut s, "add Second https://jira.example.com/browse/NMS-1").unwrap();
        assert_eq!(out.out, "Added to focus #2: Second\n");
        assert_eq!(out.learn, vec!["https://jira.example.com/browse/NMS-1"]);
        let out = exec(&mut s, "add Ask about it @Anna").unwrap();
        assert_eq!(out.out, "Delegated to Anna: Ask about it\n");
        exec(&mut s, "add Later --idea").unwrap();
        let focus = s.store().list(State::Focus);
        assert_eq!(focus[1].resources.len(), 1);
        assert_eq!(s.store().list(State::Idea).len(), 1);
        // Undo in the app takes it back, like any other change.
        s.undo().unwrap();
        assert!(s.store().list(State::Idea).is_empty());
    }

    #[test]
    fn a_link_with_spaces_is_kept_as_a_file_link() {
        let mut s = Session::new(Store::new());
        let cmd = Command::Add {
            text: "Read".into(),
            place: Place::Focus,
            links: vec!["/home/me/Mein Bericht.pdf".into()],
        };
        run(&mut s, &JiraProjects::default(), &cmd, today(), now()).unwrap();
        let it = s.store().list(State::Focus)[0].clone();
        assert_eq!(it.resources[0].target, "file:///home/me/Mein%20Bericht.pdf");
        let bad = Command::Add {
            text: "x".into(),
            place: Place::Focus,
            links: vec!["no link".into()],
        };
        assert!(run(&mut s, &JiraProjects::default(), &bad, today(), now()).is_err());
    }

    #[test]
    fn current_open_and_list() {
        let mut s = Session::new(Store::new());
        assert!(exec(&mut s, "current").is_err());
        assert_eq!(exec(&mut s, "current --json").unwrap().out, "null\n");
        exec(
            &mut s,
            "add Review https://github.com/o/repo/pull/42 /home/me/notes.md",
        )
        .unwrap();
        exec(&mut s, "add Second").unwrap();
        exec(&mut s, "add Small --inter").unwrap();
        let out = exec(&mut s, "current").unwrap().out;
        assert!(out.starts_with("Review\n"), "{out}");
        assert!(
            out.contains("1  PR #42  https://github.com/o/repo/pull/42"),
            "{out}"
        );
        assert!(out.contains("2  notes.md  /home/me/notes.md"), "{out}");
        let json: Value =
            serde_json::from_str(&exec(&mut s, "current --json").unwrap().out).unwrap();
        assert_eq!(json["title"], "Review");
        assert_eq!(json["resources"][0]["digit"], 1);

        let open = exec(&mut s, "open 2").unwrap().open;
        assert_eq!(open.len(), 1);
        assert_eq!(open[0].2, "/home/me/notes.md");
        assert_eq!(exec(&mut s, "open").unwrap().open.len(), 2);
        assert!(exec(&mut s, "open 3").is_err());

        let out = exec(&mut s, "list").unwrap().out;
        assert_eq!(
            out,
            "focus:\n  #1  Review\n  #2  Second\nintermezzo:\n  -  Small\n"
        );
        assert_eq!(
            exec(&mut s, "list --state idea").unwrap().out,
            "",
            "an empty list prints nothing"
        );
        let json: Value = serde_json::from_str(&exec(&mut s, "list --json").unwrap().out).unwrap();
        assert_eq!(json.as_array().unwrap().len(), 3);
    }
}
