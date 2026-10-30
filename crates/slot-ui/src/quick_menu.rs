use crate::bitmaps::{ButtonLegend, button_legend};
use crate::draw::{Canvas, Draw, OUT_H, OUT_W, TexId};
use crate::icon::{Icon, WARNING_INK};
use crate::plate::{
    CAP_GAP, HINT_EDGE, LEGEND_GAP, UndoFace, blit, centred_hints, word_face, word_ink,
};
use crate::power_menu::{MENU_H, MENU_INK, MENU_PAD, MENU_PX};
use crate::slot_chrome::{MOUTH_H, opening};
use crate::text;
use slot_store::{CarouselAction, GameButton, Shortcut};

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum QuickRow {
    FastForward,
    FastForwardSound,
    Screen,
    Rumble,
    DateTime,
    About,
    GbaShader,
    GbShader,
    GbaColourCorrection,
    GbColourCorrection,
    Game,
    EjectSave,
    Turbo,
    Rewind,
    GbPalette,
    Theme,
    Wallpaper,
    Recents,
    ShowTitles,
    BootAnimation,
    CarouselSounds,
    WallpaperTint,
    GbcShader,
    GbcColourCorrection,
    Resume,
    States,
    Screenshots,
    Settings,
    ExitGame,
    PlayScreen,
    PlayGameplay,
    Shader,
    ColourCorrection,
    Palette,
    ScreenGba,
    GbBezel,
    GbcBezel,
    Bezel,
    GbaBezel,
    ScreenGbc,
    ScreenGb,
    Bluetooth,
    PlaySound,
    Power,
    Kids,
    LidClose,
    SleepTimeout,
    SleepNow,
    ShutDown,
    ShellColour,
    General,
    ShelfPrev,
    ShelfNext,
    ShelfFavorite,
    PlayControls,
    Shortcuts,
    GameUp,
    GameDown,
    GameLeft,
    GameRight,
    GameA,
    GameB,
    GameSelect,
    GameStart,
    GameL,
    GameR,
    GameTurboA,
    GameTurboB,
    ScSave,
    ScLoad,
    ScScreenshot,
    ScBrightUp,
    ScBrightDown,
    ScBlueDown,
    ScBlueUp,
    ScColour,
    ScPaletteBack,
    ScPaletteNext,
    ScShaderNext,
    ScShaderBack,
    ScFastForward,
    ScFastForwardToggle,
    ScRewind,
    ScNight,
    Controls,
    ControlsShelf,
    ControlsInGame,
    LeftStick,
    ScreenGeneral,
    Brightness,
    BlueLight,
    PlayBrightness,
    PlayBlueLight,
    AutoLoad,
    UndoSave,
    UndoLoad,
    Reference,
    PlayReference,
    FavoriteSort,
    GameDetails,
    PlayDetails,
    DisplaySchedule,
}

impl QuickRow {
    pub fn toggle(self) -> bool {
        matches!(self, Self::FastForwardSound | Self::Rumble | Self::AutoLoad | Self::EjectSave
            | Self::Turbo | Self::Rewind | Self::BootAnimation | Self::CarouselSounds
            | Self::LeftStick | Self::GbaColourCorrection | Self::GbColourCorrection
            | Self::GbcColourCorrection | Self::ColourCorrection | Self::Recents)
    }
    pub const ALL: [QuickRow; 102] = [
        QuickRow::FastForward,
        QuickRow::FastForwardSound,
        QuickRow::Screen,
        QuickRow::Rumble,
        QuickRow::DateTime,
        QuickRow::About,
        QuickRow::GbaShader,
        QuickRow::GbShader,
        QuickRow::GbaColourCorrection,
        QuickRow::GbColourCorrection,
        QuickRow::Game,
        QuickRow::EjectSave,
        QuickRow::Turbo,
        QuickRow::Rewind,
        QuickRow::GbPalette,
        QuickRow::Theme,
        QuickRow::Wallpaper,
        QuickRow::Recents,
        QuickRow::ShowTitles,
        QuickRow::BootAnimation,
        QuickRow::CarouselSounds,
        QuickRow::WallpaperTint,
        QuickRow::GbcShader,
        QuickRow::GbcColourCorrection,
        QuickRow::Resume,
        QuickRow::States,
        QuickRow::Screenshots,
        QuickRow::Settings,
        QuickRow::ExitGame,
        QuickRow::PlayScreen,
        QuickRow::PlayGameplay,
        QuickRow::Shader,
        QuickRow::ColourCorrection,
        QuickRow::Palette,
        QuickRow::ScreenGba,
        QuickRow::GbBezel,
        QuickRow::GbcBezel,
        QuickRow::Bezel,
        QuickRow::GbaBezel,
        QuickRow::ScreenGbc,
        QuickRow::ScreenGb,
        QuickRow::Bluetooth,
        QuickRow::PlaySound,
        QuickRow::Power,
        QuickRow::Kids,
        QuickRow::LidClose,
        QuickRow::SleepTimeout,
        QuickRow::SleepNow,
        QuickRow::ShutDown,
        QuickRow::ShellColour,
        QuickRow::General,
        QuickRow::ShelfPrev,
        QuickRow::ShelfNext,
        QuickRow::ShelfFavorite,
        QuickRow::PlayControls,
        QuickRow::Shortcuts,
        QuickRow::GameUp,
        QuickRow::GameDown,
        QuickRow::GameLeft,
        QuickRow::GameRight,
        QuickRow::GameA,
        QuickRow::GameB,
        QuickRow::GameSelect,
        QuickRow::GameStart,
        QuickRow::GameL,
        QuickRow::GameR,
        QuickRow::GameTurboA,
        QuickRow::GameTurboB,
        QuickRow::ScSave,
        QuickRow::ScLoad,
        QuickRow::ScScreenshot,
        QuickRow::ScBrightUp,
        QuickRow::ScBrightDown,
        QuickRow::ScBlueDown,
        QuickRow::ScBlueUp,
        QuickRow::ScColour,
        QuickRow::ScPaletteBack,
        QuickRow::ScPaletteNext,
        QuickRow::ScShaderNext,
        QuickRow::ScShaderBack,
        QuickRow::ScFastForward,
        QuickRow::ScFastForwardToggle,
        QuickRow::ScRewind,
        QuickRow::ScNight,
        QuickRow::Controls,
        QuickRow::ControlsShelf,
        QuickRow::ControlsInGame,
        QuickRow::LeftStick,
        QuickRow::ScreenGeneral,
        QuickRow::Brightness,
        QuickRow::BlueLight,
        QuickRow::PlayBrightness,
        QuickRow::PlayBlueLight,
        QuickRow::AutoLoad,
        QuickRow::UndoSave,
        QuickRow::UndoLoad,
        QuickRow::Reference,
        QuickRow::PlayReference,
        QuickRow::FavoriteSort,
        QuickRow::GameDetails,
        QuickRow::PlayDetails,
        QuickRow::DisplaySchedule,
    ];

    pub const MAIN: [QuickRow; 9] = [
        QuickRow::General,
        QuickRow::Screen,
        QuickRow::Bluetooth,
        QuickRow::Game,
        QuickRow::Power,
        QuickRow::Controls,
        QuickRow::Kids,
        QuickRow::GameDetails,
        QuickRow::About,
    ];

    pub const POWER: [QuickRow; 4] = [
        QuickRow::LidClose,
        QuickRow::SleepTimeout,
        QuickRow::SleepNow,
        QuickRow::ShutDown,
    ];

    pub const GENERAL: [QuickRow; 10] = [
        QuickRow::Wallpaper,
        QuickRow::WallpaperTint,
        QuickRow::ShellColour,
        QuickRow::Theme,
        QuickRow::Recents,
        QuickRow::ShowTitles,
        QuickRow::FavoriteSort,
        QuickRow::BootAnimation,
        QuickRow::CarouselSounds,
        QuickRow::DateTime,
    ];

    pub const CONTROL_PAGES: [QuickRow; 4] = [
        QuickRow::ControlsShelf,
        QuickRow::Shortcuts,
        QuickRow::Reference,
        QuickRow::ControlsInGame,
    ];

    pub const SHELF_KEYS: [QuickRow; 3] = [
        QuickRow::ShelfPrev,
        QuickRow::ShelfNext,
        QuickRow::ShelfFavorite,
    ];

    pub const CONTROLS: [QuickRow; 12] = [
        QuickRow::GameUp,
        QuickRow::GameDown,
        QuickRow::GameLeft,
        QuickRow::GameRight,
        QuickRow::GameA,
        QuickRow::GameB,
        QuickRow::GameSelect,
        QuickRow::GameStart,
        QuickRow::GameL,
        QuickRow::GameR,
        QuickRow::GameTurboA,
        QuickRow::GameTurboB,
    ];

    pub const SHORTCUTS: [QuickRow; 16] = [
        QuickRow::ScSave,
        QuickRow::ScLoad,
        QuickRow::ScScreenshot,
        QuickRow::ScBrightUp,
        QuickRow::ScBrightDown,
        QuickRow::ScBlueDown,
        QuickRow::ScBlueUp,
        QuickRow::ScColour,
        QuickRow::ScPaletteBack,
        QuickRow::ScPaletteNext,
        QuickRow::ScShaderNext,
        QuickRow::ScShaderBack,
        QuickRow::ScFastForward,
        QuickRow::ScFastForwardToggle,
        QuickRow::ScRewind,
        QuickRow::ScNight,
    ];

