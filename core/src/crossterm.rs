use crossterm::event::KeyCode;

pub fn format_keycode(
    key_code: KeyCode
) -> String {
    use crossterm::event::{
        KeyCode::*,
        ModifierKeyCode::*,
        MediaKeyCode::*,
        KeyCode as KC,
        MediaKeyCode as MKC
    };

    match key_code {
        F(n)      => return format!("F{}", n),
        Char(' ') => return "␣".to_string(),
        Char(c)   => return c.to_uppercase().to_string(),
        _         => (),
    }

    match key_code {
        Esc                      => "ESC",
        Backspace                => "⌫",
        Tab                      => "⇥",
        CapsLock                 => "CAPS",
        Enter                    => "⏎",
        Left                     => "←",
        Right                    => "→",
        Up                       => "↑",
        Down                     => "↓",
        Home                     => "Home",
        End                      => "End",
        PageUp                   => "PgUp",
        PageDown                 => "PgDn",
        BackTab                  => "⇤",
        Delete                   => "Del",
        Insert                   => "Ins",
        Null                     => "Null",
        ScrollLock               => "ScrL",
        NumLock                  => "NumLk",
        PrintScreen              => "Prnt",
        KC::Pause                => "Paus",
        Menu                     => "Menu",
        KeypadBegin              => "KP5",
        Media(Play)              => "▶️",
        Media(MKC::Pause)        => "⏸",
        Media(PlayPause)         => "⏯",
        Media(Reverse)           => "⏪",
        Media(Stop)              => "⏹",
        Media(FastForward)       => "⏩",
        Media(Rewind)            => "⏮",
        Media(TrackNext)         => "⏭",
        Media(TrackPrevious)     => "⏮",
        Media(Record)            => "⏺",
        Media(LowerVolume)       => "🔉",
        Media(RaiseVolume)       => "🔊",
        Media(MuteVolume)        => "🔇",
        Modifier(LeftShift)      => "SHIFT",
        Modifier(RightShift)     => "SHIFT",
        Modifier(LeftControl)    => "CTRL",
        Modifier(LeftSuper)      => "⌘L",
        Modifier(LeftHyper)      => "Hyp",
        Modifier(LeftMeta)       => "Meta",
        Modifier(LeftAlt)        => "ALT",
        Modifier(RightAlt)       => "ALT",
        Modifier(RightControl)   => "CTRL",
        Modifier(RightSuper)     => "⌘R",
        Modifier(RightHyper)     => "Hyp",
        Modifier(RightMeta)      => "Meta",
        Modifier(IsoLevel3Shift) => "Iso3",
        Modifier(IsoLevel5Shift) => "Iso5",
        F(_)                     => unreachable!("F key already handled"),
        Char(_)                  => unreachable!("Char already handled"),
    }.to_string()
}