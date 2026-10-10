//! Reference scripts and table-driven alignment scenarios.
//!
//! Each [`Scenario`] is a list of [`Step`]s fed to a [`crate::Speaker`];
//! `Step::Expect` checks state and cursor *at that point*. Expected values
//! were derived by hand from the algorithm (Smith-Waterman window,
//! hysteresis, state table in `zt-align/src/tracker.rs`) — if a tuning
//! change breaks one, re-derive rather than blindly update.
//!
//! Scenario categories (ad-lib, backtrack, forward jump) follow
//! EdNutting/autocue (MIT); texts are our own.

use zt_align::{TrackerConfig, VoiceState};

/// 60 words, 4 sentences / lines. Every whitespace token normalises to
/// exactly one word, so token id == normalised index == word position:
///
/// ```text
///  0 Willkommen  1 zu  2 meinem  3 Kanal.  4 Heute  5 zeige  6 ich  7 euch,
///  8 wie  9 man 10 einen 11 Teleprompter 12 richtig 13 nutzt.
/// 14 Zuerst 15 brauchen 16 wir 17 ein 18 gutes 19 Mikrofon 20 und 21 eine
/// 22 ruhige 23 Umgebung 24 ohne 25 störende 26 Geräusche 27 im 28 Hintergrund.
/// 29 Danach 30 öffnen 31 wir 32 die 33 Software 34 und 35 laden 36 das
/// 37 Skript 38 aus 39 einer 40 einfachen 41 Textdatei 42 von 43 der 44 Festplatte.
/// 45 Zum 46 Schluss 47 stellen 48 wir 49 die 50 Geschwindigkeit 51 ein 52 und
/// 53 beginnen 54 mit 55 der 56 eigentlichen 57 Aufnahme 58 des 59 Videos.
/// ```
pub const SCRIPT_DE: &str = "Willkommen zu meinem Kanal. Heute zeige ich euch, wie man einen Teleprompter richtig nutzt.
Zuerst brauchen wir ein gutes Mikrofon und eine ruhige Umgebung ohne störende Geräusche im Hintergrund.
Danach öffnen wir die Software und laden das Skript aus einer einfachen Textdatei von der Festplatte.
Zum Schluss stellen wir die Geschwindigkeit ein und beginnen mit der eigentlichen Aufnahme des Videos.";

pub const SCRIPT_DE_WORDS: usize = 60;

/// The four sentences of [`SCRIPT_DE`] as spoken (no punctuation).
pub const S1: &str =
    "Willkommen zu meinem Kanal Heute zeige ich euch wie man einen Teleprompter richtig nutzt";
pub const S2: &str = "Zuerst brauchen wir ein gutes Mikrofon und eine ruhige Umgebung ohne störende Geräusche im Hintergrund";
pub const S3: &str = "Danach öffnen wir die Software und laden das Skript aus einer einfachen Textdatei von der Festplatte";
pub const S4: &str = "Zum Schluss stellen wir die Geschwindigkeit ein und beginnen mit der eigentlichen Aufnahme des Videos";