    pub const SCREEN: [QuickRow; 4] = [
        QuickRow::ScreenGeneral,
        QuickRow::ScreenGba,
        QuickRow::ScreenGbc,
        QuickRow::ScreenGb,
    ];

    pub const SCREEN_GENERAL: [QuickRow; 3] = [QuickRow::Brightness, QuickRow::BlueLight, QuickRow::DisplaySchedule];

    pub const SCREEN_GBA: [QuickRow; 3] = [
        QuickRow::GbaShader,
        QuickRow::GbaColourCorrection,
        QuickRow::GbaBezel,
    ];

    pub const SCREEN_GBC: [QuickRow; 3] = [
        QuickRow::GbcShader,
        QuickRow::GbcColourCorrection,
        QuickRow::GbcBezel,
    ];

    pub const SCREEN_GB: [QuickRow; 4] = [
        QuickRow::GbShader,
        QuickRow::GbColourCorrection,
        QuickRow::GbPalette,
        QuickRow::GbBezel,
    ];

    pub const GAME: [QuickRow; 8] = [
        QuickRow::FastForward,
        QuickRow::FastForwardSound,
        QuickRow::Rewind,
        QuickRow::Turbo,
        QuickRow::Rumble,
        QuickRow::LeftStick,
        QuickRow::AutoLoad,
        QuickRow::EjectSave,
    ];

    pub const PLAY: [QuickRow; 7] = [
        QuickRow::Resume,
        QuickRow::UndoSave,
        QuickRow::UndoLoad,
        QuickRow::States,
        QuickRow::Screenshots,
        QuickRow::Settings,
        QuickRow::ExitGame,
    ];

    pub const SETTINGS: [QuickRow; 6] = [
        QuickRow::PlayScreen,
        QuickRow::PlaySound,
        QuickRow::PlayGameplay,
        QuickRow::PlayControls,
        QuickRow::PlayReference,
        QuickRow::PlayDetails,
    ];

    pub const PLAY_SCREEN: [QuickRow; 6] = [
        QuickRow::Shader,
        QuickRow::Bezel,
        QuickRow::Palette,
        QuickRow::ColourCorrection,
        QuickRow::PlayBrightness,
        QuickRow::PlayBlueLight,
    ];

    pub fn index(self) -> usize {
        self as usize
    }

