//! Builders for `zt-core` test setups (one place instead of per-test boilerplate).

use zt_core::{Command, Controller, LostPolicy, ScrollMode, Settings, StatePatch, Tokens};

/// Two paragraphs, six tokens: `eins zwei drei vier | fünf sechs`.
pub const SHORT_SCRIPT: &str = "eins zwei drei vier\n\nfünf sechs";

#[derive(Debug, Clone)]
pub struct ControllerBuilder {
    tokens: Tokens,
    settings: Settings,
    script: Option<String>,
    wpm: Option<u16>,
    mode: Option<ScrollMode>,
    voice: bool,
    live: bool,
    play: bool,
}

impl Default for ControllerBuilder {
    fn default() -> Self {
        let tokens = Tokens::embedded();
        let mut settings = Settings::from_tokens(&tokens);
        settings.countdown_secs = 0; // tests start immediately unless they opt in
        Self {
            tokens,
            settings,
            script: Some(SHORT_SCRIPT.into()),
            wpm: None,
            mode: None,
            voice: false,
            live: false,
            play: false,
        }
    }
}

impl ControllerBuilder {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn script(mut self, text: &str) -> Self {
        self.script = Some(text.into());
        self
    }
    pub fn no_script(mut self) -> Self {
        self.script = None;
        self
    }
    pub fn countdown(mut self, secs: u8) -> Self {
        self.settings.countdown_secs = secs;
        self
    }
    pub fn wpm(mut self, wpm: u16) -> Self {
        self.wpm = Some(wpm);
        self
    }
    pub fn lost_policy(mut self, p: LostPolicy) -> Self {
        self.settings.lost_policy = p;
        self
    }
    pub fn settings(mut self, f: impl FnOnce(&mut Settings)) -> Self {
        f(&mut self.settings);
        self
    }
    pub fn mode(mut self, m: ScrollMode) -> Self {
        self.mode = Some(m);
        self
    }
    /// Enables voice (`ToggleVoice`), which also selects `VoiceTracking`.
    pub fn voice(mut self) -> Self {
        self.voice = true;
        self
    }
    pub fn live(mut self) -> Self {
        self.live = true;
        self
    }
    /// Starts playback (implies Live).
    pub fn playing(mut self) -> Self {
        self.play = true;
        self
    }

    pub fn build(self) -> Controller {
        let mut c = Controller::new(self.tokens, self.settings);
        let mut run = |cmd: Command| {
            c.apply(cmd.clone()).unwrap_or_else(|e| panic!("builder command {cmd:?} failed: {e}"));
        };
        if let Some(s) = self.script {
            run(Command::LoadScript(s));
        }
        if let Some(w) = self.wpm {
            run(Command::SetWpm(w));
        }
        if self.voice {
            run(Command::ToggleVoice);
        }
        if let Some(m) = self.mode {
            run(Command::SetScrollMode(m));
        }
        if self.live {
            run(Command::EnterLive);
        }
        if self.play {
            run(Command::Play);
        }
        c
    }
}

/// Applies a sequence of commands, panicking on the first error.
pub fn apply_all(c: &mut Controller, cmds: impl IntoIterator<Item = Command>) -> Vec<StatePatch> {
    cmds.into_iter()
        .map(|cmd| c.apply(cmd.clone()).unwrap_or_else(|e| panic!("{cmd:?} failed: {e}")))
        .collect()
}

/// Advances time in `frame_ms` steps for `total_ms`.
pub fn run_for(c: &mut Controller, total_ms: u32, frame_ms: u32) {
    let mut t = 0;
    while t < total_ms {
        let dt = frame_ms.min(total_ms - t);
        c.apply(Command::Tick(dt)).expect("tick never fails before the end");
        t += dt;
    }
}