/// Last token of each sentence.
pub const END_S1: usize = 13;
pub const END_S2: usize = 28;
pub const END_S3: usize = 44;
pub const END_S4: usize = 59;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Category {
    Normal,
    /// Slip of the tongue / STT typo / self-correction.
    Versprecher,
    /// Speaker goes back (small: accepted at once, large: confirmed jump).
    Zurueckspringen,
    /// Speaker skips words or sentences.
    Ueberspringen,
    /// VAD silence / pauses.
    Stille,
    /// Unscripted speech.
    AdLib,
    /// Operator intervention.
    Manual,
    /// Lost and recovered.
    Lost,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Step {
    /// Growing partials + final (see [`crate::Speaker::say`]).
    Say(&'static str),
    /// VAD silence, then this many ms.
    Silence(u64),
    Tick(u64),
    Seek(usize),
    /// Asserts `(state, cursor_token)` at this point.
    Expect(VoiceState, Option<usize>),
}

#[derive(Debug, Clone, Copy)]
pub struct Scenario {
    pub name: &'static str,
    pub category: Category,
    pub cfg: fn() -> TrackerConfig,
    pub steps: &'static [Step],
}

fn default_cfg() -> TrackerConfig {
    TrackerConfig::default()
}

/// Narrow forward window so a far target can only be found by the global search.
fn narrow_window_cfg() -> TrackerConfig {
    TrackerConfig { window_after: 20, ..TrackerConfig::default() }
}

use Step::{Expect, Say, Seek, Silence, Tick};
use VoiceState::{Hold, Lost, Tracking};

pub const SCENARIOS: &[Scenario] = &[
    Scenario {
        name: "normal: reads two sentences",
        category: Category::Normal,
        cfg: default_cfg,
        steps: &[
            Say(S1),
            Expect(Tracking, Some(END_S1)),
            Say("Zuerst brauchen wir ein gutes Mikrofon"),
            Expect(Tracking, Some(19)),
        ],
    },
    Scenario {
        name: "versprecher: STT typo (Mikrofom) is matched fuzzily",
        category: Category::Versprecher,
        cfg: default_cfg,
        steps: &[
            Say(S1),
            Say("Zuerst brauchen wir ein gutes Mikrofom und eine ruhige Umgebung"),
            Expect(Tracking, Some(23)),
        ],
    },
    Scenario {
        name: "versprecher: self-correction with filler (\"ein gutes äh ein gutes\")",
        category: Category::Versprecher,
        cfg: default_cfg,
        steps: &[
            Say(S1),
            Say("Zuerst brauchen wir ein gutes äh ein gutes Mikrofon"),
            Expect(Tracking, Some(19)),
        ],
    },
    Scenario {
        name: "zurückspringen: small backtrack accepted immediately",
        category: Category::Zurueckspringen,
        cfg: default_cfg,
        steps: &[
            Say(S1),
            Expect(Tracking, Some(END_S1)),
            Say("wie man einen Teleprompter"),
            Expect(Tracking, Some(11)),
        ],
    },
    Scenario {
        name: "zurückspringen: re-reading the first line from line 3 needs confirmation",
        category: Category::Zurueckspringen,
        cfg: default_cfg,
        steps: &[
            Say(S1),
            Say(S2),
            Say(S3),
            Expect(Tracking, Some(END_S3)),
            Say("Willkommen zu meinem Kanal Heute zeige"),
            Tick(300),
            Expect(Tracking, Some(5)),
        ],
    },
    Scenario {
        name: "überspringen: a few skipped words are followed without a jump",
        category: Category::Ueberspringen,
        cfg: default_cfg,
        steps: &[
            Say(S1),
            Say("und eine ruhige Umgebung ohne störende"),
            Expect(Tracking, Some(25)),
        ],
    },
    Scenario {
        name: "überspringen: skipped sentence within jump range",
        category: Category::Ueberspringen,
        cfg: default_cfg,
        steps: &[Say(S1), Say("Danach öffnen wir die Software"), Expect(Tracking, Some(33))],
    },
    Scenario {
        name: "überspringen: far forward jump is confirmed, then eased",
        category: Category::Ueberspringen,
        cfg: default_cfg,
        steps: &[
            Say("Willkommen zu meinem Kanal Heute zeige ich euch"),
            Expect(Tracking, Some(7)),
            Say("Danach öffnen wir die Software"),
            Tick(300),
            Expect(Tracking, Some(33)),
        ],
    },
    Scenario {
        name: "stille: silence holds, reading resumes at the cursor",
        category: Category::Stille,
        cfg: default_cfg,
        steps: &[
            Say("Willkommen zu meinem Kanal"),
            Expect(Tracking, Some(3)),
            Silence(10_000),
            Expect(Hold, Some(3)),
            Say("Heute zeige ich euch"),
            Expect(Tracking, Some(7)),
        ],
    },
    Scenario {
        name: "stille: very long silence is never `lost`",
        category: Category::Stille,
        cfg: default_cfg,
        steps: &[Say("Willkommen zu meinem Kanal"), Silence(60_000), Expect(Hold, Some(3))],
    },
    Scenario {
        name: "ad-lib: holds the text, then resumes",
        category: Category::AdLib,
        cfg: default_cfg,
        steps: &[
            Say("Willkommen zu meinem Kanal Heute zeige ich euch"),
            Say("oh sorry kurz vergessen hier etwas"),
            Expect(Hold, Some(7)),
            Say("wie man einen Teleprompter"),
            Expect(Tracking, Some(11)),
        ],
    },
    Scenario {
        name: "manual: seek overrides voice for two seconds",
        category: Category::Manual,
        cfg: default_cfg,
        steps: &[
            Say("Willkommen zu meinem Kanal"),
            Seek(29),
            Expect(Hold, Some(29)),
            Say("Heute zeige"),
            Expect(Hold, Some(29)),
            Tick(2000),
            Say("Danach öffnen wir die"),
            Expect(Tracking, Some(32)),
        ],
    },
    Scenario {
        name: "lost: unrelated speech -> lost, global search recovers",
        category: Category::Lost,
        cfg: narrow_window_cfg,
        steps: &[
            Say("Willkommen zu meinem Kanal Heute zeige ich euch"),
            Say("bla blub quux zzz"),
            Say("bla blub quux zzz"),
            Say("bla blub quux zzz"),
            Say("bla blub quux zzz"),
            Say("bla blub quux zzz"),
            Expect(Lost, Some(7)),
            Say("Zum Schluss stellen wir die Geschwindigkeit"),
            Tick(300),
            Expect(Tracking, Some(50)),
        ],
    },
];

impl Scenario {
    /// Runs all steps; panics with the scenario name and step index on mismatch.
    pub fn run(&self) -> crate::Speaker {
        let mut s = crate::Speaker::new(SCRIPT_DE, (self.cfg)());
        for (i, step) in self.steps.iter().enumerate() {
            match *step {
                Say(text) => {
                    s.say(text);
                }
                Silence(ms) => {
                    s.silence(ms);
                }
                Tick(ms) => {
                    s.tick(ms);
                }
                Seek(tok) => {
                    s.seek(tok);
                }
                Expect(state, cursor) => {
                    let o = s.out();
                    assert_eq!(
                        (o.state, o.cursor_token),
                        (state, cursor),
                        "scenario '{}' ({:?}), step #{i}: got {o:?}",
                        self.name,
                        self.category
                    );
                }
            }
        }
        s
    }
}