    pub fn page(self) -> &'static [QuickRow] {
        match self {
            QuickRow::LidClose
            | QuickRow::SleepTimeout
            | QuickRow::SleepNow
            | QuickRow::ShutDown => &QuickRow::POWER,
            QuickRow::ScreenGeneral
            | QuickRow::ScreenGba
            | QuickRow::ScreenGbc
            | QuickRow::ScreenGb => &QuickRow::SCREEN,
            QuickRow::Brightness | QuickRow::BlueLight | QuickRow::DisplaySchedule => &QuickRow::SCREEN_GENERAL,
            QuickRow::PlayBrightness | QuickRow::PlayBlueLight => &QuickRow::PLAY_SCREEN,

            QuickRow::GbaShader | QuickRow::GbaColourCorrection | QuickRow::GbaBezel => {
                &QuickRow::SCREEN_GBA
            }
            QuickRow::GbcShader | QuickRow::GbcColourCorrection | QuickRow::GbcBezel => {
                &QuickRow::SCREEN_GBC
            }
            QuickRow::GbShader
            | QuickRow::GbColourCorrection
            | QuickRow::GbPalette
            | QuickRow::GbBezel => &QuickRow::SCREEN_GB,
            QuickRow::FastForward
            | QuickRow::FastForwardSound
            | QuickRow::Rewind
            | QuickRow::Turbo
            | QuickRow::Rumble
            | QuickRow::LeftStick
            | QuickRow::AutoLoad
            | QuickRow::EjectSave => &QuickRow::GAME,
            QuickRow::Wallpaper
            | QuickRow::WallpaperTint
            | QuickRow::DateTime
            | QuickRow::ShellColour
            | QuickRow::Theme
            | QuickRow::Recents
            | QuickRow::ShowTitles
            | QuickRow::FavoriteSort
            | QuickRow::BootAnimation
            | QuickRow::CarouselSounds => &QuickRow::GENERAL,
            QuickRow::Reference | QuickRow::ControlsShelf | QuickRow::Shortcuts | QuickRow::ControlsInGame => {
                &QuickRow::CONTROL_PAGES
            }
            QuickRow::Resume
            | QuickRow::UndoSave
            | QuickRow::UndoLoad
            | QuickRow::States
            | QuickRow::Screenshots
            | QuickRow::Settings
            | QuickRow::ExitGame => &QuickRow::PLAY,
            QuickRow::PlayScreen
            | QuickRow::PlaySound
            | QuickRow::PlayGameplay
            | QuickRow::PlayReference
            | QuickRow::PlayDetails
            | QuickRow::PlayControls => &QuickRow::SETTINGS,

            r if r.game_button().is_some() => &QuickRow::CONTROLS,
            r if r.shortcut().is_some() => &QuickRow::SHORTCUTS,
            r if r.carousel_action().is_some() => &QuickRow::SHELF_KEYS,
            QuickRow::Shader | QuickRow::ColourCorrection | QuickRow::Palette | QuickRow::Bezel => {
                &QuickRow::PLAY_SCREEN
            }
            _ => &QuickRow::MAIN,
        }
    }

    pub fn parent(self) -> Option<QuickRow> {
        match self.page()[0] {
            QuickRow::LidClose => Some(QuickRow::Power),
            QuickRow::ScreenGeneral => Some(QuickRow::Screen),
            QuickRow::Brightness => Some(QuickRow::ScreenGeneral),
            QuickRow::ScreenGba => Some(QuickRow::Screen),

            QuickRow::GbaShader => Some(QuickRow::ScreenGba),
            QuickRow::GbcShader => Some(QuickRow::ScreenGbc),
            QuickRow::GbShader => Some(QuickRow::ScreenGb),
            QuickRow::FastForward => Some(QuickRow::Game),
            QuickRow::PlayScreen => Some(QuickRow::Settings),
            QuickRow::Shader => Some(QuickRow::PlayScreen),
            QuickRow::GameUp => Some(QuickRow::ControlsInGame),
            QuickRow::ScSave => Some(QuickRow::Shortcuts),
            QuickRow::Wallpaper => Some(QuickRow::General),
            QuickRow::ControlsShelf => Some(QuickRow::Controls),
            QuickRow::ShelfPrev => Some(QuickRow::ControlsShelf),
            _ => None,
        }
    }

    pub fn parent_in_game(self) -> Option<QuickRow> {
        match self.parent() {
            Some(QuickRow::Game) => Some(QuickRow::PlayGameplay),
            Some(QuickRow::ControlsInGame) => Some(QuickRow::PlayControls),
            parent => parent,
        }
    }

    /// The title is cached once per page, never built during an animation.
    pub fn heading(self) -> &'static str {
        match self.page()[0] {
            QuickRow::General => "Settings",
            QuickRow::Resume => "Paused",
            QuickRow::PlayScreen => "Settings",
            QuickRow::Shader => "Screen",
            _ => self.parent().map_or("Settings", QuickRow::label),
        }
    }

    pub fn centred(self) -> bool {
        [
            &QuickRow::MAIN[..],
            &QuickRow::PLAY,
            &QuickRow::SETTINGS,
            &QuickRow::SCREEN,
            &QuickRow::CONTROL_PAGES,
        ]
        .contains(&self.page())
    }

    pub fn child(self) -> Option<QuickRow> {
        match self {
            QuickRow::Power => Some(QuickRow::POWER[0]),
            QuickRow::Screen => Some(QuickRow::SCREEN[0]),
            QuickRow::Game => Some(QuickRow::GAME[0]),
            QuickRow::Settings => Some(QuickRow::SETTINGS[0]),
            QuickRow::PlayScreen => Some(QuickRow::PLAY_SCREEN[0]),
            QuickRow::PlayGameplay => Some(QuickRow::GAME[0]),
            QuickRow::PlayControls => Some(QuickRow::CONTROLS[0]),
            QuickRow::Shortcuts => Some(QuickRow::SHORTCUTS[0]),
            QuickRow::General => Some(QuickRow::GENERAL[0]),
            QuickRow::Controls => Some(QuickRow::CONTROL_PAGES[0]),
            QuickRow::ControlsShelf => Some(QuickRow::SHELF_KEYS[0]),
            QuickRow::ControlsInGame => Some(QuickRow::CONTROLS[0]),
            QuickRow::ScreenGeneral => Some(QuickRow::SCREEN_GENERAL[0]),
            QuickRow::ScreenGba => Some(QuickRow::SCREEN_GBA[0]),
            QuickRow::ScreenGbc => Some(QuickRow::SCREEN_GBC[0]),
            QuickRow::ScreenGb => Some(QuickRow::SCREEN_GB[0]),
            _ => None,
        }
    }

    pub fn shown(self, hidden: &[QuickRow]) -> Vec<QuickRow> {
        self.page()
            .iter()
            .copied()
            .filter(|r| !hidden.contains(r))
            .collect()
    }

    pub fn position(self, hidden: &[QuickRow]) -> usize {
        self.shown(hidden)
            .iter()
            .position(|r| *r == self)
            .unwrap_or(0)
    }

    pub fn label(self) -> &'static str {
        match self {
            QuickRow::Power => "Power",
            QuickRow::Kids => "Kid Mode",
            QuickRow::LidClose => "Lid Close",
            QuickRow::SleepTimeout => "Shut Down After",
            QuickRow::SleepNow => "Sleep Now",
            QuickRow::ShutDown => "Power Off",
            QuickRow::FastForward => "Fast Forward",
            QuickRow::FastForwardSound => "Fast Forward Sound",
            QuickRow::Screen => "Screen",
            QuickRow::Rumble => "Rumble",
            QuickRow::LeftStick => "Left Stick as D-Pad",
            QuickRow::ScreenGeneral => "General",
            QuickRow::Brightness | QuickRow::PlayBrightness => "Brightness",
            QuickRow::BlueLight | QuickRow::PlayBlueLight => "Blue Light",
            QuickRow::DateTime => "Date & Time",
            QuickRow::About => "About",
            QuickRow::DisplaySchedule => "Day / Night",
            QuickRow::FavoriteSort => "Sort Favorites",
            QuickRow::GameDetails | QuickRow::PlayDetails => "Game Details",
            QuickRow::Reference | QuickRow::PlayReference => "Buttons & Shortcuts",
            QuickRow::Bluetooth | QuickRow::PlaySound => "Sound",
            QuickRow::GbaShader => "Shader",
            QuickRow::GbShader => "Shader",
            QuickRow::GbcShader => "Shader",
            QuickRow::GbaColourCorrection => "Color Correction",
            QuickRow::GbColourCorrection => "Color Correction",
            QuickRow::GbcColourCorrection => "Color Correction",
            QuickRow::Game => "Gameplay",
            QuickRow::AutoLoad => "Load State on Insert",
            QuickRow::EjectSave => "Save State on Eject",
            QuickRow::Turbo => "Turbo Buttons",
            QuickRow::Rewind => "Rewind",
            QuickRow::GbPalette => "Palette",
            QuickRow::Theme => "Theme",
            QuickRow::ShellColour => "Shell Color",
            QuickRow::Wallpaper => "Wallpaper",
            QuickRow::Recents => "Recents",
            QuickRow::ShowTitles => "Game Titles",
            QuickRow::BootAnimation => "Boot Animation",
            QuickRow::CarouselSounds => "Carousel Sounds",
            QuickRow::WallpaperTint => "Wallpaper Tint",
            QuickRow::Resume => "Resume",
            QuickRow::UndoSave => "Undo Save",
            QuickRow::UndoLoad => "Undo Load",
            QuickRow::States => "States",
            QuickRow::Screenshots => "Screenshots",
            QuickRow::Settings => "Settings",
            QuickRow::ExitGame => "Exit Game",
            QuickRow::General => "General",
            QuickRow::Controls => "Controls",
            QuickRow::ControlsShelf => "Shelf",
            QuickRow::ControlsInGame => "In-Game",
            QuickRow::PlayControls => "Controls",
            QuickRow::Shortcuts => "Shortcuts",
            QuickRow::GameUp
            | QuickRow::GameDown
            | QuickRow::GameLeft
            | QuickRow::GameRight
            | QuickRow::GameA
            | QuickRow::GameB
            | QuickRow::GameSelect
            | QuickRow::GameStart
            | QuickRow::GameL
            | QuickRow::GameR
            | QuickRow::GameTurboA
            | QuickRow::GameTurboB => self.game_button().map_or("", GameButton::label),
            QuickRow::ScSave
            | QuickRow::ScLoad
            | QuickRow::ScScreenshot
            | QuickRow::ScBrightUp
            | QuickRow::ScBrightDown
            | QuickRow::ScBlueDown
            | QuickRow::ScBlueUp
            | QuickRow::ScColour
            | QuickRow::ScPaletteBack
            | QuickRow::ScPaletteNext
            | QuickRow::ScShaderNext
            | QuickRow::ScShaderBack
            | QuickRow::ScFastForward
            | QuickRow::ScFastForwardToggle
            | QuickRow::ScRewind
            | QuickRow::ScNight => self.shortcut().map_or("", Shortcut::label),
            QuickRow::ShelfPrev | QuickRow::ShelfNext | QuickRow::ShelfFavorite => {
                self.carousel_action().map_or("", CarouselAction::label)
            }
            QuickRow::PlayScreen => "Screen",
            QuickRow::PlayGameplay => "Gameplay",
            QuickRow::Shader => "Shader",
            QuickRow::ColourCorrection => "Color Correction",
            QuickRow::Palette => "Palette",
            QuickRow::ScreenGba => "Game Boy Advance",
            QuickRow::ScreenGbc => "Game Boy Color",
            QuickRow::ScreenGb => "Game Boy",
            QuickRow::GbBezel => "Bezel",
            QuickRow::GbcBezel => "Bezel",
            QuickRow::GbaBezel => "Bezel",
            QuickRow::Bezel => "Bezel",
        }
    }

    pub fn opens(self) -> bool {
        matches!(
            self,
            QuickRow::Power
                | QuickRow::Kids
                | QuickRow::SleepNow
                | QuickRow::ShutDown
                | QuickRow::DateTime
                | QuickRow::Bluetooth
                | QuickRow::PlaySound
                | QuickRow::About
                | QuickRow::Reference
                | QuickRow::PlayReference
                | QuickRow::DisplaySchedule
                | QuickRow::GameDetails
                | QuickRow::PlayDetails
                | QuickRow::Screen
                | QuickRow::Game
                | QuickRow::Resume
                | QuickRow::UndoSave
                | QuickRow::UndoLoad
                | QuickRow::States
                | QuickRow::Screenshots
                | QuickRow::Settings
                | QuickRow::ExitGame
                | QuickRow::PlayScreen
                | QuickRow::PlayGameplay
                | QuickRow::General
                | QuickRow::Controls
                | QuickRow::ControlsShelf
                | QuickRow::ControlsInGame
                | QuickRow::PlayControls
                | QuickRow::Shortcuts
                | QuickRow::ScreenGba
                | QuickRow::ScreenGbc
                | QuickRow::ScreenGb
                | QuickRow::ScreenGeneral
        )
    }

    pub fn game_button(self) -> Option<GameButton> {
        Some(match self {
            QuickRow::GameUp => GameButton::Up,
            QuickRow::GameDown => GameButton::Down,
            QuickRow::GameLeft => GameButton::Left,
            QuickRow::GameRight => GameButton::Right,
            QuickRow::GameA => GameButton::A,
            QuickRow::GameB => GameButton::B,
            QuickRow::GameSelect => GameButton::Select,
            QuickRow::GameStart => GameButton::Start,
            QuickRow::GameL => GameButton::L,
            QuickRow::GameR => GameButton::R,
            QuickRow::GameTurboA => GameButton::TurboA,
            QuickRow::GameTurboB => GameButton::TurboB,
            _ => return None,
        })
    }

    pub fn shortcut(self) -> Option<Shortcut> {
        Some(match self {
            QuickRow::ScSave => Shortcut::SaveState,
            QuickRow::ScLoad => Shortcut::LoadState,
            QuickRow::ScScreenshot => Shortcut::Screenshot,
            QuickRow::ScBrightUp => Shortcut::BrightnessUp,
            QuickRow::ScBrightDown => Shortcut::BrightnessDown,
            QuickRow::ScBlueDown => Shortcut::BlueLightDown,
            QuickRow::ScBlueUp => Shortcut::BlueLightUp,
            QuickRow::ScColour => Shortcut::ColorCorrection,
            QuickRow::ScPaletteBack => Shortcut::PaletteBack,
            QuickRow::ScPaletteNext => Shortcut::PaletteNext,
            QuickRow::ScShaderNext => Shortcut::ShaderNext,
            QuickRow::ScShaderBack => Shortcut::ShaderBack,
            QuickRow::ScFastForward => Shortcut::FastForward,
            QuickRow::ScFastForwardToggle => Shortcut::FastForwardToggle,
            QuickRow::ScRewind => Shortcut::Rewind,
            QuickRow::ScNight => Shortcut::NightToggle,
            _ => return None,
        })
    }

    pub fn carousel_action(self) -> Option<CarouselAction> {
        Some(match self {
            QuickRow::ShelfPrev => CarouselAction::PreviousShelf,
            QuickRow::ShelfNext => CarouselAction::NextShelf,
            QuickRow::ShelfFavorite => CarouselAction::Favorite,
            _ => return None,
        })
    }

    pub fn binds(self) -> bool {
        self.game_button().is_some()
            || self.shortcut().is_some()
            || self.carousel_action().is_some()
    }

    pub fn up(self, hidden: &[QuickRow]) -> QuickRow {
        let rows = self.shown(hidden);
        rows[(self.position(hidden) + rows.len() - 1) % rows.len()]
    }

    pub fn down(self, hidden: &[QuickRow]) -> QuickRow {
        let rows = self.shown(hidden);
        rows[(self.position(hidden) + 1) % rows.len()]
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum QuickValue {
    Speed1_5,
    Speed2,
    Speed3,
    Speed4,
    Speed6,
    On,
    Off,
    Lcd3x,
    Grid,
    Dot,
    Simpletex,
    Dark,
    Light,
    Show,
    Hide,
    Low,
    Medium,
    High,
    LightSleep,
    DeepSleep,
    PowerOff,
    Minutes3,
    Minutes15,
    Minutes30,
    Never,
    MatchTheme,
    Graphite,
    Platinum,
    PearlBlue,
    PearlPink,
    Minutes1,

    Minutes5,
    Minutes10,
    Level0,
    Level1,
    Level2,
    Level3,
    Level4,
    Level5,
    Level6,
    Level7,
    Level8,
    Level9,
    Clean,
    Filename,
    Alphabetical,
    MostUsed,
    RecentlyUsed,
}

impl QuickValue {
    pub const ALL: [QuickValue; 48] = [
        QuickValue::Speed1_5,
        QuickValue::Speed2,
        QuickValue::Speed3,
        QuickValue::Speed4,
        QuickValue::Speed6,
        QuickValue::On,
        QuickValue::Off,
        QuickValue::Lcd3x,
        QuickValue::Grid,
        QuickValue::Dot,
        QuickValue::Simpletex,
        QuickValue::Dark,
        QuickValue::Light,
        QuickValue::Show,
        QuickValue::Hide,
        QuickValue::Low,
        QuickValue::Medium,
        QuickValue::High,
        QuickValue::LightSleep,
        QuickValue::DeepSleep,
        QuickValue::PowerOff,
        QuickValue::Minutes3,
        QuickValue::Minutes15,
        QuickValue::Minutes30,
        QuickValue::Never,
        QuickValue::MatchTheme,
        QuickValue::Graphite,
        QuickValue::Platinum,
        QuickValue::PearlBlue,
        QuickValue::PearlPink,
        QuickValue::Minutes1,
        QuickValue::Minutes5,
        QuickValue::Minutes10,
        QuickValue::Level0,
        QuickValue::Level1,
        QuickValue::Level2,
        QuickValue::Level3,
        QuickValue::Level4,
        QuickValue::Level5,
        QuickValue::Level6,
        QuickValue::Level7,
        QuickValue::Level8,
        QuickValue::Level9,
        QuickValue::Clean,
        QuickValue::Filename,
        QuickValue::Alphabetical,
        QuickValue::MostUsed,
        QuickValue::RecentlyUsed,
    ];

    pub const LEVELS: [QuickValue; 10] = [
        QuickValue::Level0,
        QuickValue::Level1,
        QuickValue::Level2,
        QuickValue::Level3,
        QuickValue::Level4,
        QuickValue::Level5,
        QuickValue::Level6,
        QuickValue::Level7,
        QuickValue::Level8,
        QuickValue::Level9,
    ];

    pub const LID_OFF: [QuickValue; 6] = [
        QuickValue::Minutes1,
        QuickValue::Minutes3,
        QuickValue::Minutes5,
        QuickValue::Minutes10,
        QuickValue::Minutes15,
        QuickValue::Minutes30,
    ];

    pub fn index(self) -> usize {
        self as usize
    }

    pub fn text(self) -> &'static str {
        match self {
            QuickValue::Speed1_5 => "1.5×",
            QuickValue::Speed2 => "2×",
            QuickValue::Speed3 => "3×",
            QuickValue::Speed4 => "4×",
            QuickValue::Speed6 => "6×",
            QuickValue::On => "On",
            QuickValue::Off => "Off",
            QuickValue::Lcd3x => "LCD3x",
            QuickValue::Grid => "Grid",
            QuickValue::Dot => "Dot",
            QuickValue::Simpletex => "Simpletex",
            QuickValue::Dark => "Dark",
            QuickValue::Light => "Light",
            QuickValue::Show => "Show",
            QuickValue::Hide => "Hide",
            QuickValue::Low => "25%",
            QuickValue::Medium => "45%",
            QuickValue::High => "65%",
            QuickValue::LightSleep => "Light Sleep",
            QuickValue::DeepSleep => "Deep Sleep",
            QuickValue::PowerOff => "Power Off",
            QuickValue::Minutes3 => "3 min",
            QuickValue::Minutes15 => "15 min",
            QuickValue::Minutes30 => "30 min",
            QuickValue::Never => "Never",
            QuickValue::MatchTheme => "Match Theme",
            QuickValue::Graphite => "Graphite",
            QuickValue::Platinum => "Platinum",
            QuickValue::PearlBlue => "Pearl Blue",
            QuickValue::PearlPink => "Pearl Pink",
            QuickValue::Minutes1 => "1 min",

            QuickValue::Minutes5 => "5 min",
            QuickValue::Minutes10 => "10 min",
            QuickValue::Level0 => "0",
            QuickValue::Level1 => "1",
            QuickValue::Level2 => "2",
            QuickValue::Level3 => "3",
            QuickValue::Level4 => "4",
            QuickValue::Level5 => "5",
            QuickValue::Level6 => "6",
            QuickValue::Level7 => "7",
            QuickValue::Level8 => "8",
            QuickValue::Level9 => "9",
            QuickValue::Clean => "Clean",
            QuickValue::Filename => "Filename",
            QuickValue::Alphabetical => "Alphabetically",
            QuickValue::MostUsed => "Most used",
            QuickValue::RecentlyUsed => "Recently used",

        }
    }

    pub fn level(n: u8) -> QuickValue {
        QuickValue::LEVELS[usize::from(n).min(QuickValue::LEVELS.len() - 1)]
    }

    pub fn speed(quarters: u8) -> Option<QuickValue> {
        match quarters {
            0 => Some(QuickValue::Off),
            6 => Some(QuickValue::Speed1_5),
            8 => Some(QuickValue::Speed2),
            12 => Some(QuickValue::Speed3),
            16 => Some(QuickValue::Speed4),
            24 => Some(QuickValue::Speed6),
            _ => None,
        }
    }

    pub fn flag(on: bool) -> QuickValue {
        if on { QuickValue::On } else { QuickValue::Off }
    }
}

pub const QUICK_PITCH: f32 = 52.0;
pub const MENU_HINT_H: u32 = 32;
pub const QUICK_VISIBLE_ROWS: usize = 5;
pub const QUICK_EDGE: f32 = 80.0;
pub const QUICK_LABEL_X: f32 = 80.0;
const POINTER_SCALE: usize = 4;
const POINTER: [&str; 11] = [
    "X.......", "XX......", "XoX.....", "XooX....", "XoooX...", "XooooX..", "XoooX...", "XooX....",
    "XoX.....", "XX......", "X.......",
];
const POINTER_BLEED: u32 = 4;
const GROUND_SCRIM: f32 = 0.65;
const TYPE_DROP: f32 = 1.0;
const CARET_GAP: f32 = 14.0;
const LEGEND_Y: f32 = OUT_H as f32 - MOUTH_H - MENU_HINT_H as f32 - 12.0;
const DIM_INK: [u8; 3] = [0x9a, 0x9a, 0xa4];
const DESCRIPTION_PX: f32 = 16.0;
const DESCRIPTION_GAP: f32 = 8.0;
const PANEL_GAP: f32 = 12.0;
const WARNING_PX: f32 = 22.0;
const WARNING_GAP: u32 = 10;

pub fn quick_label_face(row: QuickRow, lit: bool) -> UndoFace {
    quick_text_face(&menu_sentence_case(row.label()), if lit { MENU_INK } else { DIM_INK }, if lit { text::Role::Focus } else { text::Role::Menu })
}

fn cap_centre(label: &fontdue::Font, h: u32) -> f32 {
    match label.horizontal_line_metrics(MENU_PX) {
        Some(v) => {
            (h as f32 - v.new_line_size) / 2.0 + v.ascent
                - label.metrics('H', MENU_PX).height as f32 / 2.0
        }
        None => h as f32 / 2.0,
    }
}

pub fn quick_pointer_face() -> UndoFace {
    let s = POINTER_SCALE;
    let w = POINTER[0].len() * s;
    let h = (MENU_H + 2 * POINTER_BLEED) as usize;
    let centre = match text::font(text::Role::Focus) {
        Some(label) => cap_centre(label, MENU_H),
        None => MENU_H as f32 / 2.0,
    } + POINTER_BLEED as f32;
    let tip = POINTER.len() / 2 * s + s / 2;
    let top = (centre - tip as f32).round() as usize;
    let mut rgba = vec![0u8; w * h * 4];
    for (y, row) in POINTER.iter().enumerate() {
        for (x, _) in row.chars().enumerate().filter(|(_, ch)| *ch != '.') {
            for py in top + y * s..top + (y + 1) * s {
                for px in x * s..(x + 1) * s {
                    let at = (py * w + px) * 4;
                    rgba[at..at + 4].copy_from_slice(&[
                        MENU_INK[0],
                        MENU_INK[1],
                        MENU_INK[2],
                        0xff,
                    ]);
                }
            }
        }
    }
    UndoFace {
        rgba,
        w: w as u32,
        h: h as u32,
    }
}

pub fn quick_value_face(text: &str, lit: bool) -> UndoFace {
    let role = if text.chars().next().is_some_and(|c| c.is_ascii_digit()) || text.contains(':') { text::Role::Value }
        else if lit { text::Role::Focus } else { text::Role::Menu };
    quick_text_face(text, if lit { MENU_INK } else { DIM_INK }, role)
}

pub fn quick_heading_face(label: &str) -> UndoFace {
    quick_text_face(&menu_sentence_case(label), MENU_INK, text::Role::Display)
}

pub fn quick_warning_value_face(text: &str, lit: bool) -> UndoFace {
    let words = quick_value_face(text, lit);
    let (Some((cov, iw, ih)), Some(label)) = (
        crate::icon::icon_coverage(Icon::Alert, WARNING_PX),
        text::label_font(),
    ) else {
        return words;
    };
    let (w, h) = (iw + WARNING_GAP + words.w, words.h);
    let mut rgba = vec![0u8; (w * h * 4) as usize];
    let top = (cap_centre(label, h) - ih as f32 / 2.0).round().max(0.0) as u32;
    blit(
        &mut rgba,
        w,
        &words.rgba,
        words.w,
        words.h,
        iw + WARNING_GAP,
        0,
    );
    for gy in 0..ih.min(h.saturating_sub(top)) {
        for gx in 0..iw {
            let a = cov[(gy * iw + gx) as usize];
            if a == 0 {
                continue;
            }
            let at = (((top + gy) * w + MENU_PAD + gx) * 4) as usize;
            rgba[at..at + 4].copy_from_slice(&[WARNING_INK[0], WARNING_INK[1], WARNING_INK[2], a]);
        }
    }
    UndoFace { rgba, w, h }
}

fn quick_text_face(label: &str, colour: [u8; 3], role: text::Role) -> UndoFace {
    let Some(font) = text::font(role) else {
        return UndoFace {
            rgba: Vec::new(),
            w: 0,
            h: 0,
        };
    };
    let layout = text::Layout { lines: vec![label.to_string()],
        px: if role == text::Role::Display { MENU_PX } else { 26.0 }, tracking: 0.0, role };
    let set = layout
        .lines
        .iter()
        .map(|l| text::line_width(font, l, layout.px, layout.tracking))
        .fold(0.0, f32::max);
    let w = set.ceil() as u32 + 2 * MENU_PAD;
    let mut rgba = vec![0u8; (w * MENU_H * 4) as usize];
    text::draw_centred(&mut rgba, w, MENU_H, &layout, colour);
    UndoFace { rgba, w, h: MENU_H }
}

pub fn quick_caret_face(right: bool) -> UndoFace {
    // Small vector chevrons avoid adding glyphs to the symbols-font subset.
    let (w, h) = (14, MENU_H);
    let mut rgba = vec![0; (w * h * 4) as usize];
    for y in 0..h {
        for x in 0..w {
            let px = if right { x as f32 + 0.5 } else { w as f32 - x as f32 - 0.5 };
            let py = y as f32 + 0.5;
            let distance = |ay: f32, by: f32| {
                let (dx, dy) = (6.0, by - ay);
                let t = (((px - 4.0) * dx + (py - ay) * dy) / (dx * dx + dy * dy)).clamp(0.0, 1.0);
                ((px - 4.0 - t * dx).powi(2) + (py - ay - t * dy).powi(2)).sqrt()
            };
            let a = ((1.9 - distance(14.0, 20.0).min(distance(26.0, 20.0))).clamp(0.0, 1.0) * 255.0).round() as u8;
            let at = ((y * w + x) * 4) as usize;
            rgba[at..at+4].copy_from_slice(&[MENU_INK[0], MENU_INK[1], MENU_INK[2], a]);
        }
    }
    UndoFace { rgba, w, h }
}

pub fn quick_legend_faces(ink: [u8; 3]) -> [UndoFace; 8] {
    let hint = Some(ink);
    [
        with_hint(button_face(ButtonLegend::B), "Back", hint, true),
        with_hint(
            button_pair(ButtonLegend::Left, ButtonLegend::Right),
            "Change",
            hint,
            false,
        ),
        with_hint(button_face(ButtonLegend::A), "Select", hint, false),
        with_hint(button_face(ButtonLegend::B), "Resume", hint, true),
        with_hint(button_face(ButtonLegend::X), "Restart", hint, false),
        with_hint(button_face(ButtonLegend::A), "Set", hint, false),
        with_hint(button_face(ButtonLegend::X), "Clear", hint, false),
        with_hint(button_face(ButtonLegend::Y), "Reset All", hint, true),
    ]
}

/// Sound uses the same bitmap buttons and typography as the other menus.
/// Back goes on the left; Forget and Connect/Select/Change on the right.
pub fn sound_button_legends(ink: [u8; 3]) -> [UndoFace; 5] {
    let ink = Some(ink);
    [
        with_hint(button_face(ButtonLegend::B), "Back", ink, true),
        with_hint(button_face(ButtonLegend::A), "Connect", ink, false),
        with_hint(button_face(ButtonLegend::Y), "Forget", ink, false),
        with_hint(button_face(ButtonLegend::A), "Select", ink, false),
        with_hint(
            button_pair(ButtonLegend::Left, ButtonLegend::Right),
            "Change",
            ink,
            false,
        ),
    ]
}

/// Parent controls share the production bitmap legends and footer typography.
pub fn parent_button_legends(page: u8, ink: [u8; 3]) -> Vec<UndoFace> {
    let hint = Some(ink);
    match page {
        6 => vec![
            with_hint(button_face(ButtonLegend::B), "Back", hint, true),
            with_hint(button_pair(ButtonLegend::Left, ButtonLegend::Right), "Page", hint, false),
        ],
        1 => vec![
            with_hint(button_face(ButtonLegend::Menu), "Cancel", hint, true),
            with_hint(button_face(ButtonLegend::Select), "Delete", hint, false),
        ],
        2 => vec![
            with_hint(button_face(ButtonLegend::B), "Games", hint, true),
            with_hint(button_face(ButtonLegend::A), "Parents", hint, false),
        ],
        5 => vec![
            with_hint(button_face(ButtonLegend::B), "Exit game", hint, true),
            with_hint(button_face(ButtonLegend::A), "Continue", hint, false),
        ],
        4 => vec![
            with_hint(button_face(ButtonLegend::B), "Cancel", hint, true),
            with_hint(button_face(ButtonLegend::A), "Again to remove", hint, false),
        ],
        3 => vec![
            with_hint(button_face(ButtonLegend::B), "Back", hint, true),
            with_hint(button_face(ButtonLegend::A), "Change", hint, false),
        ],
        _ => vec![
            with_hint(button_face(ButtonLegend::B), "Back", hint, true),
            with_hint(button_pair(ButtonLegend::Left, ButtonLegend::Right), "Change", hint, false),
            with_hint(button_face(ButtonLegend::A), "Select", hint, false),
        ],
    }
}

/// Transparent parent menu raster, using the standard menu type, pointer,
/// spacing, carets and bitmap footer. The app draws its normal menu ground.
pub fn parent_settings_face(rows: &[(String, String, bool)], selected: usize,
    footer: u8, status: &str) -> UndoFace {
    parent_settings_face_with_space(rows, selected, footer, status, 0.0)
}

pub fn parent_settings_face_with_space(rows: &[(String, String, bool)], selected: usize,
    footer: u8, status: &str, reserve: f32) -> UndoFace {
    parent_settings_face_on(Canvas::FRAMED, rows, selected, footer, status, reserve)
}
pub fn parent_settings_face_on(canvas: Canvas, rows: &[(String,String,bool)], selected: usize, footer: u8, status: &str, reserve: f32) -> UndoFace {
    let width=canvas.w as u32;
    let mut face = parent_footer_face_on(canvas,footer,rows.get(selected).is_some_and(|r|r.2));
    let visible = rows.len().min(QUICK_VISIBLE);
    let start = selected.saturating_sub(visible / 2).min(rows.len().saturating_sub(visible));
    let top = 88.0 - reserve.max(0.0) / 2.0;
    crate::expressive::raster_menu_rows(&mut face, top, QUICK_PITCH, visible, Some(selected - start));
    let pad = MENU_PAD as f32;
    let right = width as f32 - QUICK_EDGE;
    for (i, (label, value, adjustable)) in rows.iter().enumerate().skip(start).take(visible) {
        let lit = i == selected;
        let y = top + (i - start) as f32 * QUICK_PITCH + TYPE_DROP;
        let label = parent_label_face(label, lit, !value.is_empty(), width);
        parent_blit(&mut face, &label, QUICK_LABEL_X - pad, y);
        if value.is_empty() { continue; }
        let toggle = matches!(value.as_str(), "On" | "Off");
        let value = if toggle { crate::toggle_face(value == "On", lit) }
            else { quick_value_face(value, lit) };
        if toggle {
            parent_blit(&mut face, &value, right + pad - value.w as f32, y);
            continue;
        }
        if lit && *adjustable {
            let left = quick_caret_face(false);
            let next = quick_caret_face(true);
            let rx = right - next.w as f32;
            let vx = rx - CARET_GAP + pad - value.w as f32;
            parent_blit(&mut face, &next, rx, y);
            parent_blit(&mut face, &value, vx, y);
            parent_blit(&mut face, &left, vx + pad - CARET_GAP - left.w as f32, y);
        } else {
            parent_blit(&mut face, &value, right + pad - value.w as f32, y);
        }
    }
    if rows.len() > visible {
        let h = QUICK_PITCH * visible as f32;
        let thumb = h * visible as f32 / rows.len() as f32;
        let y = top + (h - thumb) * selected as f32 / (rows.len() - 1) as f32;
        for yy in y as u32..(y + thumb) as u32 {
            for x in width - 44..width - 40 {
                let at = ((yy * width + x) * 4) as usize;
                face.rgba[at..at + 4].copy_from_slice(&[246, 244, 239, 255]);
            }
        }
    }
    if !status.is_empty() {
        let description = quick_description_face(&[status]);
        parent_blit(&mut face, &description, (width - description.w) as f32 / 2.0,
            LEGEND_Y - description.h as f32 - DESCRIPTION_GAP);
    }
    face
}

pub fn parent_footer_face(page: u8, adjustable: bool) -> UndoFace {
    parent_footer_face_on(Canvas::FRAMED,page,adjustable)
}
pub fn parent_footer_face_on(canvas: Canvas, page: u8, adjustable: bool) -> UndoFace {
    let width=canvas.w as u32;
    let mut face = UndoFace { rgba: vec![0; (width * OUT_H * 4) as usize], w: width, h: OUT_H };
    let hints = parent_button_legends(page, MENU_INK);
    parent_blit(&mut face, &hints[0], 16.0, LEGEND_Y);
    let right: Vec<_> = if page == 0 {
        vec![&hints[if adjustable { 1 } else { 2 }]]
    } else { hints[1..].iter().collect() };
    let total = right.iter().map(|h| h.w - HINT_EDGE).sum::<u32>()
        + 16 * right.len().saturating_sub(1) as u32;
    let mut x = width.saturating_sub(16 + total);
    for hint in right {
        parent_blit(&mut face, hint, x as f32, LEGEND_Y);
        x += hint.w - HINT_EDGE + 16;
    }
    face
}
pub fn kid_time_text(seconds: u64) -> String {
    if seconds == 0 { return "Time is up".into(); }
    if seconds < 60 { return "Less than a minute left".into(); }
    let minutes = seconds.div_ceil(60);
    if minutes >= 60 {
        let (hours, mins) = (minutes / 60, minutes % 60);
        if mins != 0 { return format!("{hours} h {mins} min left"); }
        return format!("{hours} hour{} left", if hours == 1 { "" } else { "s" });
    }
    format!("{minutes} minute{} left", if minutes == 1 { "" } else { "s" })
}

/// One quiet line of remaining time and a bar, above the native menu footer.
pub fn kid_time_meter(seconds: u64, budget: u64) -> UndoFace {
    let label = quick_description_face(&[&kid_time_text(seconds)]);
    let (bar_w, h) = (160, label.h.max(24));
    let w = label.w + 16 + bar_w;
    let mut face = UndoFace { rgba: vec![0; (w * h * 4) as usize], w, h };
    parent_blit(&mut face, &label, 0.0, ((h - label.h) / 2) as f32);
    let filled = ((bar_w as u64 * seconds.min(budget)) / budget.max(1)) as u32;
    for y in h / 2 - 3..h / 2 + 3 {
        for x in 0..bar_w {
            let at = ((y * w + label.w + 16 + x) * 4) as usize;
            let alpha = if x < filled { 255 } else { 64 };
            face.rgba[at..at + 4].copy_from_slice(&[246, 244, 239, alpha]);
        }
    }
    face
}

fn parent_label_face(label: &str, lit: bool, has_value: bool, width: u32) -> UndoFace {
    let Some(font) = text::font(if lit { text::Role::Focus } else { text::Role::Menu }) else { return quick_value_face(label, lit); };
    let max = if has_value { width as f32 - 290.0 } else { width as f32 - 160.0 };
    let layout = text::fit_preserving_case(font, &menu_sentence_case(label), max, 1, 26.0, 26.0);
    let width = layout.lines.first().map_or(0.0, |line|
        text::line_width(font, line, layout.px, layout.tracking)).ceil() as u32 + 2 * MENU_PAD;
    let mut rgba = vec![0; (width * MENU_H * 4) as usize];
    text::draw_centred(&mut rgba, width, MENU_H, &layout, if lit { MENU_INK } else { DIM_INK });
    UndoFace { rgba, w: width, h: MENU_H }
}
fn parent_blit(dst: &mut UndoFace, src: &UndoFace, x: f32, y: f32) {
    let (left, top) = (x.round().max(0.0) as u32, y.round().max(0.0) as u32);
    for yy in 0..src.h.min(dst.h.saturating_sub(top)) {
        for xx in 0..src.w.min(dst.w.saturating_sub(left)) {
            let s = ((yy * src.w + xx) * 4) as usize;
            let d = (((top + yy) * dst.w + left + xx) * 4) as usize;
            let a = u32::from(src.rgba[s + 3]);
            let behind = u32::from(dst.rgba[d + 3]) * (255 - a) / 255;
            let alpha = a + behind;
            if alpha == 0 { continue; }
            for c in 0..3 {
                dst.rgba[d + c] = ((u32::from(src.rgba[s + c]) * a
                    + u32::from(dst.rgba[d + c]) * behind) / alpha) as u8;
            }
            dst.rgba[d + 3] = alpha as u8;
        }
    }
}

pub fn sound_forget_confirmation() -> UndoFace {
    with_hint(
        button_face(ButtonLegend::Y),
        "Press again to forget",
        Some(MENU_INK),
        true,
    )
}

pub fn quick_text_centre() -> f32 {
    text::font(text::Role::Focus)
        .map(|f| cap_centre(f, MENU_H))
        .unwrap_or(MENU_H as f32 / 2.)
}

pub fn quick_description_face(lines: &[&str]) -> UndoFace {
    let Some(font) = text::font(text::Role::Detail) else {
        return UndoFace {
            rgba: Vec::new(),
            w: 0,
            h: 0,
        };
    };
    let line_h = (DESCRIPTION_PX * 1.4).ceil() as u32;
    let faces: Vec<(Vec<u8>, u32)> = lines
        .iter()
        .map(|line| {
            let layout = text::fit_preserving_case(font, line, OUT_W as f32, 1, DESCRIPTION_PX, DESCRIPTION_PX);
            let w = text::line_width(font, &layout.lines[0], layout.px, layout.tracking).ceil()
                as u32
                + 2;
            let mut rgba = vec![0u8; (w * line_h * 4) as usize];
            text::draw_centred(&mut rgba, w, line_h, &layout, DIM_INK);
            (rgba, w)
        })
        .collect();
    let w = faces.iter().map(|(_, w)| *w).max().unwrap_or(0);
    let h = line_h * faces.len() as u32;
    let mut rgba = vec![0u8; (w * h * 4) as usize];
    for (i, (face, fw)) in faces.iter().enumerate() {
        crate::plate::blit(
            &mut rgba,
            w,
            face,
            *fw,
            line_h,
            (w - fw) / 2,
            i as u32 * line_h,
        );
    }
    UndoFace { rgba, w, h }
}

/// Cancel, Swap, Choose. Swap keeps the button before the word, the way the old arrows did.
pub fn core_button_legends() -> Vec<UndoFace> {
    vec![
        with_hint(button_face(ButtonLegend::B), "Cancel", None, true),
        with_hint(
            button_pair(ButtonLegend::Left, ButtonLegend::Right),
            "Swap",
            None,
            true,
        ),
        with_hint(button_face(ButtonLegend::A), "Choose", None, false),
    ]
}

/// Cancel, Mode, Swap, Link, OK, Back, End Link.
pub fn link_button_legends(ink: [u8; 3]) -> Vec<UndoFace> {
    let hint = Some(ink);
    vec![
        with_hint(button_face(ButtonLegend::B), "Cancel", hint, true),
        with_hint(button_face(ButtonLegend::Select), "Mode", hint, true),
        with_hint(
            button_pair(ButtonLegend::Left, ButtonLegend::Right),
            "Swap",
            hint,
            false,
        ),
        with_hint(button_face(ButtonLegend::A), "Link", hint, false),
        with_hint(button_face(ButtonLegend::A), "OK", hint, false),
        with_hint(button_face(ButtonLegend::B), "Back", hint, true),
        with_hint(button_face(ButtonLegend::A), "End Link", hint, false),
    ]
}

pub fn repair_legends(ink: [u8; 3]) -> [UndoFace; 2] {
    prompt_legends(ink, "Not now", "Repair")
}

pub fn delete_legends(ink: [u8; 3]) -> [UndoFace; 2] {
    let hint = Some(ink);
    [
        with_hint(button_face(ButtonLegend::B), "Keep", hint, true),
        with_hint(button_face(ButtonLegend::X), "Delete", hint, false),
    ]
}

pub fn prompt_legends(ink: [u8; 3], no: &str, yes: &str) -> [UndoFace; 2] {
    let hint = Some(ink);
    [
        with_hint(button_face(ButtonLegend::B), no, hint, true),
        with_hint(button_face(ButtonLegend::A), yes, hint, false),
    ]
}

/// `hint_on_right` puts the word after the button, for a legend parked on the left of the footer.
fn with_hint(button: UndoFace, label: &str, ink: Option<[u8; 3]>, hint_on_right: bool) -> UndoFace {
    let word = match ink {
        Some(ink) => word_ink(label, ink),
        None => word_face(label),
    };
    let w = button.w + CAP_GAP + word.w + HINT_EDGE;
    let h = MENU_HINT_H;
    let mut rgba = vec![0u8; (w * h * 4) as usize];
    let text_y = h.saturating_sub(word.h) / 2;
    let (button_x, text_x) = match hint_on_right {
        true => (0, button.w + CAP_GAP),
        false => (word.w + CAP_GAP, 0),
    };
    blit(&mut rgba, w, &button.rgba, button.w, button.h, button_x, 0);
    blit(&mut rgba, w, &word.rgba, word.w, word.h, text_x, text_y);
    UndoFace { rgba, w, h }
}

pub(crate) fn button_face(which: ButtonLegend) -> UndoFace {
    let (rgba, w, h) = button_legend(which);
    let target_h = MENU_HINT_H;
    let target_w = (w as f32 * target_h as f32 / h as f32).round() as u32;
    UndoFace {
        rgba: crate::art::cover_rgba(rgba, w, h, target_w, target_h).expect("button dimensions"),
        w: target_w,
        h: target_h,
    }
}

fn button_pair(left: ButtonLegend, right: ButtonLegend) -> UndoFace {
    let a = button_face(left);
    let b = button_face(right);
    let (aw, ah, bw, bh) = (a.w, a.h, b.w, b.h);
    let (w, h) = (aw + bw, ah.max(bh));
    let mut rgba = vec![0u8; (w * h * 4) as usize];
    blit(&mut rgba, w, &a.rgba, aw, ah, 0, 0);
    blit(&mut rgba, w, &b.rgba, bw, bh, aw, 0);
    UndoFace { rgba, w, h }
}

pub type FacePair = [(TexId, u32, u32); 2];

pub struct QuickMenuFaces {
    pub headings: Vec<(QuickRow, (TexId, u32, u32))>,
    pub labels: Vec<[(TexId, u32, u32); 2]>,
    pub pointer: (TexId, u32, u32),
    pub values: Vec<[(TexId, u32, u32); 2]>,
    pub toggles: [[(TexId, u32, u32); 2]; 2],
    pub carets: [(TexId, u32, u32); 2],
    pub legend: [(TexId, u32); 8],
    pub legend_light: [(TexId, u32); 8],
    pub descriptions: [(TexId, u32, u32); 2],
}

pub struct QuickMenu<'a> {
    pub row: QuickRow,
    pub hidden: &'a [QuickRow],
    pub values: [Option<QuickValue>; QuickRow::ALL.len()],
    pub clock: Option<[(TexId, u32, u32); 2]>,
    pub wallpaper: Option<[(TexId, u32, u32); 2]>,
    pub faces: Option<&'a QuickMenuFaces>,
    pub ground: Option<(TexId, u32)>,
    pub palette: Option<[(TexId, u32, u32); 2]>,
    pub gba_bezel: Option<[(TexId, u32, u32); 2]>,
    pub gb_bezel: Option<[(TexId, u32, u32); 2]>,
    pub gbc_bezel: Option<[(TexId, u32, u32); 2]>,
    pub bezel: Option<[(TexId, u32, u32); 2]>,
    pub bound: Option<&'a [Option<FacePair>]>,
    pub description: Option<(TexId, u32, u32)>,
    pub over_game: bool,
    /// The carousel footer carries the legends, so the centred pair is not drawn.
    pub footer_legends: bool,
    /// Where the sliding footer starts. The scrim stops there so it does not darken the strip.
    pub footer_top: f32,
    pub canvas: Canvas,
}

impl QuickMenu<'_> {
    pub fn draw(&self, out: &mut Vec<Draw>) {
        self.draw_animated(crate::MenuFrame::settled(self.row, self.hidden), out);
    }

    pub fn draw_animated(&self, frame: crate::MenuFrame, out: &mut Vec<Draw>) {
        let begin = out.len();
        self.draw_content(frame, out);
        // The scene is a few existing sprites. Transform them in place: no
        // framebuffer copies, live blur, text rasterisation or texture uploads.
        let dy = (1.0 - frame.entry) * 18.0;
        let dx = frame.page_shift * (self.canvas.w - 2.0 * crate::expressive::MENU_INSET);
        for draw in &mut out[begin..] {
            match draw {
                Draw::RoundRect { x, y, colour, .. } => { *x += dx; *y += dy; colour[3] *= frame.opacity; }
                Draw::Tex { x, y, alpha, .. } => { *x += dx; *y += dy; *alpha *= frame.opacity; }
                // Keep full-screen grounds anchored.
                Draw::Rect { colour, .. } => colour[3] *= frame.opacity,
                Draw::Part { x, y, alpha, .. } if *y >= 40.0 => { *x += dx; *y += dy; *alpha *= frame.opacity; }
                Draw::Part { alpha, .. } => *alpha *= frame.opacity,
                _ => {}
            }
        }
        clip_quads(out, begin, [crate::expressive::MENU_INSET, 40.0, self.canvas.w - crate::expressive::MENU_INSET, LEGEND_Y + MENU_HINT_H as f32]);
    }

    fn draw_content(&self, frame: crate::MenuFrame, out: &mut Vec<Draw>) {
        use crate::expressive::{rounded, ACCENT, SELECTED, MENU_INSET, ROW_H, ROW_RADIUS};
        if frame.background { if frame.closing {
            out.push(Draw::Rect { x: 0.0, y: 0.0, w: self.canvas.w,
                h: OUT_H as f32 - MOUTH_H, colour: [0.0, 0.0, 0.0, GROUND_SCRIM] });
        } else { match (self.over_game, self.row.page() == QuickRow::PLAY_SCREEN) {
            (false, _) => draw_menu_ground(self.canvas, self.ground, out),
            (true, panel) => {
                let y = match panel {
                    true => quick_row_top(self.row.page()[0], self.hidden) - 52.0,
                    false => 0.0,
                };
                let bottom = self.footer_top.max(y);
                out.push(Draw::Rect {
                    x: 0.0,
                    y,
                    w: self.canvas.w,
                    h: bottom - y,
                    colour: [0.0, 0.0, 0.0, GROUND_SCRIM],
                });
            }
        }
        }
        }
        let Some(faces) = self.faces else {
            return;
        };
        let (right, pad) = (self.canvas.w - QUICK_EDGE, MENU_PAD as f32);
        let shown = self.row.shown(self.hidden);
        let (start, len) = quick_window(self.row, self.hidden);
        if len == 0 { return; }
        let top = quick_row_top_at(shown[start], self.row, self.hidden);
        let panel_x = MENU_INSET;
        let panel_w = self.canvas.w - panel_x * 2.0;
        let panel_y = top - 48.0;
        if let Some((_, (tex, w, h))) = faces.headings.iter().find(|(key, _)| *key == self.row.page()[0]) {
            push(out, *tex, QUICK_LABEL_X - pad, panel_y, *w, *h);
        }
        // The selection travels as one shape. Keep it within the current
        // window during wraparound and scrolling so it never covers a heading.
        let focus_y = frame.focus_y.clamp(top - 2.0, top + (len - 1) as f32 * QUICK_PITCH + 2.0);
        rounded(out, panel_x, focus_y, panel_w, ROW_H, ROW_RADIUS, SELECTED);
        for (index, row) in shown.iter().copied().enumerate() {
            let y = top + (index as f32 - frame.window_start) * QUICK_PITCH + TYPE_DROP;
            if y + MENU_H as f32 <= top || y >= top + len as f32 * QUICK_PITCH { continue; }
            let row_begin = out.len();
            let lit = row == self.row;
            if let Some(&(tex, w, h)) = faces.labels.get(row.index()).map(|l| &l[lit as usize]) {
                let x = QUICK_LABEL_X - pad;
                push(out, tex, x, y, w, h);
            }
            if row.opens() && !row.binds() {
                let (tex, w, h) = faces.carets[1];
                push(out, tex, right - w as f32, y, w, h);
            }
            if row.toggle() {
                if let Some(value) = self.values[row.index()] {
                    let on = matches!(value, QuickValue::On | QuickValue::Show);
                    let t = if lit { frame.switch.unwrap_or(if on { 1.0 } else { 0.0 }) }
                        else if on { 1.0 } else { 0.0 };
                    let x = right - 60.0;
                    let off = [0.33, 0.36, 0.40, 1.0];
                    let track = std::array::from_fn(|i| off[i] + (ACCENT[i] - off[i]) * t);
                    rounded(out, x, y + 5.0, 60.0, 30.0, 15.0, track);
                    let size = 19.0 + t * 3.0;
                    rounded(out, x + 5.0 + 28.0 * t, y + 20.0 - size / 2.0,
                        size, size, size / 2.0, [0.95, 0.97, 0.96, 1.0]);
                }
                clip_quads(out, row_begin, [MENU_INSET, top, self.canvas.w - MENU_INSET, top + len as f32 * QUICK_PITCH - 8.0]);
                continue;
            }
            let value = match row {
                _ if row.toggle() => self.values[row.index()].and_then(|value| match value {
                    QuickValue::On | QuickValue::Show => Some(faces.toggles[1][lit as usize]),
                    QuickValue::Off | QuickValue::Hide => Some(faces.toggles[0][lit as usize]),
                    _ => None,
                }),
                QuickRow::DateTime => self.clock.map(|c| c[lit as usize]),
                QuickRow::Wallpaper => self.wallpaper.map(|c| c[lit as usize]),
                QuickRow::Palette | QuickRow::GbPalette => self.palette.map(|c| c[lit as usize]),
                QuickRow::GbaBezel => self.gba_bezel.map(|c| c[lit as usize]),
                QuickRow::GbBezel => self.gb_bezel.map(|c| c[lit as usize]),
                QuickRow::GbcBezel => self.gbc_bezel.map(|c| c[lit as usize]),
                QuickRow::Bezel => self.bezel.map(|c| c[lit as usize]),
                _ if row.binds() => self
                    .bound
                    .and_then(|b| b.get(row.index()).copied().flatten())
                    .map(|c| c[lit as usize]),
                _ => self.values[row.index()]
                    .and_then(|v| faces.values.get(v.index()))
                    .map(|v| v[lit as usize]),
            };
            let Some((tex, w, h)) = value else {
                clip_quads(out, row_begin, [MENU_INSET, top, self.canvas.w - MENU_INSET, top + len as f32 * QUICK_PITCH - 8.0]);
                continue;
            };
            if !lit || row.opens() || row.binds() || row.toggle() {
                push(out, tex, right + pad - w as f32, y, w, h);
                clip_quads(out, row_begin, [MENU_INSET, top, self.canvas.w - MENU_INSET, top + len as f32 * QUICK_PITCH - 8.0]);
                continue;
            }
            let [(left_tex, lw, lh), (right_tex, rw, rh)] = faces.carets;
            let rx = right - rw as f32;
            push(out, right_tex, rx, y, rw, rh);
            let vx = rx - CARET_GAP + pad - w as f32;
            push(out, tex, vx, y, w, h);
            push(out, left_tex, vx + pad - CARET_GAP - lw as f32, y, lw, lh);
            clip_quads(out, row_begin, [MENU_INSET, top, self.canvas.w - MENU_INSET, top + len as f32 * QUICK_PITCH - 8.0]);
        }
        if let Some(mut bar) = quick_scroll(self.row, self.hidden).filter(|_| !frame.outgoing) {
            let along = (frame.scroll_position / (shown.len() - 1) as f32).clamp(0.0, 1.0);
            bar.thumb_y = bar.y + (bar.h - bar.thumb_h) * along;
            let x = self.canvas.w - SCROLL_INSET - SCROLL_W;
            let ink = DIM_INK.map(|c| f32::from(c) / 255.0);
            out.push(Draw::Rect {
                x,
                y: bar.y,
                w: SCROLL_W,
                h: bar.h,
                colour: [ink[0], ink[1], ink[2], 0.3],
            });
            let lit = crate::hud::HUD_INK.map(|c| f32::from(c) / 255.0);
            out.push(Draw::Rect {
                x,
                y: bar.thumb_y,
                w: SCROLL_W,
                h: bar.thumb_h,
                colour: [lit[0], lit[1], lit[2], 1.0],
            });
        }
        if let Some((tex, w, h)) = self.description {
            let y = LEGEND_Y - h as f32 - DESCRIPTION_GAP;
            push(out, tex, self.canvas.centred(w as f32), y, w, h);
        }
        if self.over_game || self.footer_legends {
            return;
        }
        let [back, change, open, resume, ..] = faces.legend;
        let back = match self.row.page() == QuickRow::PLAY {
            true => resume,
            false => back,
        };
        let other = if self.row.opens() { open } else { change };
        for (tex, w, x) in centred_hints(self.canvas, &[back, other], LEGEND_GAP) {
            push(out, tex, x, LEGEND_Y, w, MENU_HINT_H);
        }
    }
}

pub fn draw_menu_ground(canvas: Canvas, ground: Option<(TexId, u32)>, out: &mut Vec<Draw>) {
    let (w, h) = (canvas.w, OUT_H as f32);
    match ground {
        Some((tex, tex_w)) => {
            out.push(Draw::Part {
                x: 0.0,
                y: 0.0,
                w,
                h,
                tex,
                alpha: 1.0,
                uv: canvas.cover(tex_w as f32),
            });
            out.push(Draw::Rect {
                x: 0.0,
                y: 0.0,
                w,
                h,
                colour: [0.0, 0.0, 0.0, GROUND_SCRIM],
            });
        }
        None => out.push(Draw::Rect {
            x: 0.0,
            y: 0.0,
            w,
            h,
            colour: opening(),
        }),
    }
}

pub const QUICK_VISIBLE: usize = 5;

const PANEL_VISIBLE: usize = 3;

const SCROLL_INSET: f32 = 40.0;

const SCROLL_W: f32 = 4.0;

#[derive(Copy, Clone, PartialEq, Debug)]
pub struct QuickScroll {
    pub y: f32,
    pub h: f32,
    pub thumb_y: f32,
    pub thumb_h: f32,
}

fn visible(page: &[QuickRow]) -> usize {
    match page == QuickRow::PLAY_SCREEN {
        true => PANEL_VISIBLE,
        false => QUICK_VISIBLE,
    }
}

pub fn quick_window(selected: QuickRow, hidden: &[QuickRow]) -> (usize, usize) {
    let count = selected.shown(hidden).len();
    let fits = visible(selected.page());
    if count <= fits {
        return (0, count);
    }
    let at = selected.position(hidden);
    let start = at.saturating_sub(fits / 2).min(count - fits);
    (start, fits)
}

pub fn quick_scroll(selected: QuickRow, hidden: &[QuickRow]) -> Option<QuickScroll> {
    let shown = selected.shown(hidden);
    let (start, len) = quick_window(selected, hidden);
    if len >= shown.len() {
        return None;
    }
    let y = quick_row_top_at(shown[start], selected, hidden);
    let h = QUICK_PITCH * len as f32;
    let thumb_h = h * len as f32 / shown.len() as f32;
    let along = selected.position(hidden) as f32 / (shown.len() - 1) as f32;
    Some(QuickScroll {
        y,
        h,
        thumb_y: y + (h - thumb_h) * along,
        thumb_h,
    })
}

pub fn quick_row_top_at(row: QuickRow, selected: QuickRow, hidden: &[QuickRow]) -> f32 {
    let (start, len) = quick_window(selected, hidden);
    let rows = QUICK_PITCH * len as f32;
    let top = match row.page() == QuickRow::PLAY_SCREEN {
        true => LEGEND_Y - PANEL_GAP - rows,
        false => 88.0,
    };
    top + QUICK_PITCH * (row.position(hidden) as f32 - start as f32)
}

pub fn quick_row_top(row: QuickRow, hidden: &[QuickRow]) -> f32 {
    quick_row_top_at(row, row, hidden)
}

pub fn quick_window_start(selected: QuickRow, hidden: &[QuickRow]) -> usize {
    quick_window(selected, hidden).0
}

fn push(out: &mut Vec<Draw>, tex: TexId, x: f32, y: f32, w: u32, h: u32) {
    out.push(Draw::Tex {
        x: x.round(),
        y: y.round(),
        w: w as f32,
        h: h as f32,
        tex,
        alpha: 1.0,
    });
}

/// Sentence case for authored interface copy, retaining acronyms and brand names.
pub fn menu_sentence_case(label: &str) -> String {
    label.split(' ').enumerate().map(|(i, word)| {
        if i == 0 || word.chars().filter(|c| c.is_alphabetic()).all(|c| c.is_uppercase()) ||
            ["Bluetooth", "Wi-Fi", "D-Pad", "mGBA", "gpSP"].contains(&word) { word.to_string() }
        else { word.to_lowercase() }
    }).collect::<Vec<_>>().join(" ")
}

pub(crate) fn clip_quads(out: &mut Vec<Draw>, begin: usize, bounds: [f32; 4]) {
    let mut at = begin;
    while at < out.len() {
        let old = out[at];
        let (x,y,w,h) = match old {
            Draw::Tex{x,y,w,h,..} | Draw::Part{x,y,w,h,..} | Draw::RoundRect{x,y,w,h,..} => (x,y,w,h),
            _ => { at += 1; continue; }
        };
        // Full-screen ground textures stay anchored and uncropped.
        if y < 40.0 && matches!(old, Draw::Part { .. }) { at += 1; continue; }
        let (left,top,right,bottom) = (x.max(bounds[0]),y.max(bounds[1]),(x+w).min(bounds[2]),(y+h).min(bounds[3]));
        if right <= left || bottom <= top { out.remove(at); continue; }
        if left != x || top != y || right != x+w || bottom != y+h {
            out[at] = match old {
                Draw::Tex{tex,alpha,..} => Draw::Part{x:left,y:top,w:right-left,h:bottom-top,tex,alpha,uv:[(left-x)/w,(top-y)/h,(right-left)/w,(bottom-top)/h]},
                Draw::Part{tex,alpha,uv,..} => Draw::Part{x:left,y:top,w:right-left,h:bottom-top,tex,alpha,uv:[uv[0]+(left-x)/w*uv[2],uv[1]+(top-y)/h*uv[3],(right-left)/w*uv[2],(bottom-top)/h*uv[3]]},
                Draw::RoundRect{radius,colour,..} => Draw::RoundRect{x:left,y:top,w:right-left,h:bottom-top,radius,colour},
                _ => unreachable!(),
            };
        }
        at += 1;
    }
}
