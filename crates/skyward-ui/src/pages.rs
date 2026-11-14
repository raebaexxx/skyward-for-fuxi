use crate::app::{
    Action, BackupAt, BackupEnd, BackupJob, Check, ClashChoice, Confirm, Damage, Data, Dock, DownloadEnd, EraseStop, Fetch, Field, FileKind,
    FileOp, FileTask, Fold, Folder, Frame, FsAction, FsChoice, FsRow, Have, HeroArt, Hit, Install, JobEnd, Keys, Pick, Place, Rows, Screen,
    SkywardCheck, SkywardEnd, SkywardJob, SkywardStage, Slider, SortKey, Source, State, Switch, SystemsWork, Target, UsbDrive, UsbLink, Wipe,
    ADDABLE,
    BACKUP_PARTS, DATA_ERASED, DATA_PARTITION, DOWNLOADS, FASTBOOT_CHOICES, SCREEN_OFF, TARGETS, TIME_ZONES, USB_DRIVE_UNREADABLE,
    USB_DRIVE_WANTED, WIPES, installs, starting,
};
use crate::canvas::Canvas;
use crate::color::Rgba;
use crate::editor;
use crate::files;
use crate::fmt::{self, Verb};
use crate::font::Fonts;
use crate::geom::Rect;
use crate::lang::{Lang, Text};
use crate::search;
use crate::term::{self, Term};
use crate::theme::Theme;
use crate::tr;
use crate::icons::{self, Icon};
use crate::widgets::{self, Cap, Caption, Detail, Fact, Outline, Row, Spec, SystemCard, SystemMark, Tone, Trail};
use skyward_info::{DeviceInfo, BUILDER, VERSION};
use skyward_install::{Outcome, Reflash, Refusal, Stage};
use skyward_net::download::Progress;
use skyward_net::supplicant::Security;
use skyward_net::wlan::{Bands, Network, Power, Stage as Joining};
use skyward_systems::Who;
use skyward_vab::{Slot, Standing};
use std::path::Path;

// The colours the stock Settings fills its icons' tiles with, the same in
// light and dark mode.
const BLUE: Rgba = Rgba::rgb(0x34, 0x82, 0xff);
const GREEN: Rgba = Rgba::rgb(0x1d, 0xcd, 0x3a);
const PURPLE: Rgba = Rgba::rgb(0x77, 0x67, 0xf9);
const SLATE: Rgba = Rgba::rgb(0x8c, 0x9d, 0xb0);
const ORANGE: Rgba = Rgba::rgb(0xff, 0x69, 0x00);
const RED: Rgba = Rgba::rgb(0xfa, 0x38, 0x2e);

/// What every page draws from: style, typefaces, the device readings, and
/// what the pages hold, from the choices made on them to where the data
/// partition stands.
pub struct Ctx<'a> {
    pub t: &'a Theme,
    pub f: &'a Fonts,
    pub info: &'a DeviceInfo,
    pub state: &'a State,
    /// The shell the terminal page shows, once one has started.
    pub term: Option<&'a Term>,
    /// The part of the page to draw; all of it if none is given. The page
    /// is laid out whole either way.
    pub band: Option<Band>,
}

/// Rows `rows` of a page, and of them the columns `cols`, each from the
/// first to one past the last.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Band {
    pub rows: (u32, u32),
    pub cols: (u32, u32),
}

pub fn render(screen: Screen, cx: &Ctx) -> Frame {
    match screen {
        Screen::Decrypt => decrypt(cx),
        Screen::Home => home(cx),
        Screen::Settings => settings(cx),
        Screen::Reboot => reboot(cx),
        Screen::Wipe => wipe(cx),
        Screen::Mount => mount(cx),
        Screen::Install => install(cx),
        Screen::Choose(place) => choose(cx, place),
        Screen::Installing => installing(cx),
        Screen::About => about(cx),
        Screen::Specs => specs(cx),
        Screen::ScreenOff => screen_off(cx),
        Screen::Search => search(cx),
        Screen::Backup => backup(cx),
        Screen::Restore => restore(cx),
        Screen::RestoreBackup => restore_backup(cx),
        Screen::BackupJob => backup_job(cx),
        Screen::Advanced => advanced(cx),
        Screen::Files => files(cx),
        Screen::FileInfo => file_info(cx),
        Screen::Permissions => permissions(cx),
        Screen::Editor => editor(cx),
        Screen::Terminal => terminal(cx),
        Screen::Relabel => relabel(cx),
        Screen::Filesystems => filesystems(cx),
        Screen::Filesystem => filesystem(cx),
        Screen::PartWork => part_work(cx),
        Screen::Language => language(cx),
        Screen::TimeZone => time_zone(cx),
        Screen::Logs => logs(cx),
        Screen::Update => update(cx),
        Screen::SkywardUpdate => skyward_update(cx),
        Screen::License => license(cx),
        Screen::FontLicense => font_license(cx),
        Screen::Slot => slot(cx),
        Screen::Systems => systems(cx),
        Screen::Fastboot => fastboot(cx),
        Screen::Erase => erase(cx),
        Screen::CantStart => cant_start(cx),
        Screen::Unhandled => unhandled(cx),
        Screen::Rename => rename(cx),
        Screen::Clash => clash(cx),
        Screen::FileJob => file_job(cx),
        Screen::Wlan => wlan(cx),
        Screen::WlanPassword => wlan_password(cx),
        Screen::WlanNetwork => wlan_network(cx),
        Screen::WlanAdd => wlan_add(cx),
        Screen::Wireless => wireless(cx),
        Screen::Fetch => fetch(cx),
        Screen::Sort => sort(cx),
    }
}

/// Label and colour of each swipe-to-confirm control: the filesystem
/// page's as `s` says what it is to do.
pub fn confirm_label(which: Confirm, s: &State) -> (&'static str, bool) {
    const INSTALL: Text = ["Slide to install", "Сдвиньте для установки"];
    let (label, danger) = match which {
        Confirm::Filesystem => match s.fs_action {
            FsAction::Repair => (["Slide to repair", "Сдвиньте, чтобы починить"], true),
            FsAction::Remake(_) => (["Slide to change", "Сдвиньте для изменения"], true),
        },
        Confirm::Reboot => (["Slide to reboot", "Сдвиньте для перезагрузки"], false),
        Confirm::Wipe => (["Slide to wipe", "Сдвиньте для очистки"], true),
        Confirm::Flash => (["Slide to flash", "Сдвиньте для прошивки"], true),
        Confirm::Backup => (["Slide to back up", "Сдвиньте для создания копии"], false),
        Confirm::Restore => (["Slide to restore", "Сдвиньте для восстановления"], true),
        Confirm::Slot => (["Slide to switch", "Сдвиньте для переключения"], true),
        Confirm::FactoryReset => (["Slide to factory reset", "Сдвиньте для сброса к заводским настройкам"], true),
        Confirm::DeleteEntry | Confirm::DeleteBackup => (["Slide to delete", "Сдвиньте для удаления"], true),
        Confirm::Install => (INSTALL, false),
        Confirm::InstallWipe => (["Slide to install and wipe", "Сдвиньте для установки и очистки"], true),
        Confirm::RunScript => (INSTALL, true),
        Confirm::Merge => (["Slide to merge", "Сдвиньте для объединения"], true),
        Confirm::Relabel => (["Slide to fix", "Сдвиньте, чтобы исправить"], true),
        Confirm::SwitchSystem => (["Slide to switch and reboot", "Сдвиньте для переключения и перезагрузки"], false),
        Confirm::RemoveSystem => (["Slide to remove", "Сдвиньте, чтобы убрать"], true),
        Confirm::InstallSystem => (INSTALL, false),
    };
    (s.lang.pick(label), danger)
}

/// The home page's large title, and what its search bar and the search
/// page's field say while nothing is typed.
pub const HOME_TITLE: &str = "Skyward";
pub const SEARCH_HINT: Text = ["Search features and partitions", "Поиск функций и разделов"];

const REBOOT_TARGETS: [(Text, Option<Text>); 6] = [
    (["System", "Система"], None),
    (["Recovery", "Recovery"], None),
    (["Fastboot"; 2], Some(["Userspace fastbootd", "fastbootd в пользовательском пространстве"])),
    (["Bootloader"; 2], None),
    (["EDL"; 2], Some(["Qualcomm emergency download", "Аварийная загрузка Qualcomm"])),
    (["Power off", "Выключить"], None),
];

/// What pages are called both on the rows that open them and in their own
/// titles.
const WLAN: Text = ["Wi-Fi", "Wi-Fi"];
const DECRYPT: Text = ["Decrypt", "Расшифровать"];
const ADVANCED: Text = ["Advanced", "Дополнительно"];
const SETTINGS: Text = ["Settings", "Настройки"];
const REBOOT: Text = ["Reboot", "Перезагрузить"];
const ABOUT: Text = ["About phone", "О телефоне"];
const SPECS: Text = ["All specs", "Все характеристики"];
const LOGS: Text = ["Export logs", "Экспорт журналов"];
const SCREEN_TIMEOUT: Text = ["Screen timeout", "Отключение экрана"];
const UPDATE: Text = ["System update", "Обновление системы"];
const SKYWARD_UPDATE: Text = ["Skyward update", "Обновление Skyward"];
const FILES: Text = ["File manager", "Файловый менеджер"];
const TERMINAL: Text = ["Terminal", "Терминал"];
const LANGUAGE: Text = ["Language", "Язык"];
const TIME_ZONE: Text = ["Time zone", "Часовой пояс"];
const LICENSE: Text = ["Open source licenses", "Лицензии"];
const FONTS: Text = ["UI fonts", "Шрифты интерфейса"];
const DUAL_BOOT: Text = ["Dual boot", "Вторая система"];

/// A canvas for a page `h` tall in the colour `bg`, holding the rows `cx`
/// asks to be drawn.
fn sheet(cx: &Ctx, h: u32, bg: Rgba) -> Canvas {
    let w = cx.t.m.w;
    let Band { rows, cols } = cx.band.unwrap_or(Band { rows: (0, h), cols: (0, w) });
    Canvas::band(w, h, rows.0..rows.1, cols.0..cols.1, bg)
}

fn page(cx: &Ctx) -> Canvas {
    sheet(cx, cx.t.m.h, cx.t.pal.bg)
}

/// A canvas tall enough for a scrolling page; [`fit`] trims it to the content.
fn long_page(cx: &Ctx) -> Canvas {
    sheet(cx, cx.t.m.h * 3, cx.t.pal.bg)
}

/// How tall a long page whose content ends at `bottom` is: to the bottom
/// margin after it, and never shorter than one screen.
fn fitted(t: &Theme, bottom: f32) -> u32 {
    ((bottom + t.m.margin * 2.0).ceil() as u32).max(t.m.h)
}

/// Trim a long page to its content plus the bottom margin, and never shorter
/// than one screen.
fn fit(c: Canvas, t: &Theme, bottom: f32) -> Canvas {
    c.crop_height(fitted(t, bottom))
}

fn hits(rects: &[Rect], actions: &[Action]) -> Vec<Hit> {
    rects.iter().zip(actions).map(|(&rect, &action)| Hit { rect, action }).collect()
}

/// Recovery home, laid out like the Settings root: title, search, a device
/// card, then grouped entries. The status bar carries the readings: the
/// slot, the data partition's lock and the battery's temperature.
pub fn home(cx: &Ctx) -> Frame {
    let (t, f, i, l) = (cx.t, cx.f, cx.info, cx.state.lang);
    let mut c = long_page(cx);
    let mut out = Vec::new();
    // The search bar is left room on the page and drawn over it, folding
    // away as the page scrolls up.
    let bar = Rect::new(t.m.margin, t.m.content_top, t.m.w as f32 - t.m.margin * 2.0, t.m.search_h);
    let mut y = bar.y + bar.h + t.m.after_search;
    let cards_top = y;
    let fold = Fold { bar, hint: l.pick(SEARCH_HINT), span: cards_top - bar.y };
    out.push(Hit { rect: bar, action: Action::Open(Screen::Search) });

    let sub = match &i.os_version {
        Some(v) => format!("{} · {v}", i.codename),
        None => i.codename.clone(),
    };
    // Under the phone's name the card holds only what asks for something to
    // be done: a battery too low to install on, an update waiting to merge,
    // and a restart into the slot the system does not run from now.
    let low = match i.battery.as_ref().and_then(skyward_install::battery_refusal) {
        Some(Refusal::Battery { level, charging }) => Some(fmt::battery_low(l, level, charging)),
        _ => None,
    };
    let next = match (i.slot, &cx.state.boot_slots) {
        (Some(now), Some(Ok(slots))) if slots.active != now => Some(fmt::next_boot(l, slots.active, now)),
        _ => None,
    };
    let mut rows = Vec::new();
    let mut opens = Vec::new();
    if let Some((title, note)) = &low {
        rows.push(Row::new(title).icon(RED, Icon::Battery).subtitle(note).trail(Trail::None));
        opens.push(None);
    }
    if let Some(note) = fmt::snapshot_note(l, &i.snapshot) {
        rows.push(Row::new(fmt::snapshot(l, &i.snapshot)).icon(ORANGE, Icon::Layers).subtitle(note).trail(Trail::Arrow));
        opens.push(Some(Action::Open(Screen::Update)));
    }
    if let Some((title, note)) = &next {
        rows.push(Row::new(title).icon(ORANGE, Icon::Power).subtitle(note).trail(Trail::Arrow));
        opens.push(Some(Action::Open(Screen::Slot)));
    }
    if cx.state.system_notice.is_some() {
        let why = l.pick(["Tap to see why", "Нажмите, чтобы узнать причину"]);
        rows.push(Row::new(l.pick(SECOND_FAILED)).icon(RED, Icon::Systems).subtitle(why).trail(Trail::Arrow));
        opens.push(Some(Action::Open(Screen::Systems)));
    }
    let card = widgets::device_card(&mut c, t, f, y, &i.model, &sub, &rows);
    out.push(Hit { rect: card.rows[0], action: Action::Open(Screen::About) });
    out.extend(card.rows[1..].iter().zip(&opens).filter_map(|(&rect, open)| open.map(|action| Hit { rect, action })));
    y = card.bottom + t.m.card_gap;
    // On a phone whose WLAN the recovery brings up, the WLAN comes first
    // under the device, as on the stock settings home; with it, where the
    // phone can hold a second system, the two systems.
    let mut rows = Vec::new();
    let mut opens = Vec::new();
    if let Some(w) = &cx.state.wlan {
        rows.push(Row::new(l.pick(WLAN)).icon(BLUE, Icon::Wlan).trail(Trail::Value(fmt::wlan_value(l, w))));
        opens.push(Action::Open(Screen::Wlan));
    }
    let starts = systems_value(cx);
    if cx.state.offers(Screen::Systems) {
        let row = Row::new(l.pick(DUAL_BOOT)).icon(PURPLE, Icon::Systems);
        rows.push(match &starts {
            Some(value) => row.trail(Trail::Value(value)),
            None => row,
        });
        opens.push(Action::Open(Screen::Systems));
    }
    if !rows.is_empty() {
        let first = widgets::list(&mut c, t, f, y, &rows);
        out.extend(hits(&first.rows, &opens));
        y = first.bottom + t.m.card_gap;
    }
    let tools = widgets::list(&mut c, t, f, y, &[
        Row::new(l.pick(FILES)).icon(BLUE, Icon::Folder),
        Row::new(l.pick(["Install", "Установить"])).icon(GREEN, Icon::Install),
        Row::new(l.pick(["Backup", "Резервная копия"])).icon(BLUE, Icon::Backup),
        Row::new(l.pick(["Restore", "Восстановить"])).icon(PURPLE, Icon::Restore),
        Row::new(l.pick(["Mount", "Смонтировать"])).icon(SLATE, Icon::Mount),
        Row::new(l.pick(DECRYPT)).icon(BLUE, Icon::Lock).trail(Trail::Value(fmt::data(l, &cx.state.data))),
    ]);
    out.extend(hits(&tools.rows, &[
        Action::Open(Screen::Files),
        Action::Open(Screen::Install),
        Action::Open(Screen::Backup),
        Action::Open(Screen::Restore),
        Action::Open(Screen::Mount),
        Action::Open(Screen::Decrypt),
    ]));
    y = tools.bottom + t.m.card_gap;
    let target = l.pick(REBOOT_TARGETS[cx.state.reboot].0);
    let system = widgets::list(&mut c, t, f, y, &[
        Row::new(l.pick(["Wipe data", "Очистить данные"])).icon(RED, Icon::Wipe),
        Row::new(l.pick(ADVANCED)).icon(SLATE, Icon::More),
        Row::new(l.pick(SETTINGS)).icon(SLATE, Icon::Gear),
        Row::new(l.pick(REBOOT)).icon(ORANGE, Icon::Power).trail(Trail::Value(target)),
    ]);
    out.extend(hits(&system.rows, &[
        Action::Open(Screen::Wipe),
        Action::Open(Screen::Advanced),
        Action::Open(Screen::Settings),
        Action::Open(Screen::Reboot),
    ]));
    // However little it holds, the page scrolls far enough to fold its bar
    // away.
    let height = fitted(t, system.bottom).max(t.m.h + fold.span.ceil() as u32);
    Frame { back: false, fold: Some(fold), ..Frame::new(t, c.crop_height(height), out, HOME_TITLE, cards_top) }
}

/// What the home page says beside its entry to the systems: the one that
/// starts, where a second system is installed, once they are read.
fn systems_value(cx: &Ctx) -> Option<String> {
    let l = cx.state.lang;
    match &cx.state.systems {
        Some(Ok(systems)) => Some(match (&systems.other, starting(systems)) {
            (None, _) => l.pick(["No second system", "Нет второй системы"]).into(),
            (Some(_), Who::Installed) => systems.installed.name.clone(),
            (Some(other), Who::Other) => other.name.clone(),
        }),
        _ => None,
    }
}

/// What the advanced page says beside its system update entry: a download
/// under way, a newer system found, or where an update the system installed
/// stands.
fn update_value(cx: &Ctx) -> &'static str {
    let (s, l) = (cx.state, cx.state.lang);
    if s.downloading() {
        return l.pick(DOWNLOADING);
    }
    if s.newest().is_some_and(|r| cx.info.os_version.as_deref() != Some(r.version.as_str())) {
        return l.pick(["New version", "Новая версия"]);
    }
    fmt::update_value(l, &s.update)
}

/// Preferences, opening with the stock “Display and brightness” theme previews.
pub fn settings(cx: &Ctx) -> Frame {
    let (t, f, s, l) = (cx.t, cx.f, cx.state, cx.state.lang);
    let mut c = long_page(cx);
    let mut out = Vec::new();
    let top = t.m.content_top;
    let themes = widgets::theme_picker(&mut c, t, f, l, top + t.m.dp(8.9));
    out.extend(hits(&themes.rows, &[Action::Theme { dark: false }, Action::Theme { dark: true }]));
    let mut y = widgets::section(&mut c, t, f, themes.bottom, l.pick(["Brightness", "Яркость"]));
    let off = fmt::screen_off(l, SCREEN_OFF[s.screen_off]);
    let below = Row::new(l.pick(SCREEN_TIMEOUT)).trail(Trail::Value(&off));
    let slider = widgets::slider_card(&mut c, t, f, y, Caption::Sun, l.pick(["Screen brightness", "Яркость экрана"]), s.brightness, Some(below));
    out.push(Hit { rect: slider.track, action: Action::Slider(Slider::Brightness) });
    if let Some(r) = slider.below {
        out.push(Hit { rect: r, action: Action::Open(Screen::ScreenOff) });
    }
    y = widgets::section(&mut c, t, f, slider.bottom, l.pick(["Vibration", "Вибрация"]));
    let below = Row::new(l.pick(["Touch vibration", "Вибрация при касании"])).trail(Trail::Switch(s.vibrate));
    let vibration = widgets::slider_card(&mut c, t, f, y, Caption::Vibrate, l.pick(["Vibration intensity", "Интенсивность вибрации"]), s.vibration, Some(below));
    out.push(Hit { rect: vibration.track, action: Action::Slider(Slider::Vibration) });
    if let Some(r) = vibration.below {
        out.push(Hit { rect: r, action: Action::Toggle(Switch::Vibrate) });
    }
    y = widgets::section(&mut c, t, f, vibration.bottom, l.pick(["General", "Общие"]));
    let general = widgets::list(&mut c, t, f, y, &[
        Row::new(l.pick(LANGUAGE)).icon(BLUE, Icon::Globe).trail(Trail::Value(l.name())),
        Row::new(l.pick(TIME_ZONE)).icon(ORANGE, Icon::Clock).trail(Trail::Value(zone_short(l, s.time_zone))),
    ]);
    out.extend(hits(&general.rows, &[Action::Open(Screen::Language), Action::Open(Screen::TimeZone)]));
    y = widgets::section(&mut c, t, f, general.bottom, "USB");
    let usb = widgets::list(&mut c, t, f, y, &[Row::new(l.pick(["File transfer", "Передача файлов"])).icon(BLUE, Icon::Usb).trail(Trail::Switch(s.file_transfer))]);
    out.extend(hits(&usb.rows, &[Action::Toggle(Switch::FileTransfer)]));
    let note = l.pick([
        "When on, a computer can manage the files in internal storage, once it is decrypted.",
        "Когда включено, компьютер может управлять файлами во внутренней памяти, как только она расшифрована.",
    ]);
    y = widgets::footnote(&mut c, t, f, usb.bottom, note);
    y = widgets::section(&mut c, t, f, y, l.pick(["Security", "Безопасность"]));
    let security = widgets::list(&mut c, t, f, y, &[Row::new(l.pick(REMEMBER)).icon(ORANGE, Icon::Lock).trail(Trail::Switch(s.remember))]);
    out.extend(hits(&security.rows, &[Action::Toggle(Switch::Remember)]));
    let note = l.pick([
        "When on, the screen lock is kept on /metadata so the next start decrypts without asking. Turn it off where someone else may get the phone.",
        "Когда включено, пароль экрана сохраняется в /metadata, и при следующем запуске расшифровка пройдёт без вопроса. Выключите, если телефоном может воспользоваться кто-то другой.",
    ]);
    y = widgets::footnote(&mut c, t, f, security.bottom, note);
    y = widgets::section(&mut c, t, f, y, l.pick(["Debugging", "Отладка"]));
    let debug = widgets::list(&mut c, t, f, y, &[Row::new(l.pick(TERMINAL)).icon(SLATE, Icon::Terminal)]);
    out.extend(hits(&debug.rows, &[Action::Open(Screen::Terminal)]));
    y = widgets::section(&mut c, t, f, debug.bottom, l.pick(["About", "О программе"]));
    let mut rows = vec![Row::new(l.pick(ABOUT)).trail(Trail::Value(VERSION))];
    let mut opens = vec![Action::Open(Screen::About)];
    if s.offers(Screen::SkywardUpdate) {
        let row = Row::new(l.pick(SKYWARD_UPDATE));
        rows.push(match skyward_value(cx) {
            Some(value) => row.trail(Trail::Value(value)),
            None => row,
        });
        opens.push(Action::Open(Screen::SkywardUpdate));
    }
    let about = widgets::list(&mut c, t, f, y, &rows);
    out.extend(hits(&about.rows, &opens));
    Frame::new(t, fit(c, t, about.bottom), out, l.pick(SETTINGS), top)
}

/// What the settings page says beside its Skyward update entry: an update
/// under way, or a newer release found.
fn skyward_value(cx: &Ctx) -> Option<&'static str> {
    let (s, l) = (cx.state, cx.state.lang);
    if s.updating_skyward() {
        return Some(l.pick(["Updating", "Обновление"]));
    }
    match (s.skyward_newest(), &s.skyward_job) {
        (_, Some(SkywardJob { outcome: Some(SkywardEnd::Updated(_)), .. })) => Some(l.pick(["Updated", "Обновлено"])),
        (Some(n), _) if n.newer && n.build.is_some() => Some(l.pick(["New version", "Новая версия"])),
        _ => None,
    }
}

/// How long the screen stays on with nothing touched, opened from the
/// settings.
pub fn screen_off(cx: &Ctx) -> Frame {
    let (t, f, s, l) = (cx.t, cx.f, cx.state, cx.state.lang);
    let mut c = page(cx);
    let top = t.m.content_top;
    let labels = SCREEN_OFF.map(|ms| fmt::screen_off(l, ms));
    let rows: Vec<Row> = labels.iter().enumerate().map(|(k, label)| Row::new(label).trail(Trail::Check(s.screen_off == k))).collect();
    let times = widgets::list(&mut c, t, f, top, &rows);
    let picks: Vec<Action> = (0..SCREEN_OFF.len()).map(Action::ScreenOff).collect();
    let note = l.pick([
        "After the time chosen with nothing touched, the screen dims and turns off. Press the power key to light it again.",
        "По истечении выбранного времени без касаний экран тускнеет и выключается. Нажмите кнопку питания, чтобы включить его снова.",
    ]);
    let y = widgets::footnote(&mut c, t, f, times.bottom, note);
    if SCREEN_OFF[s.screen_off].is_none() {
        let note = l.pick([
            "The screen stays on. Showing the same image for long may leave it burnt in.",
            "Экран остаётся включённым. Долгий показ одного и того же изображения может оставить его выгоревшим.",
        ]);
        widgets::footnote(&mut c, t, f, y, note);
    }
    Frame::new(t, c, hits(&times.rows, &picks), l.pick(SCREEN_TIMEOUT), top)
}

/// Mount manager: the slot's partitions, each with a switch to mount it
/// read-only or unmount it; one that programs are using says which, and
/// stays.
pub fn mount(cx: &Ctx) -> Frame {
    let (t, f, s, l) = (cx.t, cx.f, cx.state, cx.state.lang);
    let mut c = long_page(cx);
    let mut out = Vec::new();
    let top = t.m.content_top;

    let y = widgets::section_first(&mut c, t, f, top, l.pick(["Partitions", "Разделы"]));
    let bottom = match &s.mounts {
        Err(why) => {
            let y = widgets::list(&mut c, t, f, y, &[Row::new(l.pick(["Couldn't read the partitions", "Не удалось прочитать разделы"])).trail(Trail::None)]).bottom;
            widgets::footnote(&mut c, t, f, y, why)
        }
        Ok(mounts) if mounts.is_empty() => {
            widgets::list(&mut c, t, f, y, &[Row::new(l.pick(["No partitions to mount", "Нет разделов для монтирования"])).trail(Trail::None)]).bottom
        }
        Ok(mounts) => {
            let subs: Vec<String> = mounts
                .iter()
                .map(|m| {
                    let (point, holders) = (&m.point, m.holders.join(l.pick([", ", ", "])));
                    match m.holders.is_empty() {
                        true => point.clone(),
                        false => tr!(l, "{point} · in use by {holders}", "{point} · используется {holders}"),
                    }
                })
                .collect();
            let rows: Vec<Row> = mounts
                .iter()
                .zip(&subs)
                .map(|(m, sub)| {
                    let trail = if m.holders.is_empty() { Trail::Switch(m.mounted) } else { Trail::Text(l.pick(["In use", "Используется"])) };
                    Row::new(&m.base).subtitle(sub).trail(trail)
                })
                .collect();
            let list = widgets::list(&mut c, t, f, y, &rows);
            out.extend(list.rows.iter().zip(mounts).enumerate().map(|(i, (&rect, m))| {
                let action = match m.holders.is_empty() {
                    true => Action::MountToggle(i),
                    false => Action::Unavailable(l.pick(["A partition in use can't be unmounted", "Используемый раздел нельзя отмонтировать"])),
                };
                Hit { rect, action }
            }));
            let y = match &s.mount_failure {
                Some(failure) => widgets::footnote(&mut c, t, f, list.bottom, &fmt::mount_failure(l, failure)),
                None => list.bottom,
            };
            let about = match cx.info.slot.map(|s| s.letter()) {
                Some(slot) => tr!(
                    l,
                    "Only the current slot {slot}'s partitions in super show here, all mounted read-only.",
                    "Здесь показаны только разделы super текущего слота {slot}, все смонтированы только для чтения.",
                ),
                None => l
                    .pick([
                        "Only partitions in super show here, all mounted read-only.",
                        "Здесь показаны только разделы в super, все смонтированы только для чтения.",
                    ])
                    .into(),
            };
            widgets::footnote(&mut c, t, f, y, &about)
        }
    };

    Frame::new(t, fit(c, t, bottom), out, l.pick(["Mount", "Смонтировать"]), top)
}

/// Who made the recovery, as the About page names them under its version.
const AUTHOR: &str = "© 2026 RaeBaeXXX";

/// About this device, modelled on the stock “My device” page: the wordmark, the
/// version and its author over a moving colour field, and the summary,
/// hardware and link cards scrolling over it. Only the cards are drawn
/// here; the canvas is clear wherever the field shows.
pub fn about(cx: &Ctx) -> Frame {
    let (t, f) = (cx.t, cx.f);
    let m = &t.m;
    // The page is drawn whole, under the field: laid out first to learn
    // how tall it comes to, then drawn that tall.
    let (_, _, bottom) = about_cards(cx, &mut Canvas::band(m.w, m.h * 3, 0..0, 0..0, Rgba { r: 0, g: 0, b: 0, a: 0 }));
    let mut c = Canvas::new(m.w, fitted(t, bottom));
    let (out, summary_top, bottom) = about_cards(cx, &mut c);
    let canvas = fit(c, t, bottom);
    let art = HeroArt {
        wordmark: widgets::wordmark(t, f, "Skyward", m.dp(233.7), m.dp(37.8)),
        version: widgets::version_line(t, f, VERSION),
        author: widgets::author_line(t, f, AUTHOR),
        runs: canvas.runs(),
    };
    Frame { header: false, hero: Some(art), ..Frame::new(t, canvas, out, cx.state.lang.pick(ABOUT), summary_top) }
}

const MODEL: Text = ["Model", "Модель"];
const PROCESSOR: Text = ["Processor", "Процессор"];
const RAM: Text = ["RAM", "ОЗУ"];
const BATTERY_CAPACITY: Text = ["Battery capacity", "Ёмкость аккумулятора"];
const SCREEN_SIZE: Text = ["Screen size", "Размер экрана"];
const RESOLUTION: Text = ["Resolution", "Разрешение"];
const OS_VERSION: Text = ["OS version", "Версия ОС"];
const ANDROID_VERSION: Text = ["Android version", "Версия Android"];
const CURRENT_SLOT: Text = ["Current slot", "Текущий слот"];
const DATA_PART: Text = ["data partition", "Раздел data"];

/// The phone's processor by its marketing name in `l`, from the phone's
/// settings, or else by its cores.
fn soc(l: Lang, i: &DeviceInfo) -> Option<&str> {
    i.soc.as_ref().map(|n| l.pick([n.en.as_str(), n.ru.as_str()]))
}

/// The About page's cards on `c`: what touching them does, where the
/// first starts, and where the last ends.
fn about_cards(cx: &Ctx, c: &mut Canvas) -> (Vec<Hit>, f32, f32) {
    let (t, f, i, l) = (cx.t, cx.f, cx.info, cx.state.lang);
    let m = &t.m;
    let tone = Tone::hero(t);
    let mut out = Vec::new();

    let storage = fmt::storage_used(&i.storage).unwrap_or_else(|| fmt::capacity(i.storage.total));
    let mut facts = vec![
        Fact { title: l.pick(MODEL), value: i.model.clone(), link: false },
        Fact { title: l.pick(["Storage", "Память"]), value: storage, link: true },
    ];
    if let Some(v) = &i.os_version {
        facts.push(Fact { title: l.pick(OS_VERSION), value: v.clone(), link: false });
    }
    if let Some(r) = &i.android {
        facts.push(Fact { title: l.pick(ANDROID_VERSION), value: r.clone(), link: false });
    }
    let summary = widgets::facts(c, t, f, m.dp(490.7), &facts, &tone);
    out.push(Hit { rect: summary.rows[1], action: Action::Open(Screen::Specs) });

    let processor = soc(l, i).map_or_else(|| fmt::cpu(l, &i.cpu), str::to_string);
    let mut specs = vec![(l.pick(PROCESSOR), processor), (l.pick(RAM), fmt::ram(i.ram_kib))];
    if let Some(design) = i.battery.as_ref().and_then(|b| b.design) {
        specs.push((l.pick(BATTERY_CAPACITY), fmt::mah(design)));
    }
    if let Some(d) = i.screen {
        specs.push((l.pick(SCREEN_SIZE), fmt::inches(d)));
    }
    if let Some(p) = i.panel {
        specs.push((l.pick(RESOLUTION), fmt::panel(p)));
    }
    let y = widgets::spec_card(c, t, f, summary.bottom + m.card_gap, &fmt::model_lines(&i.model), &specs, &tone);
    let links = widgets::list_toned(c, t, f, y + m.card_gap, &[Row::new(l.pick(SPECS)), Row::new(l.pick(LOGS))], &tone);
    out.extend(hits(&links.rows, &[Action::Open(Screen::Specs), Action::Open(Screen::Logs)]));
    (out, summary.rows[0].y, links.bottom)
}

/// Every reading the recovery has, modelled on the stock “All specs” page.
pub fn specs(cx: &Ctx) -> Frame {
    let (t, f, i, l) = (cx.t, cx.f, cx.info, cx.state.lang);
    let gap = t.m.card_gap;
    let mut c = long_page(cx);
    let mut out = Vec::new();
    // This page leaves more room under the title than the settings pages.
    let top = t.m.dp(188.0);

    let mut grid = vec![Spec { icon: Outline::Phone, label: l.pick(MODEL), value: fmt::model_lines(&i.model) }];
    if let Some(design) = i.battery.as_ref().and_then(|b| b.design) {
        grid.push(Spec { icon: Outline::Battery, label: l.pick(BATTERY_CAPACITY), value: fmt::mah(design) });
    }
    let cpu = match soc(l, i) {
        Some(soc) => format!("{soc}\n{}", fmt::cpu(l, &i.cpu)),
        None => fmt::cpu(l, &i.cpu),
    };
    grid.push(Spec { icon: Outline::Chip, label: l.pick(PROCESSOR), value: cpu });
    grid.push(Spec { icon: Outline::Memory, label: l.pick(RAM), value: fmt::ram(i.ram_kib) });
    if let Some(p) = i.panel {
        grid.push(Spec { icon: Outline::Resolution, label: l.pick(RESOLUTION), value: fmt::panel(p) });
    }
    if let Some(d) = i.screen {
        grid.push(Spec { icon: Outline::Screen, label: l.pick(SCREEN_SIZE), value: fmt::inches(d) });
    }
    grid.push(Spec { icon: Outline::Storage, label: l.pick(["Storage capacity", "Объём памяти"]), value: fmt::capacity(i.storage.total) });
    grid.push(Spec { icon: Outline::Tag, label: l.pick(["Codename", "Кодовое имя"]), value: i.codename.clone() });
    let mut y = widgets::spec_grid(&mut c, t, f, top, &grid);

    if let Some(v) = &i.os_version {
        y = widgets::info_card(&mut c, t, f, y + gap, &[(l.pick(OS_VERSION), v.clone())]);
    }
    if let Some(r) = &i.android {
        y = widgets::info_card(&mut c, t, f, y + gap, &[(l.pick(ANDROID_VERSION), fmt::android(l, r, i.security_patch.as_ref()))]);
    }
    let mut recovery = vec![
        (l.pick(["Recovery version", "Версия recovery"]), format!("Skyward {VERSION}")),
        (l.pick(["Kernel version", "Версия ядра"]), i.kernel.clone()),
    ];
    if let Some(builder) = BUILDER.filter(|name| !name.is_empty()) {
        recovery.push((l.pick(["Built by", "Собрано"]), builder.to_string()));
    }
    y = widgets::info_card(&mut c, t, f, y + gap, &recovery);

    let mut boot = Vec::new();
    if let Some(s) = i.slot {
        boot.push((l.pick(CURRENT_SLOT), s.letter().to_string()));
        boot.push((l.pick(UPDATE), fmt::snapshot(l, &i.snapshot).to_string()));
    }
    if let Some(unlocked) = i.boot.unlocked {
        let state = if unlocked { ["Unlocked", "Разблокировано"] } else { ["Locked", "Заблокировано"] };
        boot.push(("Bootloader", l.pick(state).to_string()));
    }
    if let Some(v) = &i.boot.verified {
        boot.push((l.pick(["Verified boot", "Проверенная загрузка"]), fmt::verified(l, v)));
    }
    boot.push((l.pick(DATA_PART), fmt::data(l, &cx.state.data).to_string()));
    y = widgets::info_card(&mut c, t, f, y + gap, &boot);

    if let Some(b) = &i.battery {
        let mut rows = vec![(l.pick(["Level", "Уровень"]), fmt::level(l, b))];
        if let Some(v) = b.temp {
            rows.push((l.pick(["Temperature", "Температура"]), fmt::celsius(v)));
        }
        if let Some(v) = fmt::wear(l, b) {
            rows.push((l.pick(["Health", "Состояние"]), v));
        }
        if let Some(v) = b.cycles {
            rows.push((l.pick(["Charge cycles", "Циклы заряда"]), tr!(l, "{v}", "{v}")));
        }
        if let Some(v) = b.voltage {
            rows.push((l.pick(["Voltage", "Напряжение"]), fmt::volts(v)));
        }
        y = widgets::info_card(&mut c, t, f, y + gap, &rows);
    }

    let total = fmt::capacity(i.storage.total);
    let storage = match i.storage.free.map(fmt::gb) {
        Some(free) => tr!(l, "{free} free of {total}", "{free} свободно из {total}"),
        None => tr!(l, "{total} in all · data partition not mounted", "Всего {total} · раздел data не смонтирован"),
    };
    y = widgets::info_card(&mut c, t, f, y + gap, &[(l.pick(["Internal storage", "Внутренняя память"]), storage)]);
    let links = widgets::list(&mut c, t, f, y + gap, &[
        Row::new(l.pick(LICENSE)).trail(Trail::Value("GPL-3.0")),
        Row::new(l.pick(FONTS)).trail(Trail::Value("MiSans")),
    ]);
    out.extend(hits(&links.rows, &[Action::Open(Screen::License), Action::Open(Screen::FontLicense)]));
    Frame::new(t, fit(c, t, links.bottom), out, l.pick(SPECS), top)
}

/// The data partition and its users' own files. The page opens the
/// partition as it shows, which is how it learns each user's screen lock;
/// a user whose files wait on one is asked with the control that lock is
/// answered with, from the start.
pub fn decrypt(cx: &Ctx) -> Frame {
    let (c, l) = (page(cx), cx.state.lang);
    match &cx.state.data {
        Data::Locked { user, keys, checking } => code(cx, c, *user, *keys, *checking),
        Data::Closed | Data::Opening => {
            let opening = matches!(cx.state.data, Data::Opening);
            let note = match opening {
                true => data_stage(l, cx.state.opening),
                false => l.pick(["Reading the screen lock", "Чтение блокировки экрана"]),
            };
            data_status(
                cx,
                c,
                Icon::Lock,
                l.pick(["Decrypt the data partition", "Расшифруйте раздел данных"]),
                Some(note),
                None,
                opening.then_some(cx.state.opening),
            )
        }
        Data::Open => data_status(
            cx,
            c,
            Icon::Folder,
            l.pick(["The data partition is decrypted", "Раздел data расшифрован"]),
            Some(l.pick(["You can see the phone's files in File manager", "Файлы телефона можно посмотреть в файловом менеджере"])),
            None,
            None,
        ),
        Data::Failed(why) => data_status(cx, c, Icon::Lock, l.pick(["Couldn't open the data partition", "Не удалось открыть раздел данных"]), None, Some(why), None),
        Data::Erased => data_status(
            cx,
            c,
            Icon::Wipe,
            l.pick(DATA_ERASED),
            Some(l.pick(["It's made anew when the phone next starts", "Он создаётся заново при следующем запуске телефона"])),
            None,
            None,
        ),
    }
}

/// What the opening of the data partition is at, as its page says it: the
/// steps the work reports, from the partition being opened to the users'
/// keys being installed.
fn data_stage(l: Lang, value: f32) -> &'static str {
    match (value * 5.0).round() as u8 {
        0 | 1 => l.pick(["Opening the partition", "Открытие раздела"]),
        2 => l.pick(["Installing the shared keys", "Установка общих ключей"]),
        3 => l.pick(["Reading the users' keys", "Чтение ключей пользователей"]),
        4 => l.pick(["Registering the system services", "Регистрация системных служб"]),
        _ => l.pick(["Finishing", "Завершение"]),
    }
}

/// Where the partition stands, under its sign: a line on it, or why it
/// did not open and a way to try again; then the way off the page.
fn data_status(cx: &Ctx, mut c: Canvas, icon: Icon, title: &str, note: Option<&str>, why: Option<&str>, progress: Option<f32>) -> Frame {
    let (t, f, l) = (cx.t, cx.f, cx.state.lang);
    let m = &t.m;
    let x = m.w as f32 / 2.0;
    let sign_cy = m.dp(150.0);
    c.fill_circle(x, sign_cy, m.dp(30.0), t.pal.card);
    icons::glyph(&mut c, icon, x, sign_cy, m.dp(40.0), t.pal.accent);
    // While the partition is being opened, the sign wears the share of the
    // work done as a ring, as the page this one is taken from has it.
    if let Some(value) = progress {
        let r = m.dp(38.0);
        let thin = m.dp(2.5);
        c.stroke_circle(x, sign_cy, r, thin, t.pal.text.with_alpha(28));
        let start = -std::f32::consts::FRAC_PI_2;
        c.stroke_arc(x, sign_cy, r, start, std::f32::consts::TAU * value.clamp(0.0, 1.0), thin, t.pal.accent);
    }
    c.text_center(&f.heading, title, x, sign_cy + m.dp(64.0), m.dp(24.0), t.pal.text);
    if let Some(note) = note {
        c.text_center(&f.body, note, x, sign_cy + m.dp(98.0), m.sub_px, t.pal.subtitle);
    }
    if let Some(value) = progress {
        widgets::progress_bar(&mut c, t, x - m.dp(100.0), sign_cy + m.dp(122.0), m.dp(200.0), value);
    }
    let mut out = Vec::new();
    if let Some(why) = why {
        widgets::paragraph(&mut c, t, f, sign_cy + m.dp(88.0), why);
        let again = widgets::button(&mut c, t, f, m.h as f32 * 0.62, l.pick(["Try again", "Повторить"]));
        out.push(Hit { rect: again, action: Action::OpenData });
        // A partition that holds nothing can be built here, as the system
        // would build it on its first start, rather than waiting for one.
        let build_cy = m.h as f32 * 0.62 + m.dp(58.0);
        c.text_center(&f.heading, l.pick(["Build data afresh", "Создать раздел заново"]), x, build_cy, m.row_title_px, t.pal.accent);
        out.push(Hit { rect: Rect::new(x - m.dp(90.0), build_cy - m.dp(24.0), m.dp(180.0), m.dp(48.0)), action: Action::RebuildData });
    }
    let leave_cy = m.h as f32 - m.dp(64.0);
    let leave = l.pick(if matches!(cx.state.data, Data::Open | Data::Erased) { ["Done", "Готово"] } else { NOT_NOW });
    c.text_center(&f.heading, leave, x, leave_cy, m.row_title_px, t.pal.accent);
    out.push(Hit { rect: Rect::new(x - m.dp(80.0), leave_cy - m.dp(24.0), m.dp(160.0), m.dp(48.0)), action: Action::Skip });
    Frame { header: false, ..Frame::new(t, c, out, l.pick(DECRYPT), 0.0) }
}

/// What leaves the decrypt page with the data partition left as it is.
const NOT_NOW: Text = ["Not now", "Не сейчас"];

/// What a row whose work waits on the data partition says under it until
/// that is decrypted.
const DECRYPT_NEEDED: Text = ["Decrypt first", "Сначала расшифруйте"];

/// A user's own files wait on their screen lock: the control it is
/// answered with, which takes nothing while what was entered is checked.
fn code(cx: &Ctx, mut c: Canvas, user: u32, keys: Keys, checking: bool) -> Frame {
    let (t, f, s, l) = (cx.t, cx.f, cx.state, cx.state.lang);
    let m = &t.m;
    let x = m.w as f32 / 2.0;
    let mut out = Vec::new();
    let title = match (keys == Keys::Pattern, user) {
        (true, 0) => l.pick(["Draw the screen lock pattern", "Нарисуйте графический ключ"]).to_string(),
        (true, user) => tr!(l, "Draw user {user}'s screen lock pattern", "Нарисуйте графический ключ пользователя {user}"),
        (false, 0) => l.pick(["Enter the screen lock", "Введите код блокировки экрана"]).to_string(),
        (false, user) => tr!(l, "Enter user {user}'s screen lock", "Введите код блокировки экрана пользователя {user}"),
    };
    let note = l.pick(if checking { ["Checking", "Проверка"] } else { ["Decrypt to see the phone's files", "Расшифруйте, чтобы увидеть файлы телефона"] });
    c.text_center(&f.heading, &title, x, m.dp(130.0), m.dp(24.0), t.pal.text);
    c.text_center(&f.body, note, x, m.dp(166.0), m.sub_px, t.pal.subtitle);
    let mut pattern = None;
    let bottom = match keys {
        Keys::Letters => {
            let top = m.h as f32 - widgets::keyboard_height(t);
            let caps = widgets::keyboard(&mut c, t, f, l, top, s.layer, s.key_held(), l.pick(DECRYPT));
            if !checking {
                out.extend(caps.into_iter().map(|(rect, cap)| Hit { rect, action: Action::Type(cap) }));
            }
            top
        }
        Keys::Pattern => {
            let span = m.dp(300.0);
            let cy = m.dp(300.0) + span / 2.0;
            widgets::pattern_lock(&mut c, t, x, cy, span, &s.pattern);
            if !checking {
                out.push(Hit { rect: Rect::new(x - span, cy - span, span * 2.0, span * 2.0), action: Action::Pattern });
                pattern = Some((x, cy, span));
            }
            m.h as f32 - m.dp(26.0)
        }
        Keys::Digits => {
            let top = m.h as f32 - widgets::keypad_height(t);
            let caps = widgets::keypad(&mut c, t, f, l, top, s.key_held());
            if !checking {
                out.extend(caps.into_iter().map(|(rect, cap)| Hit { rect, action: Action::Type(cap) }));
            }
            top
        }
    };
    // The dots sit just above whatever types into them; a pattern is its
    // own showing of what has been drawn.
    if keys != Keys::Pattern {
        widgets::code_dots(&mut c, t, x, bottom - m.dp(92.0), s.code.chars().count());
    }
    let skip_cy = bottom - m.dp(34.0);
    c.text_center(&f.heading, l.pick(NOT_NOW), x, skip_cy, m.row_title_px, t.pal.accent);
    out.push(Hit { rect: Rect::new(x - m.dp(80.0), skip_cy - m.dp(24.0), m.dp(160.0), m.dp(48.0)), action: Action::Skip });
    Frame { header: false, pattern, ..Frame::new(t, c, out, l.pick(DECRYPT), 0.0) }
}

/// What a slot is shown as where the boot control could not be read.
const UNREADABLE: Text = ["Unreadable", "Не читается"];

/// Reboot target picker with a swipe-to-confirm control.
pub fn reboot(cx: &Ctx) -> Frame {
    let (t, f, s, l) = (cx.t, cx.f, cx.state, cx.state.lang);
    let mut c = page(cx);
    let mut out = Vec::new();
    let top = t.m.content_top;
    let mut y = widgets::section_first(&mut c, t, f, top, l.pick(["Reboot to", "Перезагрузить в"]));
    let rows: Vec<Row> = REBOOT_TARGETS
        .iter()
        .enumerate()
        .map(|(k, &(title, sub))| {
            let r = Row::new(l.pick(title)).trail(Trail::Check(s.reboot == k));
            match sub {
                Some(text) => r.subtitle(l.pick(text)),
                None => r,
            }
        })
        .collect();
    let targets = widgets::list(&mut c, t, f, y, &rows);
    let picks: Vec<Action> = (0..TARGETS.len()).map(Action::RebootTarget).collect();
    out.extend(hits(&targets.rows, &picks));
    y = targets.bottom;
    if let Some(slot) = cx.info.slot {
        let next = match &s.boot_slots {
            Some(Ok(slots)) if slots.active == slot => {
                let slot = slot.letter();
                tr!(l, "{slot} (current)", "{slot} (текущий)")
            }
            Some(Ok(slots)) => slots.active.letter().to_string(),
            Some(Err(_)) => l.pick(UNREADABLE).to_string(),
            None => String::new(),
        };
        y = widgets::section(&mut c, t, f, y, l.pick(["Slot", "Слот"]));
        let slots = widgets::list(&mut c, t, f, y, &[Row::new(l.pick(["Next boot", "При следующей загрузке"])).trail(Trail::Value(&next))]);
        out.extend(hits(&slots.rows, &[Action::Open(Screen::Slot)]));
        y = slots.bottom;
    }
    let (label, danger) = confirm_label(Confirm::Reboot, cx.state);
    let track = widgets::confirm(&mut c, t, f, y + t.m.dp(28.0), label, danger, s.confirm);
    out.push(Hit { rect: track, action: Action::Confirm(Confirm::Reboot) });
    Frame::new(t, c, out, l.pick(REBOOT), top)
}

/// Installing, opened from the home page: the system on the phone now, and
/// where a package can come from.
pub fn install(cx: &Ctx) -> Frame {
    let (t, f, i, l) = (cx.t, cx.f, cx.info, cx.state.lang);
    let mut c = long_page(cx);
    let top = t.m.content_top;
    let mut facts = Vec::new();
    if let Some(v) = &i.os_version {
        facts.push((l.pick(["Current system", "Текущая система"]), v.clone()));
    }
    if let Some(p) = &i.security_patch {
        facts.push((l.pick(SECURITY_PATCH), fmt::patch(l, p)));
    }
    if let Some(s) = i.slot {
        let (slot, other) = (s.letter(), s.other().letter());
        let detail = tr!(l, "{slot}, and a system update installs to slot {other}", "{slot}, а обновление системы устанавливается в слот {other}");
        facts.push((l.pick(CURRENT_SLOT), detail));
    }
    let ways = l.pick(["Ways to install", "Способы установки"]);
    let y = if facts.is_empty() {
        widgets::section_first(&mut c, t, f, top, ways)
    } else {
        let y = widgets::section_first(&mut c, t, f, top, l.pick(["This phone", "Этот телефон"]));
        let y = widgets::info_card(&mut c, t, f, y, &facts);
        widgets::section(&mut c, t, f, y, ways)
    };
    let mut rows = Vec::new();
    if let Some(note) = fmt::snapshot_note(l, &i.snapshot) {
        rows.push(Row::new(fmt::snapshot(l, &i.snapshot)).icon(ORANGE, Icon::Layers).subtitle(note).trail(Trail::None));
    }
    // Internal storage opens with the data partition, which the decrypt
    // page opens or tells why it does not.
    let data = &cx.state.data;
    let (storage, from_storage) = if data.storage_open() {
        (
            ["Choose a package or image in internal storage", "Выберите пакет или образ во внутренней памяти"],
            Action::Open(Screen::Choose(Place::Storage)),
        )
    } else if *data == Data::Erased {
        (DATA_ERASED, Action::Open(Screen::Decrypt))
    } else {
        (DECRYPT_NEEDED, Action::Open(Screen::Decrypt))
    };
    // The drive's page waits for one to be plugged in, and says why one
    // that is cannot be read.
    let usb = match &cx.state.usb_drive {
        Some(UsbDrive::Shown) => ["Choose a package or image on the USB drive", "Выберите пакет или образ на USB-накопителе"],
        Some(UsbDrive::Unreadable(_)) => USB_DRIVE_UNREADABLE,
        None => USB_DRIVE_WANTED,
    };
    rows.push(Row::new(install_title(l, Source::File(Place::Storage))).icon(BLUE, Icon::Phone).subtitle(l.pick(storage)));
    rows.push(Row::new(install_title(l, Source::File(Place::UsbDrive))).icon(PURPLE, Icon::UsbDrive).subtitle(l.pick(usb)));
    let sideload = l.pick(["Run adb sideload on the computer", "Запустите adb sideload на компьютере"]);
    rows.push(Row::new(install_title(l, Source::Computer)).icon(GREEN, Icon::Usb).subtitle(sideload));
    let methods = widgets::list(&mut c, t, f, y, &rows);
    let from_usb = Action::Open(Screen::Choose(Place::UsbDrive));
    let mut out = hits(&methods.rows[methods.rows.len() - 3..], &[from_storage, from_usb, Action::Sideload]);
    // A phone that can hold a second system installs one from here too.
    let what = l.pick(match cx.state.dual_boot {
        true => [
            "Installs A/B full OTA packages, flashable zips with an install script and second systems' packages, and flashes boot, init_boot, vendor_boot, recovery, dtbo and vbmeta partition images. A system update installs to the other slot, which starts after a reboot; a second system installs into the data partition, and Dual boot switches to it. Choosing a file lists only folders and .zip and .img files.",
            "Устанавливает полные пакеты OTA для A/B, zip-архивы для прошивки со скриптом установки и пакеты второй системы, а также прошивает образы разделов boot, init_boot, vendor_boot, recovery, dtbo и vbmeta. Обновление системы устанавливается в другой слот, который запускается после перезагрузки; вторая система устанавливается в раздел data, и «Вторая система» переключается на неё. При выборе файла показываются только папки и файлы .zip и .img.",
        ],
        false => [
            "Installs A/B full OTA packages and flashable zips with an install script, and flashes boot, init_boot, vendor_boot, recovery, dtbo and vbmeta partition images. A system update installs to the other slot, which starts after a reboot. Choosing a file lists only folders and .zip and .img files.",
            "Устанавливает полные пакеты OTA для A/B и zip-архивы для прошивки со скриптом установки, а также прошивает образы разделов boot, init_boot, vendor_boot, recovery, dtbo и vbmeta. Обновление системы устанавливается в другой слот, который запускается после перезагрузки. При выборе файла показываются только папки и файлы .zip и .img.",
        ],
    });
    let y = widgets::footnote(&mut c, t, f, methods.bottom, what);
    let y = widgets::section(&mut c, t, f, y, l.pick(["Options", "Параметры"]));
    let check = Row::new(l.pick(["Verify zip signatures", "Проверять подписи zip"])).trail(Trail::Switch(cx.state.check_signature));
    let signature = widgets::list(&mut c, t, f, y, &[check]);
    out.extend(hits(&signature.rows, &[Action::Toggle(Switch::Signature)]));
    let note = l.pick([
        "When on, only zips signed with this phone's official keys install.",
        "Когда включено, устанавливаются только zip-архивы, подписанные официальными ключами этого телефона.",
    ]);
    let mut y = widgets::footnote(&mut c, t, f, signature.bottom, note);
    if cx.state.recovery_partitions {
        let row = Row::new(l.pick(["Flash Skyward again", "Прошить Skyward снова"])).trail(Trail::Switch(cx.state.keep_recovery));
        let options = widgets::list(&mut c, t, f, y + t.m.card_gap, &[row]);
        out.extend(hits(&options.rows, &[Action::Toggle(Switch::KeepRecovery)]));
        let note = l.pick([
            "When on, Skyward goes back into the recovery partitions a system update or flashable zip writes over.",
            "Когда включено, Skyward возвращается в разделы recovery, которые перезаписывает обновление системы или zip-архив для прошивки.",
        ]);
        y = widgets::footnote(&mut c, t, f, options.bottom, note);
    }
    Frame::new(t, fit(c, t, y), out, l.pick(["Install", "Установить"]), top)
}

const SECURITY_PATCH: Text = ["Security patch", "Патч безопасности"];

/// How installing from `source` is named: on the install page, and as the
/// title of the page that installs.
fn install_title(l: Lang, source: Source) -> &'static str {
    l.pick(match source {
        Source::Computer => ["Install from a computer", "Установить с компьютера"],
        Source::File(Place::Storage) => ["Install from internal storage", "Установить из внутренней памяти"],
        Source::File(Place::UsbDrive) => ["Install from a USB drive", "Установить с USB-накопителя"],
        Source::Opened { .. } => ["Install from File manager", "Установить из файлового менеджера"],
        Source::Asked { security: false } => ["Install a system update", "Установить обновление системы"],
        Source::Asked { security: true } => ["Install a security update", "Установить обновление безопасности"],
        Source::Rescue => ["Rescue mode", "Режим спасения"],
    })
}

/// What the package being installed is called on its page: the file's own
/// name where the page was given one, and the version it brings otherwise.
fn package_named<'a>(s: &'a State, version: Option<&'a str>) -> Option<&'a str> {
    s.package_name.as_deref().or(version)
}

/// What a place is called at the top of its folders.
fn place_name(l: Lang, place: Place) -> &'static str {
    l.pick(match place {
        Place::Storage => ["Internal storage", "Внутренняя память"],
        Place::UsbDrive => ["USB drive", "USB-накопитель"],
    })
}

/// A one-line card on how things stand.
fn standing<'a>(title: &'a str, color: Rgba, icon: Icon, detail: &'a str) -> Row<'a> {
    Row::new(title).icon(color, icon).subtitle(detail).trail(Trail::None)
}

fn share(done: u64, total: u64) -> f32 {
    if total == 0 {
        0.0
    } else {
        (done as f64 / total as f64) as f32
    }
}

fn percent(share: f32) -> u32 {
    (share * 100.0).floor() as u32
}

/// Where an update stands, and the share of its step done when known.
fn update_stage(l: Lang, stage: Option<&Stage>) -> (String, Option<f32>) {
    match stage {
        Some(Stage::Writing { done, total }) => {
            let s = share(*done, *total);
            let p = percent(s);
            (tr!(l, "Step 1 of 3: writing the new system {p}%", "Шаг 1 из 3: запись новой системы {p}%"), Some(s))
        }
        Some(Stage::Verifying { index, count, partition }) => {
            let n = index + 1;
            let said = tr!(l, "Step 2 of 3: verifying {partition} ({n}/{count})", "Шаг 2 из 3: проверка {partition} ({n}/{count})");
            (said, Some(*index as f32 / *count as f32))
        }
        Some(Stage::Finishing) => (l.pick(["Step 3 of 3: finishing", "Шаг 3 из 3: завершение"]).into(), None),
        Some(Stage::Reflashing { done, total }) => (l.pick(REFLASHING).into(), Some(*done as f32 / *total as f32)),
        Some(Stage::Flashing { .. } | Stage::Script { .. } | Stage::Signature { .. } | Stage::Allocating { .. } | Stage::Root { .. } | Stage::Storing) | None => {
            (l.pick(GETTING_READY).into(), None)
        }
    }
}

/// Where installing a system beside the installed one stands, and the share
/// of its step done when known.
fn system_stage(l: Lang, stage: Option<&Stage>) -> (String, Option<f32>) {
    match stage {
        Some(Stage::Allocating { done, total }) => {
            let s = share(*done, *total);
            let p = percent(s);
            (tr!(l, "Step 1 of 3: setting room aside in the data partition {p}%", "Шаг 1 из 3: выделение места в разделе data {p}%"), Some(s))
        }
        Some(Stage::Root { done, total }) => {
            let s = share(*done, *total);
            let p = percent(s);
            let said = match done * 2 <= *total {
                true => tr!(l, "Step 2 of 3: writing the system {p}%", "Шаг 2 из 3: запись системы {p}%"),
                false => tr!(l, "Step 2 of 3: verifying the system {p}%", "Шаг 2 из 3: проверка системы {p}%"),
            };
            (said, Some(s))
        }
        Some(Stage::Storing) => (l.pick(["Step 3 of 3: keeping its boot images", "Шаг 3 из 3: сохранение его boot-образов"]).into(), None),
        _ => (l.pick(GETTING_READY).into(), None),
    }
}

const REFLASHING: Text = ["Flashing Skyward again", "Повторная прошивка Skyward"];

/// What the card says while an install script runs, and how far it is: the
/// last line the script showed and how far it says it is, then Skyward
/// going back into the recovery partitions it wrote over.
fn script_stage<'a>(l: Lang, stage: Option<&Stage>, said: &'a [String]) -> (&'a str, Option<f32>) {
    let running = l.pick(["Running the install script", "Запуск скрипта установки"]);
    let last = said.iter().rev().map(|line| line.trim()).find(|line| !line.is_empty()).unwrap_or(running);
    match stage {
        Some(Stage::Reflashing { done, total }) => (l.pick(REFLASHING), Some(*done as f32 / *total as f32)),
        Some(Stage::Script { thousandths }) => (last, Some(f32::from(*thousandths) / 1000.0)),
        _ => (last, None),
    }
}

/// How Skyward went back into the recovery partitions the install wrote
/// over, where it did.
fn reflashed(c: &mut Canvas, cx: &Ctx, y: f32) -> f32 {
    let (t, f, l) = (cx.t, cx.f, cx.state.lang);
    match &cx.state.reflash {
        None => y,
        Some(Reflash::Written(written)) => {
            let slots = fmt::slots(l, written);
            let said = match (l, written.len()) {
                (Lang::En, 1) => format!("Skyward went back into {slots}'s recovery partition."),
                _ => tr!(l, "Skyward went back into the recovery partitions of {slots}.", "Skyward вернулся в разделы recovery {slots}."),
            };
            widgets::footnote(c, t, f, y, &said)
        }
        Some(Reflash::Failed(why)) => {
            let note = l.pick(["Couldn't flash Skyward back into the recovery partitions.", "Не удалось прошить Skyward обратно в разделы recovery."]);
            let y = widgets::footnote(c, t, f, y, note);
            widgets::footnote(c, t, f, y, why)
        }
    }
}

/// What the install script showed, where there is any: its heading, and
/// under it the lines as the page's long list, the last thing on the page.
fn script_output(c: &mut Canvas, cx: &Ctx, y: f32, rows: &mut Option<Rows>) -> f32 {
    let lines = &cx.state.script_output;
    if lines.is_empty() {
        return y;
    }
    let y = widgets::section(c, cx.t, cx.f, y, cx.state.lang.pick(["Install script output", "Вывод скрипта установки"]));
    *rows = Some(Rows { edges: widgets::output_edges(c, cx.t, cx.f, y, lines), action: None });
    y
}

fn pick_title(l: Lang, pick: Pick, current: Option<Slot>) -> String {
    let letter = |s: Option<Slot>| s.map(|s| format!(" {}", s.letter())).unwrap_or_default();
    match pick {
        Pick::Current => format!("{}{}", l.pick(CURRENT_SLOT), letter(current)),
        Pick::Other => format!("{}{}", l.pick(["Other slot", "Другой слот"]), letter(current.map(Slot::other))),
        Pick::Both => l.pick(["Both slots", "Оба слота"]).into(),
    }
}

/// Words the installing page says of more than one of its steps.
const SWITCHING_USB: Text = ["Switching USB mode", "Переключение режима USB"];
const READING_FILE: Text = ["Reading the file", "Чтение файла"];
const WAITING: Text = ["Waiting for the computer", "Ожидание компьютера"];
const CONNECT: Text = ["Connect a computer with a cable, and run these commands on it", "Подключите компьютер кабелем и выполните на нём эти команды"];
const RUN_ON_COMPUTER: Text = ["Run on the computer", "Запустите на компьютере"];
const KEEP_CABLE: Text = ["Keep the cable in until it's read.", "Не отключайте кабель, пока не будет прочитано."];
const KEEP_DRIVE: Text = ["Keep the USB drive in until it's read.", "Не извлекайте USB-накопитель, пока он не будет прочитан."];
const GETTING_READY: Text = ["Getting ready", "Подготовка"];

/// Installing a package step by step: receiving what the computer sends
/// with `adb sideload`, or reading the file chosen, then installing it.
pub fn installing(cx: &Ctx) -> Frame {
    let (t, f, s, i, l) = (cx.t, cx.f, cx.state, cx.info, cx.state.lang);
    let mut c = long_page(cx);
    let (mut out, mut rows) = (Vec::new(), None);
    let top = t.m.content_top;
    let computer = s.source == Source::Computer;
    let bottom = match &s.install {
        Install::Idle | Install::Starting | Install::Ending if computer => {
            let row = standing(l.pick(SWITCHING_USB), GREEN, Icon::Usb, l.pick(["From adb debugging to sideload", "Из отладки adb в sideload"]));
            widgets::list(&mut c, t, f, top, &[row]).bottom
        }
        Install::Idle | Install::Starting | Install::Ending if s.source == Source::Rescue => {
            let row = standing(l.pick(SWITCHING_USB), GREEN, Icon::Usb, l.pick(["From adb debugging to rescue mode", "Из отладки adb в режим спасения"]));
            widgets::list(&mut c, t, f, top, &[row]).bottom
        }
        Install::Idle | Install::Starting | Install::Ending => {
            widgets::list(&mut c, t, f, top, &[Row::new(l.pick(READING_FILE)).icon(GREEN, Icon::Install).trail(Trail::None)]).bottom
        }
        Install::Opening => {
            let opening = Row::new(l.pick(["Opening the data partition", "Открытие раздела data"])).icon(GREEN, Icon::Lock).trail(Trail::None);
            let y = widgets::list(&mut c, t, f, top, &[opening]).bottom;
            asked_package(&mut c, cx, y)
        }
        Install::Locked(_) => {
            let row = standing(l.pick(["The package can't be read yet", "Пакет пока не читается"]), ORANGE, Icon::Lock, l.pick(DECRYPT_NEEDED));
            let y = widgets::list(&mut c, t, f, top, &[row]).bottom;
            let open = widgets::list(&mut c, t, f, y + t.m.card_gap, &[Row::new(l.pick(DECRYPT))]);
            out.extend(hits(&open.rows, &[Action::Open(Screen::Decrypt)]));
            let y = widgets::footnote(&mut c, t, f, open.bottom, l.pick(["It installs once the data partition is decrypted.", "Установка начнётся после расшифровки раздела data."]));
            asked_package(&mut c, cx, y)
        }
        Install::Waiting if s.source == Source::Rescue => {
            let row = standing(l.pick(WAITING), GREEN, Icon::Usb, l.pick(CONNECT));
            let y = widgets::list(&mut c, t, f, top, &[row]).bottom;
            let y = widgets::section(&mut c, t, f, y, l.pick(RUN_ON_COMPUTER));
            let y = widgets::info_card(&mut c, t, f, y, &[
                (
                    l.pick(["adb rescue install <file>", "adb rescue install <file>"]),
                    l.pick([
                        "With a system update package, flashable zip or partition image as <file>",
                        "С пакетом обновления системы, zip-архивом для прошивки или образом раздела в качестве <file>",
                    ])
                    .into(),
                ),
                ("adb rescue getprop", l.pick(["Reads this phone's versions and device details", "Читает версии и сведения об устройстве этого телефона"]).into()),
                ("adb rescue wipe userdata", l.pick(["Wipes all of this phone's data", "Очищает все данные этого телефона"]).into()),
            ]);
            let note = l.pick([
                "The computer needs Android SDK Platform-Tools. Going back leaves rescue mode and switches USB back to adb debugging.",
                "Компьютеру нужны Android SDK Platform-Tools. Возврат выходит из режима rescue и переключает USB обратно на отладку adb.",
            ]);
            widgets::footnote(&mut c, t, f, y, note)
        }
        Install::Waiting => {
            let row = standing(l.pick(WAITING), GREEN, Icon::Usb, l.pick(CONNECT));
            let y = widgets::list(&mut c, t, f, top, &[row]).bottom;
            let y = widgets::section(&mut c, t, f, y, l.pick(RUN_ON_COMPUTER));
            let y = widgets::info_card(&mut c, t, f, y, &[(
                l.pick(["adb sideload <file>", "adb sideload <file>"]),
                l.pick([
                    "With a system update package, flashable zip or partition image as <file>, such as boot.img",
                    "С пакетом обновления системы, zip-архивом для прошивки или образом раздела в качестве <file>, например boot.img",
                ])
                .into(),
            )]);
            let note = l.pick([
                "The computer needs Android SDK Platform-Tools. Going back stops waiting and switches USB back to adb debugging.",
                "Компьютеру нужны Android SDK Platform-Tools. Возврат прекращает ожидание и переключает USB обратно на отладку adb.",
            ]);
            widgets::footnote(&mut c, t, f, y, note)
        }
        Install::Checking { stage: Some(Stage::Signature { done, total }), .. } => {
            let read = share(*done, *total);
            let p = percent(read);
            let detail = tr!(l, "{p}% read", "{p}% прочитано");
            let y = widgets::progress_card(&mut c, t, f, top, l.pick(["Verifying the signature", "Проверка подписи"]), &detail, Some(read));
            let note = l.pick(match s.source {
                Source::Computer => [
                    "Keep the cable in until it's read. Going back cancels the install and ends adb sideload on the computer.",
                    "Не отключайте кабель, пока не будет прочитано. Возврат отменит установку и завершит adb sideload на компьютере.",
                ],
                Source::Rescue => [
                    "Keep the cable in until it's read. Going back cancels the install and leaves rescue mode.",
                    "Не отключайте кабель, пока не будет прочитано. Возврат отменит установку и выйдет из режима спасения.",
                ],
                Source::File(Place::Storage) | Source::Opened { drive: false } | Source::Asked { .. } => ["Going back cancels the install.", "Возврат отменит установку."],
                Source::File(Place::UsbDrive) | Source::Opened { drive: true } => [
                    "Keep the USB drive in until it's read. Going back cancels the install.",
                    "Не извлекайте USB-накопитель, пока он не будет прочитан. Возврат отменит установку.",
                ],
            });
            widgets::footnote(&mut c, t, f, y, note)
        }
        Install::Checking { size, .. } => {
            let size = fmt::bytes(*size);
            let detail = tr!(l, "File size {size}", "Размер файла {size}");
            let y = widgets::list(&mut c, t, f, top, &[standing(l.pick(READING_FILE), GREEN, Icon::Install, &detail)]).bottom;
            match s.source {
                Source::Computer | Source::Rescue => widgets::footnote(&mut c, t, f, y, l.pick(KEEP_CABLE)),
                Source::File(Place::Storage) | Source::Opened { drive: false } | Source::Asked { .. } => y,
                Source::File(Place::UsbDrive) | Source::Opened { drive: true } => widgets::footnote(&mut c, t, f, y, l.pick(KEEP_DRIVE)),
            }
        }
        Install::Choosing(choice) => {
            let (name, holds) = fmt::image_kind(l, choice.kind);
            let size = fmt::bytes(choice.size);
            let detail = tr!(l, "{size} · {holds}", "{size} · {holds}");
            let y = widgets::list(&mut c, t, f, top, &[standing(name, GREEN, Icon::Install, &detail)]).bottom;
            let y = widgets::section(&mut c, t, f, y, l.pick(["Partition", "Раздел"]));
            let rows: Vec<Row> = choice
                .partitions
                .iter()
                .enumerate()
                .map(|(k, p)| Row::new(p).subtitle(fmt::partition_role(l, p)).trail(Trail::Check(k == choice.partition)))
                .collect();
            let parts = widgets::list(&mut c, t, f, y, &rows);
            let choose: Vec<Action> = (0..choice.partitions.len()).map(Action::FlashPartition).collect();
            out.extend(hits(&parts.rows, &choose));
            let y = widgets::section(&mut c, t, f, parts.bottom, l.pick(["Slot", "Слот"]));
            let titles = Pick::ALL.map(|p| pick_title(l, p, i.slot));
            let rows: Vec<Row> =
                Pick::ALL.iter().zip(&titles).map(|(p, title)| Row::new(title).trail(Trail::Check(*p == choice.pick))).collect();
            let picks = widgets::list(&mut c, t, f, y, &rows);
            out.extend(hits(&picks.rows, &Pick::ALL.map(Action::FlashSlots)));
            let (label, danger) = confirm_label(Confirm::Flash, cx.state);
            let track = widgets::confirm(&mut c, t, f, picks.bottom + t.m.dp(28.0), label, danger, s.confirm);
            out.push(Hit { rect: track, action: Action::Confirm(Confirm::Flash) });
            let note = l.pick(match s.source {
                Source::Computer => [
                    "Flashing writes over what the partition holds. Going back cancels it and ends adb sideload on the computer.",
                    "Прошивка перезаписывает содержимое раздела. Возврат отменит её и завершит adb sideload на компьютере.",
                ],
                Source::Rescue => [
                    "Flashing writes over what the partition holds. Going back cancels it and leaves rescue mode.",
                    "Прошивка перезаписывает содержимое раздела. Возврат отменит её и выйдет из режима спасения.",
                ],
                Source::File(_) | Source::Opened { .. } | Source::Asked { .. } => [
                    "Flashing writes over what the partition holds. Going back cancels it.",
                    "Прошивка перезаписывает содержимое раздела. Возврат отменит её.",
                ],
            });
            widgets::footnote(&mut c, t, f, track.y + track.h, note)
        }
        Install::Confirming { version, wipe } => {
            let named = package_named(s, version.as_deref());
            let (title, color, icon, detail, which) = if *wipe {
                let reset = factory_reset(s);
                let detail = match named {
                    Some(v) => tr!(l, "Installing {v} then {reset}.", "Установка {v}, затем {reset}."),
                    None => tr!(l, "Installing this update then {reset}.", "Установка этого обновления, затем {reset}."),
                };
                (l.pick(["All data has to be wiped", "Все данные необходимо очистить"]).to_string(), ORANGE, Icon::Wipe, detail, Confirm::InstallWipe)
            } else {
                let title = match named {
                    Some(v) => tr!(l, "Install {v}", "Установить {v}"),
                    None => l.pick(["Install the system update", "Установить обновление системы"]).into(),
                };
                let detail = match i.slot.map(|s| s.other().letter()) {
                    Some(slot) => tr!(
                        l,
                        "The new system installs to slot {slot}, which starts after a reboot.",
                        "Новая система устанавливается в слот {slot}, который запустится после перезагрузки.",
                    ),
                    None => l.pick(["The new system starts after a reboot.", "Новая система запустится после перезагрузки."]).into(),
                };
                (title, GREEN, Icon::Install, detail, Confirm::Install)
            };
            let y = widgets::list(&mut c, t, f, top, &[standing(&title, color, icon, &detail)]).bottom;
            let (label, danger) = confirm_label(which, cx.state);
            let track = widgets::confirm(&mut c, t, f, y + t.m.dp(28.0), label, danger, s.confirm);
            out.push(Hit { rect: track, action: Action::Confirm(which) });
            let note = l.pick(["Going back cancels the install without writing anything.", "Возврат отменяет установку, ничего не записывая."]);
            widgets::footnote(&mut c, t, f, track.y + track.h, note)
        }
        Install::Updating { version, stage, overwriting } => {
            let title = match package_named(s, version.as_deref()) {
                Some(v) => tr!(l, "Installing {v}", "Установка {v}"),
                None => l.pick(["Installing the system update", "Установка обновления системы"]).into(),
            };
            let (detail, done) = update_stage(l, stage.as_ref());
            let y = widgets::progress_card(&mut c, t, f, top, &title, &detail, done);
            let until = until_installed(l, s.source);
            let note = match i.slot.map(|s| s.other().letter()) {
                Some(slot) => tr!(l, "{until} The new system installs to slot {slot}.", "{until} Новая система установится в слот {slot}."),
                None => until.to_string(),
            };
            let y = widgets::footnote(&mut c, t, f, y, &note);
            if *overwriting {
                let note = l.pick([
                    "The current system is being written over, so a reboot before the install finishes starts no system.",
                    "Текущая система перезаписывается, поэтому перезагрузка до завершения установки не запустит ни одну систему.",
                ]);
                widgets::footnote(&mut c, t, f, y, note)
            } else {
                y
            }
        }
        Install::Flashing { partition, stage } => {
            let (detail, done) = match stage {
                Some(Stage::Flashing { done, total }) => {
                    let s = share(*done, *total);
                    let p = percent(s);
                    let detail = match done * 2 < *total {
                        true => tr!(l, "Writing {p}%", "Запись {p}%"),
                        false => tr!(l, "Verifying {p}%", "Проверка {p}%"),
                    };
                    (detail, Some(s))
                }
                _ => (l.pick(GETTING_READY).into(), None),
            };
            let title = tr!(l, "Flashing {partition}", "Прошивка {partition}");
            let y = widgets::progress_card(&mut c, t, f, top, &title, &detail, done);
            let keep = l.pick(match s.source {
                Source::Computer | Source::Rescue => ["Keep the cable in until it's flashed.", "Не отключайте кабель, пока не прошьётся."],
                Source::File(Place::Storage) | Source::Opened { drive: false } | Source::Asked { .. } => {
                    ["Don't power off until it's flashed.", "Не выключайте, пока не прошито."]
                }
                Source::File(Place::UsbDrive) | Source::Opened { drive: true } => {
                    ["Keep the USB drive in until it's flashed.", "Не извлекайте USB-накопитель, пока он не прошьётся."]
                }
            });
            widgets::footnote(&mut c, t, f, y, keep)
        }
        Install::SystemChoosing(choice) => {
            let version = &choice.version;
            let detail = tr!(l, "{version}, to install as the second system", "{version}, для установки как второй системы");
            let y = widgets::list(&mut c, t, f, top, &[standing(&choice.name, PURPLE, Icon::Systems, &detail)]).bottom;
            let y = widgets::section(&mut c, t, f, y, l.pick(["Room for it", "Место для неё"]));
            let sizes: Vec<String> = choice.sizes.iter().map(|&b| fmt::bytes(b)).collect();
            let least = l.pick(["The least the system takes", "Минимум, который занимает система"]);
            let rows: Vec<Row> = sizes
                .iter()
                .enumerate()
                .map(|(k, size)| {
                    let row = Row::new(size).trail(Trail::Check(k == choice.size));
                    if k == 0 { row.subtitle(least) } else { row }
                })
                .collect();
            let list = widgets::list(&mut c, t, f, y, &rows);
            let picks: Vec<Action> = (0..choice.sizes.len()).map(Action::SystemSize).collect();
            out.extend(hits(&list.rows, &picks));
            let note = l.pick([
                "The room chosen is set aside in the data partition for the second system's files, can't be changed once it's installed, and comes back when the second system is removed. Nothing of the main system's partitions is written.",
                "Выбранное место выделяется в разделе data для файлов второй системы, его нельзя изменить после установки, и оно возвращается при удалении второй системы. Разделы основной системы не записываются.",
            ]);
            let y = widgets::footnote(&mut c, t, f, list.bottom, note);
            let (label, danger) = confirm_label(Confirm::InstallSystem, cx.state);
            let track = widgets::confirm(&mut c, t, f, y + t.m.dp(24.0), label, danger, s.confirm);
            out.push(Hit { rect: track, action: Action::Confirm(Confirm::InstallSystem) });
            let note = l.pick(["Going back cancels the install without writing anything.", "Возврат отменяет установку, ничего не записывая."]);
            widgets::footnote(&mut c, t, f, track.y + track.h, note)
        }
        Install::Systeming { name, stage } => {
            let title = tr!(l, "Installing {name}", "Установка {name}");
            let (detail, done) = system_stage(l, stage.as_ref());
            let y = widgets::progress_card(&mut c, t, f, top, &title, &detail, done);
            widgets::footnote(&mut c, t, f, y, until_installed(l, s.source))
        }
        Install::ConfirmingScript => {
            let name = s.package_name.as_deref().unwrap_or(l.pick(["Flashable zip", "zip-архив для прошивки"]));
            let detail = l.pick([
                "This zip runs the install script it holds, which can rewrite any partition of this phone.",
                "Этот zip-архив запускает содержащийся в нём скрипт установки, который может перезаписать любой раздел этого телефона.",
            ]);
            let mut y = widgets::list(&mut c, t, f, top, &[standing(name, ORANGE, Icon::Install, detail)]).bottom;
            if let Some(note) = fmt::snapshot_script_note(l, &i.snapshot) {
                y = widgets::footnote(&mut c, t, f, y, note);
            }
            let (label, danger) = confirm_label(Confirm::RunScript, cx.state);
            let track = widgets::confirm(&mut c, t, f, y + t.m.dp(28.0), label, danger, s.confirm);
            out.push(Hit { rect: track, action: Action::Confirm(Confirm::RunScript) });
            let note = l.pick(["Going back cancels the install without running the script.", "Возврат отменяет установку, не запуская скрипт."]);
            widgets::footnote(&mut c, t, f, track.y + track.h, note)
        }
        Install::Scripting { stage, .. } => {
            let (detail, done) = script_stage(l, stage.as_ref(), &s.script_output);
            let y = widgets::progress_card(&mut c, t, f, top, l.pick(["Installing", "Установка"]), detail, done);
            let y = widgets::footnote(&mut c, t, f, y, until_installed(l, s.source));
            script_output(&mut c, cx, y, &mut rows)
        }
        Install::Wiping => {
            let y = widgets::progress_card(&mut c, t, f, top, l.pick(WIPING), &capitalized(l, &factory_reset(s)), None);
            widgets::footnote(&mut c, t, f, y, l.pick(KEEP_POWER))
        }
        Install::Wiped(Ok(())) => {
            let made_anew = l.pick(["The data partition is made anew when the phone next starts", "Раздел data создаётся заново при следующем запуске телефона"]);
            let y = widgets::list(&mut c, t, f, top, &[standing(l.pick(["Wiped", "Очищено"]), GREEN, Icon::Wipe, made_anew)]).bottom;
            widgets::footnote(&mut c, t, f, y, l.pick(MORE_COMMANDS))
        }
        Install::Wiped(Err(stop)) => {
            let (what, detail) = fmt::erase_stop(l, stop);
            let mut y = widgets::list(&mut c, t, f, top, &[standing(l.pick(WIPE_INCOMPLETE), RED, Icon::Wipe, &what)]).bottom;
            if let Some(d) = detail {
                y = widgets::footnote(&mut c, t, f, y, &d);
            }
            widgets::footnote(&mut c, t, f, y, l.pick(MORE_COMMANDS))
        }
        Install::Finished { outcome, damage } => finished(&mut c, cx, top, outcome, *damage, &mut out, &mut rows),
    };
    // What the system asked for says why, where it said.
    let bottom = match (s.source, &s.asked.reason) {
        (Source::Asked { .. }, Some(reason)) => widgets::info_card(&mut c, t, f, bottom + t.m.dp(12.0), &[(l.pick(REASON), reason.clone())]),
        _ => bottom,
    };
    Frame { rows, ..Frame::new(t, fit(c, t, bottom), out, install_title(l, s.source), top) }
}

/// What a request the system made gives as why.
const REASON: Text = ["Reason", "Причина"];

const WIPING: Text = ["Wiping", "Очистка"];
const WIPE_INCOMPLETE: Text = ["Wipe incomplete", "Очистка не завершена"];
const KEEP_POWER: Text = ["Don't power off until it's wiped.", "Не выключайте, пока не очищено."];

/// What rescue says once a command is done with: the computer may send
/// another.
const MORE_COMMANDS: Text = ["You can send more commands from the computer.", "Можно отправлять больше команд с компьютера."];

/// The package the system asked for, by the path it gave, at `y`.
fn asked_package(c: &mut Canvas, cx: &Ctx, y: f32) -> f32 {
    let name = cx.state.package_name.clone().unwrap_or_default();
    widgets::info_card(c, cx.t, cx.f, y + cx.t.m.dp(12.0), &[(cx.state.lang.pick(["Package", "Пакет"]), name)])
}

/// What not to do while a package from `source` is installed.
fn until_installed(l: Lang, source: Source) -> &'static str {
    l.pick(match source {
        Source::Computer | Source::Rescue => {
            [
                "Keep the cable in and don't power off until it's installed.",
                "Не отключайте кабель и не выключайте питание, пока не установится.",
            ]
        }
        Source::File(Place::Storage) | Source::Opened { drive: false } | Source::Asked { .. } => {
            ["Don't power off until it's installed.", "Не выключайте, пока не установлено."]
        }
        Source::File(Place::UsbDrive) | Source::Opened { drive: true } => {
            [
                "Keep the USB drive in and don't power off until it's installed.",
                "Не извлекайте USB-накопитель и не выключайте питание, пока он не установится.",
            ]
        }
    })
}

/// A phrase that opens a line on its own: in English with its first letter
/// a capital.
fn capitalized(l: Lang, phrase: &str) -> String {
    let mut chars = phrase.chars();
    match (l, chars.next()) {
        (Lang::En, Some(first)) => first.to_uppercase().chain(chars).collect(),
        _ => phrase.to_string(),
    }
}

/// How an install ended, and where to go from there: another package from
/// the computer, or back to choose another file.
fn finished(
    c: &mut Canvas,
    cx: &Ctx,
    top: f32,
    outcome: &Outcome,
    damage: Option<Damage>,
    out: &mut Vec<Hit>,
    rows: &mut Option<Rows>,
) -> f32 {
    let (t, f, source, l) = (cx.t, cx.f, cx.state.source, cx.state.lang);
    let gap = t.m.card_gap;
    let choose_another = ["Choose another file", "Выбрать другой файл"];
    let (again, another) = match source {
        Source::Computer => (Row::new(l.pick(["Send again", "Отправить снова"])).icon(GREEN, Icon::Usb), Action::Sideload),
        Source::File(_) | Source::Opened { .. } => (Row::new(l.pick(choose_another)).icon(GREEN, Icon::Folder), Action::Back),
        Source::Asked { .. } => (Row::new(l.pick(choose_another)).icon(GREEN, Icon::Folder), Action::Open(Screen::Install)),
        Source::Rescue => (Row::new(l.pick(["Leave rescue mode", "Выйти из режима спасения"])).icon(GREEN, Icon::Usb), Action::Back),
    };
    let restart = Row::new(l.pick(["Reboot to system", "Перезагрузить в систему"])).icon(ORANGE, Icon::Power);
    let installed = l.pick(["Installed", "Установлено"]);
    let nothing_written = l.pick(["Nothing was written.", "Ничего не было записано."]);
    match outcome {
        Outcome::Installed { slot, .. } => {
            let slot = slot.letter();
            let detail = tr!(
                l,
                "The new system is installed to slot {slot}, which starts after a reboot.",
                "Новая система установлена в слот {slot}, который запустится после перезагрузки.",
            );
            let y = widgets::list(c, t, f, top, &[standing(installed, GREEN, Icon::Install, &detail)]).bottom;
            let next = widgets::list(c, t, f, y + gap, &[restart]);
            out.extend(hits(&next.rows, &[Action::RebootTo(Target::System)]));
            let y = reflashed(c, cx, next.bottom);
            afterwards_of(c, cx, y, true)
        }
        Outcome::Flashed { partition, slots } => {
            let slots = fmt::slots(l, slots);
            let detail = tr!(l, "Flashed to {partition} in {slots}.", "Прошито в {partition} в {slots}.");
            let y = widgets::list(c, t, f, top, &[standing(l.pick(["Flashed", "Прошито"]), GREEN, Icon::Install, &detail)]).bottom;
            let mut rows = vec![restart];
            let mut acts = vec![Action::RebootTo(Target::System)];
            if *partition == "recovery" {
                rows.push(Row::new(l.pick(["Reboot to recovery", "Перезагрузить в recovery"])).icon(ORANGE, Icon::Power));
                acts.push(Action::RebootTo(Target::Recovery));
            }
            rows.push(again);
            acts.push(another);
            let next = widgets::list(c, t, f, y + gap, &rows);
            out.extend(hits(&next.rows, &acts));
            afterwards_of(c, cx, next.bottom, true)
        }
        Outcome::Ran { .. } => {
            let ran = l.pick(["The install script ran to its end.", "Скрипт установки выполнился до конца."]);
            let y = widgets::list(c, t, f, top, &[standing(installed, GREEN, Icon::Install, ran)]).bottom;
            let next = widgets::list(c, t, f, y + gap, &[restart, again]);
            out.extend(hits(&next.rows, &[Action::RebootTo(Target::System), another]));
            let y = reflashed(c, cx, next.bottom);
            let y = afterwards_of(c, cx, y, true);
            script_output(c, cx, y, rows)
        }
        Outcome::SystemInstalled { name } => {
            let detail = tr!(
                l,
                "{name} is installed as the second system. Switch to it from Dual boot.",
                "{name} установлена как вторая система. Переключитесь на неё в разделе «Вторая система».",
            );
            let y = widgets::list(c, t, f, top, &[standing(installed, GREEN, Icon::Systems, &detail)]).bottom;
            let next = widgets::list(c, t, f, y + gap, &[Row::new(l.pick(DUAL_BOOT)).icon(PURPLE, Icon::Systems)]);
            out.extend(hits(&next.rows, &[Action::Open(Screen::Systems)]));
            afterwards_of(c, cx, next.bottom, true)
        }
        Outcome::Cancelled => {
            let y = widgets::list(c, t, f, top, &[standing(l.pick(["Cancelled", "Отменено"]), SLATE, Icon::Install, nothing_written)]).bottom;
            let next = widgets::list(c, t, f, y + gap, &[again]);
            out.extend(hits(&next.rows, &[another]));
            next.bottom
        }
        Outcome::Refused(r) => {
            let reason = fmt::refusal(l, r, &cx.info.codename, source, cx.state.time_zone);
            let y = widgets::list(c, t, f, top, &[standing(l.pick(["Can't install", "Не удаётся установить"]), ORANGE, Icon::Install, &reason)]).bottom;
            // What the system asked for and did not go in leaves the phone
            // where it was, from which it may go back to the system.
            let (rows, acts) = match source {
                Source::Asked { .. } => (vec![again, restart], vec![another, Action::RebootTo(Target::System)]),
                _ => (vec![again], vec![another]),
            };
            let next = widgets::list(c, t, f, y + gap, &rows);
            out.extend(hits(&next.rows, &acts));
            let y = widgets::footnote(c, t, f, next.bottom, nothing_written);
            afterwards_of(c, cx, y, false)
        }
        Outcome::Failed(failure) => {
            let (what, detail) = fmt::failure(l, failure, source);
            let y = widgets::list(c, t, f, top, &[standing(l.pick(["Install failed", "Установка не удалась"]), RED, Icon::Install, &what)]).bottom;
            let next = widgets::list(c, t, f, y + gap, &[again, restart]);
            out.extend(hits(&next.rows, &[another, Action::RebootTo(Target::System)]));
            // The package is sent again, or installed or flashed again.
            let (install, flash) = match source {
                Source::Computer | Source::Rescue => (["Send", "Отправить"], ["Send", "Отправить"]),
                Source::File(_) | Source::Opened { .. } | Source::Asked { .. } => (["Install", "Установить"], ["Flash", "Прошить"]),
            };
            let (install, flash) = (l.pick(install), l.pick(flash));
            let warning = match damage {
                Some(Damage::System) => tr!(
                    l,
                    "The current system was partly written over, so a reboot starts no system. {install} the full OTA package again, and reboot once it's installed.",
                    "Текущая система была частично перезаписана, поэтому перезагрузка не запустит ни одну систему. {install} полный пакет OTA заново и перезагрузитесь после его установки.",
                ),
                Some(Damage::Partition(p)) => tr!(
                    l,
                    "{p} was only partly written, so the phone may not start properly after a reboot. {flash} the image again, and reboot once it's flashed.",
                    "{p} записан лишь частично, поэтому после перезагрузки телефон может не запуститься правильно. {flash} образ ещё раз и перезагрузитесь после прошивки.",
                ),
                Some(Damage::Script) => tr!(
                    l,
                    "This phone's partitions may have been partly rewritten, so it may not start properly after a reboot. {install} the zip again, and reboot once it's installed.",
                    "Разделы этого телефона могли быть частично перезаписаны, поэтому после перезагрузки он может не запуститься правильно. {install} zip-архив ещё раз и перезагрузитесь после установки.",
                ),
                None => l.pick(["The current system is untouched.", "Текущая система не тронута."]).into(),
            };
            let mut y = widgets::footnote(c, t, f, next.bottom, &warning);
            if let Some(d) = detail {
                y = widgets::footnote(c, t, f, y, &d);
            }
            let y = reflashed(c, cx, y);
            let y = afterwards_of(c, cx, y, false);
            script_output(c, cx, y, rows)
        }
    }
}

/// What comes after an install, where it is not the user's to choose:
/// rescue waits for the computer's next command, and a package the system
/// asked for, once in, has the phone leave on its own.
fn afterwards_of(c: &mut Canvas, cx: &Ctx, y: f32, installed: bool) -> f32 {
    let l = cx.state.lang;
    let leaving = match cx.state.asked.shutdown_after {
        true => ["Powering off shortly.", "Скоро выключение."],
        false => ["Rebooting shortly.", "Скоро перезагрузка."],
    };
    match cx.state.source {
        Source::Rescue => widgets::footnote(c, cx.t, cx.f, y, l.pick(MORE_COMMANDS)),
        Source::Asked { .. } if installed => widgets::footnote(c, cx.t, cx.f, y, l.pick(leaving)),
        _ => y,
    }
}

/// What the wipe page lists of [`WIPES`] beyond everything, under the
/// partition each is in.
const WIPE_GROUPS: [(Text, &[Wipe]); 2] = [
    (DATA_PART, &[Wipe::AppData, Wipe::Media, Wipe::AppCaches, Wipe::SystemCaches, Wipe::Art, Wipe::Logs, Wipe::Updates]),
    (["cache partition", "раздел cache"], &[Wipe::Cache]),
];

/// What a factory reset erases, as the wipe page and an update that asks
/// for one tell it; in English as it goes on from what leads to it, its
/// first letter small.
fn factory_reset(s: &State) -> String {
    let also = s.lang.pick(match s.cache_partition {
        true => ["the metadata and cache partitions", "разделы metadata и cache"],
        false => ["the metadata partition", "раздел metadata"],
    });
    tr!(
        s.lang,
        "empties the data partition, apps, accounts, settings and internal storage included, and formats {also}",
        "опустошает раздел data, включая приложения, аккаунты, настройки и внутреннюю память, и форматирует {also}",
    )
}

/// A choice of the wipe page as it lists it: its title, and exactly what it
/// erases.
fn wipe_row(w: Wipe, s: &State) -> (&'static str, String) {
    let l = s.lang;
    let (title, what): (Text, String) = match w {
        Wipe::All => (["All data", "Все данные"], capitalized(l, &factory_reset(s))),
        Wipe::AppData => (
            ["App data", "Данные приложений"],
            l.pick([
                "/data/data, /data/user, /data/user_de, and Android/data and Android/media in internal storage. Apps go back to how they were first installed",
                "/data/data, /data/user, /data/user_de, а также Android/data и Android/media во внутренней памяти. Приложения вернутся к состоянию после первой установки",
            ])
            .into(),
        ),
        Wipe::Media => (
            ["Internal storage", "Внутренняя память"],
            l.pick(["/data/media: the photos, videos and files in /sdcard", "/data/media: фотографии, видео и файлы в /sdcard"]).into(),
        ),
        Wipe::AppCaches => (
            ["App caches", "Кэш приложений"],
            l.pick([
                "cache and code_cache in each app's data, and cache in Android/data. Apps make them again as they need them",
                "cache и code_cache в данных каждого приложения, а также cache в Android/data. Приложения создают их заново по мере необходимости",
            ])
            .into(),
        ),
        Wipe::SystemCaches => (
            ["System caches", "Системные кэши"],
            l.pick([
                "/data/system/package_cache, /data/resource-cache and /data/cache, made again at the next start",
                "/data/system/package_cache, /data/resource-cache и /data/cache, создаваемые заново при следующем запуске",
            ])
            .into(),
        ),
        Wipe::Art => (
            ["ART cache", "Кэш ART"],
            l.pick([
                "/data/dalvik-cache, /data/misc/apexdata/com.android.art/dalvik-cache and /data/ota, compiled again at the next start, which then takes longer",
                "/data/dalvik-cache, /data/misc/apexdata/com.android.art/dalvik-cache и /data/ota, скомпилированные заново при следующем запуске, который затем займёт больше времени",
            ])
            .into(),
        ),
        Wipe::Logs => {
            let own = match (l, s.phone_logs) {
                (_, 0) => String::new(),
                (Lang::En, 1) => ", and this phone's own log folder".into(),
                (_, n) => tr!(l, ", and this phone's own {n} log folders", ", и {n} собственные папки журналов этого телефона"),
            };
            let what = tr!(
                l,
                "System logs and crash records, such as /data/tombstones, /data/anr, /data/system/dropbox and /data/misc/logd, each user's bug reports{own}",
                "Системные журналы и отчёты о сбоях, такие как /data/tombstones, /data/anr, /data/system/dropbox и /data/misc/logd, отчёты об ошибках каждого пользователя{own}",
            );
            (["Logs", "Журналы"], what)
        }
        Wipe::Updates => (
            ["Downloaded updates", "Загруженные обновления"],
            l.pick(["System update packages in /data/ota_package", "Пакеты обновления системы в /data/ota_package"]).into(),
        ),
        Wipe::Cache => (
            ["Everything", "Всё"],
            l.pick([
                "Formats the cache partition, keeping the logs in its recovery folder",
                "Форматирует раздел cache, сохраняя журналы в его папке recovery",
            ]).into(),
        ),
    };
    (l.pick(title), what)
}

/// What to erase: everything, as a factory reset does, or parts of it on
/// their own, the cache partition among them where the phone has one; then
/// a swipe erases what is chosen.
pub fn wipe(cx: &Ctx) -> Frame {
    let (t, f, s, l) = (cx.t, cx.f, cx.state, cx.state.lang);
    let chosen = |w: Wipe| s.wipe[WIPES.iter().position(|&x| x == w).expect("every choice is one of WIPES")];
    let mut c = long_page(cx);
    let mut out = Vec::new();
    let top = t.m.content_top;
    let y = widgets::section_first(&mut c, t, f, top, l.pick(["Factory reset", "Сброс к заводским настройкам"]));
    let (name, sub) = wipe_row(Wipe::All, s);
    let all = widgets::list(&mut c, t, f, y, &[Row::new(name).subtitle(&sub).trail(Trail::Check(chosen(Wipe::All)))]);
    out.extend(hits(&all.rows, &[Action::WipeItem(Wipe::All)]));
    let note = l.pick([
        "The system and installed updates stay. The phone has to be set up again afterwards.",
        "Система и установленные обновления остаются. После этого телефон придётся настроить заново.",
    ]);
    let mut y = widgets::footnote(&mut c, t, f, all.bottom, note);
    for (title, items) in WIPE_GROUPS {
        let shown: Vec<Wipe> = items.iter().copied().filter(|&w| w != Wipe::Cache || s.cache_partition).collect();
        if shown.is_empty() {
            continue;
        }
        let below = widgets::section(&mut c, t, f, y, l.pick(title));
        let texts: Vec<(&str, String)> = shown.iter().map(|&w| wipe_row(w, s)).collect();
        let rows: Vec<Row> =
            shown.iter().zip(&texts).map(|(&w, (name, sub))| Row::new(name).subtitle(sub).trail(Trail::Check(chosen(w)))).collect();
        let list = widgets::list(&mut c, t, f, below, &rows);
        out.extend(hits(&list.rows, &shown.iter().map(|&w| Action::WipeItem(w)).collect::<Vec<_>>()));
        y = list.bottom;
    }
    let note = l.pick([
        "Wiping app data or app caches needs the data partition decrypted first. What is wiped can't be brought back.",
        "Для очистки данных приложений или их кэша сначала нужно расшифровать раздел data. То, что очищено, нельзя вернуть.",
    ]);
    let y = widgets::footnote(&mut c, t, f, y, note);
    let (label, danger) = confirm_label(Confirm::Wipe, cx.state);
    let track = widgets::confirm(&mut c, t, f, y + t.m.dp(24.0), label, danger, s.confirm);
    out.push(Hit { rect: track, action: Action::Confirm(Confirm::Wipe) });
    Frame::new(t, fit(c, t, track.y + track.h), out, l.pick(["Wipe data", "Очистить данные"]), top)
}

/// A page laid out above the on-screen keyboard: where the keyboard's top
/// is, and its keys as touch targets.
fn keyboard_hits(c: &mut Canvas, t: &Theme, f: &Fonts, s: &State, enter: &str, out: &mut Vec<Hit>) -> f32 {
    let top = t.m.h as f32 - widgets::keyboard_height(t);
    let keys = widgets::keyboard(c, t, f, s.lang, top, s.layer, s.key_held(), enter);
    out.extend(keys.into_iter().map(|(rect, cap)| Hit { rect, action: Action::Type(cap) }));
    top
}

/// Finding a feature or partition: the field where the title would be, the
/// button that leaves the search beside it, what it finds, and the
/// keyboard.
pub fn search(cx: &Ctx) -> Frame {
    let (t, f, s, l) = (cx.t, cx.f, cx.state, cx.state.lang);
    let m = &t.m;
    let mut c = page(cx);
    let mut out = Vec::new();
    let (at, cancel) = widgets::search_layout(t);
    let field = widgets::search_field(&mut c, t, f, at.x, at.w, at.y + at.h / 2.0, &s.search, l.pick(SEARCH_HINT));
    widgets::cancel_button(&mut c, t, cancel);
    out.push(Hit { rect: cancel, action: Action::CloseSearch });
    let top = field.field.y + field.field.h + m.dp(12.0);
    let kb_top = m.h as f32 - widgets::keyboard_height(t);
    if s.search.trim().is_empty() {
        let how = l.pick([
            "Type the name of a feature or partition, such as backup to find Backup.",
            "Введите название функции или раздела, например backup, чтобы найти Backup.",
        ]);
        widgets::footnote(&mut c, t, f, top, how);
    } else {
        let found = search::find_on(&s.search, l, |screen| s.offers(screen));
        if found.is_empty() {
            let typed = s.search.trim();
            let none = tr!(l, "Nothing found for “{typed}”", "По запросу «{typed}» ничего не найдено");
            widgets::list(&mut c, t, f, top, &[Row::new(&none).trail(Trail::None)]);
        } else {
            let rows: Vec<Row> = found.iter().map(|e| Row::new(l.pick(e.title)).subtitle(l.pick(e.place))).collect();
            let list = widgets::list(&mut c, t, f, top, &rows);
            // The keyboard covers what does not fit above it.
            let shown = list.rows.iter().zip(&found).filter(|(r, _)| r.y + r.h <= kb_top);
            out.extend(shown.map(|(&rect, e)| Hit { rect, action: Action::Open(e.screen) }));
        }
    }
    let find = l.pick(["Search", "Поиск"]);
    keyboard_hits(&mut c, t, f, s, find, &mut out);
    if let Some(rect) = field.clear {
        out.push(Hit { rect, action: Action::ClearSearch });
    }
    Frame { back: false, header: false, caret: Some(field.caret), ..Frame::new(t, c, out, find, 0.0) }
}

/// Backing up: what to take, where it goes, and how.
pub fn backup(cx: &Ctx) -> Frame {
    let (t, f, s, i, l) = (cx.t, cx.f, cx.state, cx.info, cx.state.lang);
    let mut c = long_page(cx);
    let mut out = Vec::new();
    let top = t.m.content_top;
    let y = widgets::section_first(&mut c, t, f, top, l.pick(["Contents", "Содержимое"]));
    let system = fmt::snapshot_backup_note(l, &i.snapshot).unwrap_or(l.pick(["The current slot's partitions in super", "Разделы текущего слота в super"]));
    let data = l.pick(match s.data == Data::Open {
        true => [
            "Apps, accounts and settings, without internal storage, the screen lock or fingerprints",
            "Приложения, аккаунты и настройки, без внутренней памяти, блокировки экрана и отпечатков",
        ],
        false => [
            "Apps, accounts and settings, without internal storage, the screen lock or fingerprints. Decrypt first",
            "Приложения, аккаунты и настройки, без внутренней памяти, блокировки экрана и отпечатков. Сначала расшифруйте",
        ],
    });
    let boot = l.pick(["boot, init_boot, vendor_boot, dtbo, vbmeta and recovery", "boot, init_boot, vendor_boot, dtbo, vbmeta и recovery"]);
    let modem = l.pick(["IMEI, modem settings and sensor calibration", "IMEI, настройки модема и калибровка датчиков"]);
    let subs = [boot, system, data, modem];
    let rows: Vec<Row> = BACKUP_PARTS
        .iter()
        .zip(subs)
        .enumerate()
        .map(|(k, (name, sub))| Row::new(l.pick(*name)).subtitle(sub).trail(Trail::Check(s.backup[k])))
        .collect();
    let parts = widgets::list(&mut c, t, f, y, &rows);
    out.extend(hits(&parts.rows, &[0, 1, 2, 3].map(Action::BackupPart)));
    let y = widgets::section(&mut c, t, f, parts.bottom, l.pick(["Save to", "Сохранить в"]));
    let storage = match (s.data.storage_open(), i.storage.free.map(fmt::gb)) {
        (true, Some(free)) => tr!(l, "{free} free", "Свободно {free}"),
        (true, None) => l.pick(["Free space unknown", "Свободное место неизвестно"]).to_string(),
        (false, _) if s.data == Data::Erased => l.pick(DATA_ERASED).to_string(),
        (false, _) => l.pick(DECRYPT_NEEDED).to_string(),
    };
    let usb = l.pick(match &s.usb_drive {
        Some(UsbDrive::Shown) => ["Connected", "Подключено"],
        Some(UsbDrive::Unreadable(_)) => USB_DRIVE_UNREADABLE,
        None => USB_DRIVE_WANTED,
    });
    let places = widgets::list(&mut c, t, f, y, &[
        Row::new(place_name(l, Place::Storage)).subtitle(&storage).trail(Trail::Check(s.backup_to == Place::Storage)),
        Row::new(place_name(l, Place::UsbDrive)).subtitle(usb).trail(Trail::Check(s.backup_to == Place::UsbDrive)),
    ]);
    out.extend(hits(&places.rows, &[Action::BackupTo(Place::Storage), Action::BackupTo(Place::UsbDrive)]));
    let y = widgets::section(&mut c, t, f, places.bottom, l.pick(["Options", "Параметры"]));
    let options = widgets::list(&mut c, t, f, y, &[
        Row::new(l.pick(["Compress", "Сжать"])).subtitle(l.pick(["Smaller, and slower to make", "Меньше, но создаётся медленнее"])).trail(Trail::Switch(s.compress)),
        Row::new(l.pick(["Verify when done", "Проверять по завершении"])).subtitle(l.pick(["Checks the backup's files are whole", "Проверяет целостность файлов резервной копии"])).trail(Trail::Switch(s.verify)),
    ]);
    out.extend(hits(&options.rows, &[Action::Toggle(Switch::Compress), Action::Toggle(Switch::Verify)]));
    let note = l.pick([
        "Backups go in the Skyward/backups folder, named by when they began.",
        "Резервные копии сохраняются в папку Skyward/backups и называются по времени начала.",
    ]);
    let y = widgets::footnote(&mut c, t, f, options.bottom, note);
    let (label, danger) = confirm_label(Confirm::Backup, cx.state);
    let track = widgets::confirm(&mut c, t, f, y + t.m.dp(24.0), label, danger, s.confirm);
    out.push(Hit { rect: track, action: Action::Confirm(Confirm::Backup) });
    Frame::new(t, fit(c, t, track.y + track.h), out, l.pick(["Backup", "Резервная копия"]), top)
}

/// The backups found, newest first, to restore one; those that cannot be
/// read are listed after them, to be deleted.
pub fn restore(cx: &Ctx) -> Frame {
    let (t, f, s, l) = (cx.t, cx.f, cx.state, cx.state.lang);
    let mut c = long_page(cx);
    let mut out = Vec::new();
    let top = t.m.content_top;
    let y = widgets::section_first(&mut c, t, f, top, l.pick(["Backups", "Резервные копии"]));
    let bottom = if s.backups.is_empty() {
        widgets::list(&mut c, t, f, y, &[Row::new(l.pick(["No backups", "Нет резервных копий"])).trail(Trail::None)]).bottom
    } else {
        let texts: Vec<(String, String)> = s
            .backups
            .iter()
            .map(|b| match &b.held {
                Ok(h) => (fmt::datetime(l, h.made, s.time_zone), format!("{} · {} · {}", fmt::parts(l, &h.parts), fmt::bytes(h.size), place_name(l, b.place))),
                Err(_) => (b.name.clone(), format!("{} · {}", l.pick(UNREADABLE), place_name(l, b.place))),
            })
            .collect();
        let rows: Vec<Row> = texts.iter().map(|(title, sub)| Row::new(title).subtitle(sub)).collect();
        let list = widgets::list(&mut c, t, f, y, &rows);
        out.extend(list.rows.iter().enumerate().map(|(k, &rect)| Hit { rect, action: Action::OpenBackup(k) }));
        list.bottom
    };
    let note = l.pick([
        "Backups are looked for in the Skyward/backups folders of internal storage and the USB drive. Internal storage needs the data partition decrypted first.",
        "Резервные копии ищутся в папках Skyward/backups внутренней памяти и USB-накопителя. Для внутренней памяти сначала нужно расшифровать раздел данных.",
    ]);
    let bottom = widgets::footnote(&mut c, t, f, bottom, note);
    Frame::new(t, fit(c, t, bottom), out, l.pick(["Restore", "Восстановить"]), top)
}

/// One backup: when and where it was made, and which of its parts to
/// restore. One made on another phone is not restored here, and one that
/// cannot be read can only be deleted.
pub fn restore_backup(cx: &Ctx) -> Frame {
    let (t, f, s, i, l) = (cx.t, cx.f, cx.state, cx.info, cx.state.lang);
    let (entry, chosen) = s.restore.as_ref().expect("the restore page shows a backup");
    let mut c = long_page(cx);
    let mut out = Vec::new();
    let top = t.m.content_top;
    let location = l.pick(["Location", "Местоположение"]);
    let y = match &entry.held {
        Err(why) => {
            let facts = [(l.pick(["Folder", "Папка"]), entry.name.clone()), (location, place_name(l, entry.place).to_string())];
            let y = widgets::info_card(&mut c, t, f, top, &facts);
            let note = tr!(
                l,
                "This backup can't be read. It may not have finished, or it may be damaged. {why}",
                "Эту резервную копию нельзя прочитать. Возможно, она не завершена или повреждена. {why}",
            );
            widgets::footnote(&mut c, t, f, y, &note)
        }
        Ok(held) => {
            let mut facts = vec![
                (l.pick(["Made", "Создано"]), fmt::datetime(l, held.made, s.time_zone)),
                (l.pick(["Size", "Размер"]), fmt::bytes(held.size)),
                (location, place_name(l, entry.place).to_string()),
            ];
            if let Some(system) = &held.system {
                facts.push((l.pick(["System version", "Версия системы"]), system.clone()));
            }
            if let Some(slot) = held.slot {
                facts.push((l.pick(["Slot", "Слот"]), slot.letter().to_string()));
            }
            let y = widgets::info_card(&mut c, t, f, top, &facts);
            if held.device != i.codename {
                let device = &held.device;
                let note = tr!(
                    l,
                    "This backup is from another device ({device}) and can't be restored to this phone.",
                    "Эта резервная копия с другого устройства ({device}), и её нельзя восстановить на этот телефон.",
                );
                widgets::footnote(&mut c, t, f, y, &note)
            } else {
                let y = widgets::section(&mut c, t, f, y, l.pick(["Contents", "Содержимое"]));
                let kept: Vec<usize> = (0..BACKUP_PARTS.len()).filter(|&k| held.parts[k]).collect();
                let rows: Vec<Row> = kept.iter().map(|&k| Row::new(l.pick(BACKUP_PARTS[k])).trail(Trail::Check(chosen[k]))).collect();
                let list = widgets::list(&mut c, t, f, y, &rows);
                out.extend(list.rows.iter().zip(&kept).map(|(&rect, &k)| Hit { rect, action: Action::RestorePart(k) }));
                let note = l.pick([
                    "Restoring writes the backup over what the parts chosen hold now.",
                    "Восстановление записывает резервную копию поверх того, что сейчас хранят выбранные разделы.",
                ]);
                let mut y = widgets::footnote(&mut c, t, f, list.bottom, note);
                if chosen[2] {
                    let note = l.pick([
                        "The data partition gets its apps, accounts and settings back, and internal storage, the screen lock and fingerprints stay as they are. After a factory reset, an earlier backup of the data partition can't be restored.",
                        "Раздел data получает обратно приложения, аккаунты и настройки, а внутренняя память, блокировка экрана и отпечатки остаются как есть. После сброса к заводским настройкам более раннюю копию раздела data восстановить нельзя.",
                    ]);
                    y = widgets::footnote(&mut c, t, f, y, note);
                }
                // The boot partitions' vbmeta checks the system's: one of
                // another version without the other may not start.
                if let (Some(was), Some(now)) = (&held.system, &i.os_version) {
                    if was != now && chosen[0] != chosen[1] {
                        let warn = tr!(
                            l,
                            "This backup is from {was}, and this phone runs {now}. Restoring only one of the boot and system partitions may keep the phone from starting.",
                            "Эта резервная копия сделана на {was}, а на этом телефоне установлена {now}. Восстановление только одного из разделов boot и system может помешать телефону запуститься.",
                        );
                        y = widgets::footnote(&mut c, t, f, y, &warn);
                    }
                }
                let (label, danger) = confirm_label(Confirm::Restore, cx.state);
                let track = widgets::confirm(&mut c, t, f, y + t.m.dp(24.0), label, danger, s.confirm);
                out.push(Hit { rect: track, action: Action::Confirm(Confirm::Restore) });
                track.y + track.h
            }
        }
    };
    let delete = widgets::list(&mut c, t, f, y + t.m.dp(28.0), &[Row::new(l.pick(["Delete backup", "Удалить резервную копию"])).tint(t.pal.danger).trail(Trail::None)]);
    out.extend(hits(&delete.rows, &[Action::AskDelete]));
    let mut bottom = delete.bottom;
    if s.asking_delete {
        let ask = l.pick(["Delete this backup? It can't be brought back.", "Удалить эту резервную копию? Вернуть её будет нельзя."]);
        let y = widgets::footnote(&mut c, t, f, delete.bottom, ask);
        let (label, danger) = confirm_label(Confirm::DeleteBackup, cx.state);
        let track = widgets::confirm(&mut c, t, f, y + t.m.dp(24.0), label, danger, s.confirm);
        out.push(Hit { rect: track, action: Action::Confirm(Confirm::DeleteBackup) });
        bottom = track.y + track.h;
    }
    Frame::new(t, fit(c, t, bottom), out, l.pick(["Restore backup", "Восстановить резервную копию"]), top)
}

/// The share of its pass the part under way has done.
fn part_share(a: &BackupAt) -> f32 {
    if a.of == 0 {
        1.0
    } else {
        share(a.done, a.of).min(1.0)
    }
}

/// How far backing up or restoring has come over all, 0 to 1, and the
/// bytes of the parts the pass under way has gone through, the part under
/// way by its share. Each part counts by its bytes in each pass: a restore
/// checks all its parts first, and a backup checks them last.
fn job_progress(job: &BackupJob, a: &BackupAt) -> (f32, u64) {
    let total: u64 = job.plan.iter().map(|(_, n)| n).sum();
    let before: u64 = job.plan[..a.item].iter().map(|(_, n)| n).sum();
    let passed = before as f64 + f64::from(part_share(a)) * job.plan[a.item].1 as f64;
    let second = job.checks && a.checking != job.restore;
    let passes = if job.checks { 2.0 } else { 1.0 };
    let over = (if second { total as f64 } else { 0.0 } + passed) / (passes * total.max(1) as f64);
    (over as f32, passed as u64)
}

/// What a backup, or a restore, has done to a part, and does to it, in
/// each language.
fn job_words(restore: bool) -> (Text, Text) {
    match restore {
        true => (["Restored", "Восстановлено"], ["Restoring", "Восстановление"]),
        false => (["Backed up", "Скопировано"], ["Backing up", "Копирование"]),
    }
}

const VERIFIED: Text = ["Verified", "Проверено"];
const VERIFYING: Text = ["Verifying", "Проверка"];
const WAITING_TURN: Text = ["Waiting", "Ожидание"];
const DONE: Text = ["Done", "Готово"];

/// Where part `k` of a backup or restore stands, as its row says it.
fn part_state(l: Lang, job: &BackupJob, k: usize) -> String {
    use std::cmp::Ordering;
    let (did, doing) = job_words(job.restore);
    let ended = job.outcome.is_some();
    let a = match (&job.outcome, job.at) {
        (Some(BackupEnd::Saved { .. }), _) => return l.pick(if job.checks { VERIFIED } else { did }).to_string(),
        (Some(BackupEnd::Restored { .. }), _) => return l.pick(did).to_string(),
        (_, None) => return l.pick(if ended { ["Not started", "Не начато"] } else { WAITING_TURN }).to_string(),
        (_, Some(a)) => a,
    };
    let state = match k.cmp(&a.item) {
        Ordering::Less if a.checking => VERIFIED,
        Ordering::Less => did,
        Ordering::Equal if ended => ["Unfinished", "Незавершено"],
        Ordering::Equal => return format!("{} {}%", l.pick(if a.checking { VERIFYING } else { doing }), percent(part_share(&a))),
        // Ahead of the pass under way: a backup checks its parts once all
        // are written, and a restore writes them once all are checked.
        Ordering::Greater => match (a.checking, job.restore, ended) {
            (true, false, false) => did,
            (true, false, true) => ["Not verified", "Не проверено"],
            (false, false, false) | (true, true, false) => WAITING_TURN,
            (false, false, true) => ["Not backed up", "Копия не создана"],
            (false, true, false) => VERIFIED,
            (_, true, true) => ["Not restored", "Не восстановлено"],
        },
    };
    l.pick(state).to_string()
}

/// Backing up or restoring: how far it has come over all, each part and
/// where it stands, and a way to stop while stopping breaks nothing. Once
/// it has ended, how, and where to go.
pub fn backup_job(cx: &Ctx) -> Frame {
    let (t, f, s, l) = (cx.t, cx.f, cx.state, cx.state.lang);
    let job = s.backup_job.as_ref().expect("the backup page shows work");
    let mut c = long_page(cx);
    let mut out = Vec::new();
    let top = t.m.content_top;
    let (did, doing) = job_words(job.restore);
    let icon = if job.restore { Icon::Restore } else { Icon::Backup };
    let writing = job.at.is_some_and(|a| !a.checking);
    let mut y = match &job.outcome {
        None => {
            let (title, detail, done) = match job.at {
                Some(a) => {
                    let (over, passed) = job_progress(job, &a);
                    let total = fmt::bytes(job.plan.iter().map(|(_, n)| n).sum());
                    let (verb, past) = if a.checking { (l.pick(VERIFYING), l.pick(VERIFIED)) } else { (l.pick(doing), l.pick(did)) };
                    let (name, passed) = (&job.plan[a.item].0, fmt::bytes(passed));
                    (format!("{verb} {name}"), tr!(l, "{past} {passed} of {total}", "{past} {passed} из {total}"), over)
                }
                None if job.restore => (
                    l.pick(["Reading the backup", "Чтение резервной копии"]).to_string(),
                    l.pick(["Checking the backup against this phone", "Сверка резервной копии с этим телефоном"]).to_string(),
                    0.0,
                ),
                None => (l.pick(GETTING_READY).to_string(), l.pick(["Working out the backup's size", "Определение размера резервной копии"]).to_string(), 0.0),
            };
            widgets::progress_card(&mut c, t, f, top, &title, &detail, Some(done))
        }
        Some(end) => {
            let gone = l.pick(if job.restore { ["Nothing was written.", "Ничего не было записано."] } else { ["No backup was saved.", "Резервная копия не сохранена."] });
            let failed = l.pick(if job.restore { RESTORE_FAILED } else { ["Backup failed", "Не удалось создать резервную копию"] });
            let (title, color, detail) = match end {
                BackupEnd::Saved { name, skipped } => {
                    let saved = match job.place {
                        Place::Storage => tr!(
                            l,
                            "Saved to Skyward/backups/{name} in internal storage",
                            "Сохранено в Skyward/backups/{name} во внутренней памяти",
                        ),
                        Place::UsbDrive => tr!(l, "Saved to Skyward/backups/{name} on the USB drive", "Сохранено в Skyward/backups/{name} на USB-накопителе"),
                    };
                    (l.pick(BACKUP_COMPLETE), GREEN, match *skipped {
                        0 => saved,
                        n => tr!(
                            l,
                            "{saved} ({n} files couldn't be read and were left out)",
                            "{saved} (не удалось прочитать {n} файлов, они пропущены)"
                        ),
                    })
                }
                BackupEnd::Restored { .. } => {
                    (l.pick(RESTORE_COMPLETE), GREEN, l.pick(["The phone uses what was restored once it reboots.", "Телефон начнёт использовать восстановленное после перезагрузки."]).to_string())
                }
                BackupEnd::Stopped => (l.pick(["Stopped", "Остановлено"]), SLATE, gone.to_string()),
                BackupEnd::Refused(r) => {
                    let title = l.pick(if job.restore { ["Can't restore", "Не удаётся восстановить"] } else { ["Can't back up", "Не удаётся создать копию"] });
                    (title, ORANGE, fmt::backup_refusal(l, r, job.restore))
                }
                BackupEnd::Failed(_) if job.restore && writing => {
                    let detail = l.pick([
                        "Part of it was written, so the phone may not start properly after a reboot. Restore again, and reboot once it's done.",
                        "Часть данных была записана, поэтому после перезагрузки телефон может не запуститься правильно. Восстановите снова и перезагрузитесь после завершения.",
                    ]);
                    (failed, RED, detail.to_string())
                }
                BackupEnd::Failed(_) => (failed, RED, gone.to_string()),
            };
            widgets::list(&mut c, t, f, top, &[standing(title, color, icon, &detail)]).bottom
        }
    };
    let keep = l.pick(match job.place {
        Place::Storage => ["Don't power off", "Не выключайте"],
        Place::UsbDrive => ["Keep the USB drive in and don't power off", "Не извлекайте USB-накопитель и не выключайте питание"],
    });
    match &job.outcome {
        None => {
            let note = match (job.restore, job.at) {
                (false, _) => tr!(
                    l,
                    "{keep} until the backup is done. Stopping deletes the unfinished backup.",
                    "{keep} до завершения резервной копии. Остановка удалит незавершённую копию.",
                ),
                (true, Some(a)) if !a.checking => tr!(l, "{keep} until the restore is done.", "{keep} до завершения восстановления."),
                (true, _) => l.pick(["Nothing is written until the backup is verified.", "Ничего не записывается, пока копия не проверена."]).to_string(),
            };
            y = widgets::footnote(&mut c, t, f, y, &note);
            // A restore that has begun writing goes on to its end.
            if !(job.restore && writing) {
                let label = l.pick(if job.stopping { ["Stopping", "Остановка"] } else { STOP });
                let stop = widgets::list(&mut c, t, f, y + t.m.card_gap, &[Row::new(label).trail(Trail::None)]);
                if !job.stopping {
                    out.extend(hits(&stop.rows, &[Action::StopBackup]));
                }
                y = stop.bottom;
            }
        }
        Some(end) => {
            match end {
                BackupEnd::Restored { unmade } if !unmade.is_empty() => {
                    let folders = unmade.join(l.pick([", ", ", "]));
                    let note = tr!(
                        l,
                        "The users these folders belonged to no longer exist, so what they hold wasn't restored: {folders}",
                        "Пользователи, которым принадлежали эти папки, больше не существуют, поэтому их содержимое не было восстановлено: {folders}",
                    );
                    y = widgets::footnote(&mut c, t, f, y, &note);
                }
                BackupEnd::Failed(why) => y = widgets::footnote(&mut c, t, f, y, why),
                _ => {}
            }
            let (rows, acts): (Vec<Row>, Vec<Action>) = match end {
                BackupEnd::Restored { .. } => (
                    vec![Row::new(l.pick(["Reboot to system", "Перезагрузить в систему"])).icon(ORANGE, Icon::Power), Row::new(l.pick(DONE)).trail(Trail::None)],
                    vec![Action::RebootTo(Target::System), Action::Back],
                ),
                _ => (vec![Row::new(l.pick(DONE)).trail(Trail::None)], vec![Action::Back]),
            };
            let next = widgets::list(&mut c, t, f, y + t.m.card_gap, &rows);
            out.extend(hits(&next.rows, &acts));
            y = next.bottom;
        }
    }
    // The parts come after what can be done, which a long list would push
    // off the screen.
    if !job.plan.is_empty() {
        let heading = l.pick(["Contents", "Содержимое"]);
        let y0 = widgets::section(&mut c, t, f, y, heading);
        let states: Vec<String> = (0..job.plan.len()).map(|k| part_state(l, job, k)).collect();
        let rows: Vec<Row> = job.plan.iter().zip(&states).map(|((name, _), state)| Row::new(name).trail(Trail::Text(state))).collect();
        y = widgets::list(&mut c, t, f, y0, &rows).bottom;
    }
    let title = l.pick(match (&job.outcome, job.restore) {
        (None, restore) => job_words(restore).1,
        (Some(BackupEnd::Saved { .. }), _) => BACKUP_COMPLETE,
        (Some(BackupEnd::Restored { .. }), _) => RESTORE_COMPLETE,
        (Some(_), false) => ["Backup incomplete", "Резервная копия создана не полностью"],
        (Some(_), true) => ["Restore incomplete", "Восстановление не завершено"],
    });
    Frame { back: job.outcome.is_some(), ..Frame::new(t, fit(c, t, y), out, title, top) }
}

const BACKUP_COMPLETE: Text = ["Backup complete", "Резервная копия создана"];
const RESTORE_COMPLETE: Text = ["Restore complete", "Восстановление завершено"];
const RESTORE_FAILED: Text = ["Restore failed", "Не удалось восстановить"];
const STOP: Text = ["Stop", "Остановить"];

/// Tools for the experienced.
pub fn advanced(cx: &Ctx) -> Frame {
    let (t, f, s, l) = (cx.t, cx.f, cx.state, cx.state.lang);
    let mut c = long_page(cx);
    let top = t.m.content_top;
    let wireless = if s.wireless { l.pick(["On", "Вкл."]) } else { l.pick(["Off", "Выкл."]) };
    let tools = widgets::list(&mut c, t, f, top, &[
        Row::new(l.pick(FILES)).icon(BLUE, Icon::Folder),
        Row::new(l.pick(SORT)).icon(SLATE, Icon::Gear),
        Row::new(l.pick(TERMINAL)).icon(SLATE, Icon::Terminal),
        Row::new(l.pick(RELABEL)).icon(PURPLE, Icon::Shield),
        Row::new(l.pick(FILESYSTEMS)).icon(RED, Icon::Mount),
        Row::new(l.pick(LOGS)).icon(ORANGE, Icon::Log),
        Row::new(l.pick(UPDATE)).icon(GREEN, Icon::Layers).trail(Trail::Value(update_value(cx))),
        Row::new(l.pick(WIRELESS)).icon(GREEN, Icon::Wlan).trail(Trail::Value(wireless)),
        Row::new(l.pick(FETCH)).icon(BLUE, Icon::Globe),
    ]);
    let out = hits(&tools.rows, &[
        Action::Open(Screen::Files),
        Action::Open(Screen::Sort),
        Action::Open(Screen::Terminal),
        Action::Open(Screen::Relabel),
        Action::Open(Screen::Filesystems),
        Action::Open(Screen::Logs),
        Action::Open(Screen::Update),
        Action::Open(Screen::Wireless),
        Action::Open(Screen::Fetch),
    ]);
    Frame::new(t, fit(c, t, tools.bottom), out, l.pick(ADVANCED), top)
}

const RELABEL: Text = ["Fix SELinux contexts", "Исправить контексты SELinux"];
const FILESYSTEMS: Text = ["Filesystems", "Файловые системы"];
const STOPPED: Text = ["Stopped", "Остановлено"];
const STOPPING: Text = ["Stopping", "Остановка"];

/// Giving the data partition's files the SELinux labels the installed
/// system's rules give them: what it does, and a swipe to start it; then
/// how far it has come, a way to stop it, and how it ended.
pub fn relabel(cx: &Ctx) -> Frame {
    let (t, f, s, l) = (cx.t, cx.f, cx.state, cx.state.lang);
    let mut c = page(cx);
    let mut out = Vec::new();
    let top = t.m.content_top;
    let title = l.pick(RELABEL);
    let Some(job) = &s.relabel else {
        let what = l.pick([
            "Gives the files in the data partition the SELinux contexts the installed system's rules give them. App data stays as it is.",
            "Присваивает файлам в разделе data контексты SELinux, которые задают правила установленной системы. Данные приложений остаются как есть.",
        ]);
        let mut y = widgets::paragraph(&mut c, t, f, top, what);
        let note = match s.data {
            Data::Open | Data::Locked { .. } => None,
            Data::Erased => Some(["The data partition was wiped.", "Раздел data очищен."]),
            Data::Closed | Data::Opening | Data::Failed(_) => {
                Some(["Open the data partition in Decrypt first.", "Сначала откройте раздел data в «Расшифровке»."])
            }
        };
        if let Some(note) = note {
            y = widgets::footnote(&mut c, t, f, y, l.pick(note));
        }
        let (label, danger) = confirm_label(Confirm::Relabel, cx.state);
        let track = widgets::confirm(&mut c, t, f, y + t.m.dp(24.0), label, danger, s.confirm);
        out.push(Hit { rect: track, action: Action::Confirm(Confirm::Relabel) });
        return Frame::new(t, c, out, title, top);
    };
    let repair = Verb::of(&crate::app::PartKind::Repair);
    let heading = match &job.outcome {
        None => repair.going(l),
        Some(JobEnd::Done) => repair.complete(l),
        Some(JobEnd::Stopped) => l.pick(STOPPED).into(),
        Some(JobEnd::Failed(_)) => repair.failed(l),
    };
    let detail = match (job.total, job.checked) {
        (0, _) if job.outcome.is_none() => l.pick(GETTING_READY).to_string(),
        (total, checked) => tr!(l, "{checked} of {total} checked", "Проверено {checked} из {total}"),
    };
    let y = widgets::progress_card(&mut c, t, f, top, &heading, &detail, Some(share(job.checked, job.total)));
    match &job.outcome {
        None => {
            let label = l.pick(if job.stopping { STOPPING } else { STOP });
            let stop = widgets::list(&mut c, t, f, y + t.m.card_gap, &[Row::new(label).trail(Trail::None)]);
            if !job.stopping {
                out.extend(hits(&stop.rows, &[Action::StopRelabel]));
            }
        }
        Some(end) => {
            let changed = job.changed;
            let note = match end {
                JobEnd::Done if changed == 0 => l
                    .pick([
                        "Every file's SELinux context is right, so nothing needed fixing.",
                        "Все контексты SELinux файлов в порядке, поэтому исправлять ничего не понадобилось.",
                    ])
                    .to_string(),
                JobEnd::Done => tr!(
                    l,
                    "{changed} entries got the SELinux context the rules give them.",
                    "{changed} записей получили контекст SELinux, заданный правилами.",
                ),
                JobEnd::Stopped => tr!(l, "Stopped, with {changed} entries set again.", "Остановлено, {changed} записей задано заново."),
                JobEnd::Failed(why) => tr!(l, "Repair failed: {why}", "Не удалось исправить: {why}"),
            };
            let y = widgets::footnote(&mut c, t, f, y, &note);
            let finish = widgets::list(&mut c, t, f, y + t.m.card_gap, &[Row::new(l.pick(DONE)).trail(Trail::None)]);
            out.extend(hits(&finish.rows, &[Action::Back]));
        }
    }
    Frame { back: job.outcome.is_some(), ..Frame::new(t, c, out, title, top) }
}

/// The partitions whose filesystems can be repaired or made afresh as
/// another, each with what it holds and its size.
pub fn filesystems(cx: &Ctx) -> Frame {
    let (t, f, s, l) = (cx.t, cx.f, cx.state, cx.state.lang);
    let mut c = long_page(cx);
    let mut out = Vec::new();
    let top = t.m.content_top;
    let y = widgets::section_first(&mut c, t, f, top, l.pick(["Partitions", "Разделы"]));
    let bottom = match &s.filesystems {
        None => widgets::list(&mut c, t, f, y, &[Row::new(l.pick(["Reading the partitions", "Чтение разделов"])).trail(Trail::None)]).bottom,
        Some(Err(why)) => {
            let y = widgets::list(&mut c, t, f, y, &[Row::new(l.pick(["Couldn't read the partitions", "Не удалось прочитать разделы"])).trail(Trail::None)]).bottom;
            widgets::footnote(&mut c, t, f, y, why)
        }
        Some(Ok(rows)) if rows.is_empty() => {
            widgets::list(&mut c, t, f, y, &[Row::new(l.pick(["No partitions to repair", "Нет разделов для исправления"])).trail(Trail::None)]).bottom
        }
        Some(Ok(rows)) => {
            let subs: Vec<String> = rows.iter().map(|r| format!("{} · {}", fs_held(l, r, &s.data), fmt::bytes(r.size))).collect();
            let shown: Vec<Row> = rows.iter().zip(&subs).map(|(r, sub)| Row::new(&r.name).subtitle(sub)).collect();
            let list = widgets::list(&mut c, t, f, y, &shown);
            out.extend(hits(&list.rows, &(0..rows.len()).map(Action::OpenFilesystem).collect::<Vec<_>>()));
            let note = l.pick([
                "Only the partitions in this phone's partition table that can be formatted show here.",
                "Здесь показаны только разделы таблицы разделов этого телефона, которые можно отформатировать.",
            ]);
            widgets::footnote(&mut c, t, f, list.bottom, note)
        }
    };
    Frame::new(t, fit(c, t, bottom), out, l.pick(FILESYSTEMS), top)
}

/// What a partition holds, as its row and page name it: the data
/// partition's is read only once it is open, and until then the row says
/// where it stands, as the home page does.
fn fs_held(l: Lang, row: &FsRow, data: &Data) -> String {
    match (&row.filesystem, row.name == DATA_PARTITION) {
        (Some(filesystem), _) => filesystem.clone(),
        (None, true) => fmt::data(l, data).into(),
        (None, false) => l.pick(["Unrecognized", "Не распознано"]).into(),
    }
}

/// One partition: what it holds, what can be done to it, and a swipe to do
/// what is chosen.
pub fn filesystem(cx: &Ctx) -> Frame {
    let (t, f, s, l) = (cx.t, cx.f, cx.state, cx.state.lang);
    let mut c = long_page(cx);
    let mut out = Vec::new();
    let top = t.m.content_top;
    let Some(Ok(rows)) = &s.filesystems else { unreachable!("a partition's page opens from the list read") };
    let row = &rows[s.fs_chosen];
    let data = row.name == DATA_PARTITION;
    let facts = [
        (l.pick(["Mount point", "Точка монтирования"]), row.mount_point.clone()),
        (l.pick(["Filesystem", "Файловая система"]), fs_held(l, row, &s.data)),
        (l.pick(["Size", "Размер"]), fmt::bytes(row.size)),
    ];
    let y = widgets::info_card(&mut c, t, f, top, &facts);
    let y = widgets::section(&mut c, t, f, y, l.pick(ACTIONS));
    let chosen = |action: &FsAction| s.fs_action == *action;
    let remakes: Vec<String> = row.others.iter().map(|fs| tr!(l, "Change to {fs}", "Изменить на {fs}")).collect();
    let made: Vec<String> = row.others.iter().map(|_| remade(row, s)).collect();
    let repair = Row::new(l.pick(["Repair the filesystem", "Исправить файловую систему"]))
        .subtitle(l.pick(["Checks the whole filesystem and fixes the errors found", "Проверяет всю файловую систему и исправляет найденные ошибки"]))
        .trail(Trail::Check(chosen(&FsAction::Repair)));
    let mut shown = vec![repair];
    let mut actions = vec![Action::ChooseFs(FsChoice::Repair)];
    for (k, fs) in row.others.iter().enumerate() {
        shown.push(Row::new(&remakes[k]).subtitle(&made[k]).trail(Trail::Check(chosen(&FsAction::Remake(fs.clone())))));
        actions.push(Action::ChooseFs(FsChoice::Remake(k)));
    }
    let list = widgets::list(&mut c, t, f, y, &shown);
    out.extend(hits(&list.rows, &actions));
    let mut y = list.bottom;
    if row.others.is_empty() {
        let note = l.pick(["This partition's filesystem can't be changed.", "Файловую систему этого раздела нельзя изменить."]);
        y = widgets::footnote(&mut c, t, f, y, note);
    }
    if data {
        let note = l.pick([
            "The data partition closes for the repair, and opens again from Decrypt once it's done.",
            "Раздел data закрывается на время исправления и снова открывается из «Расшифровка» после его завершения.",
        ]);
        y = widgets::footnote(&mut c, t, f, y, note);
    }
    if !row.others.is_empty() {
        y = widgets::footnote(&mut c, t, f, y, l.pick(["Once changed, what the partition held can't be brought back.", "После изменения содержимое раздела вернуть нельзя."]));
    }
    let (label, danger) = confirm_label(Confirm::Filesystem, s);
    let track = widgets::confirm(&mut c, t, f, y + t.m.dp(24.0), label, danger, s.confirm);
    out.push(Hit { rect: track, action: Action::Confirm(Confirm::Filesystem) });
    Frame::new(t, fit(c, t, track.y + track.h), out, partition_title(l, &row.name), top)
}

/// The title of the page of a partition the filesystem page lists.
fn partition_title(l: Lang, name: &str) -> &'static str {
    l.pick(match name {
        DATA_PARTITION => DATA_PART,
        "metadata" => ["metadata partition", "раздел metadata"],
        "cache" => ["cache partition", "раздел cache"],
        _ => ["Partition", "Раздел"],
    })
}

/// What making `row` afresh as another filesystem erases: the data
/// partition goes as a factory reset goes, with /metadata and the cache
/// partition beside it.
fn remade(row: &FsRow, s: &State) -> String {
    match row.name == DATA_PARTITION {
        true => {
            let parts = s.lang.pick(match s.cache_partition {
                true => ["the data, cache and metadata partitions", "разделы data, cache и metadata"],
                false => ["the data and metadata partitions", "разделы data и metadata"],
            });
            tr!(s.lang, "Formats {parts}, emptying all of this phone's data", "Форматирует {parts}, удаляя все данные этого телефона")
        }
        false => {
            let name = &row.name;
            tr!(s.lang, "Formats the {name} partition, emptying all it holds", "Форматирует раздел {name}, удаляя всё его содержимое")
        }
    }
}

/// Work on whole partitions: each partition and where it stands, what the
/// program said of the last, and once it has ended, how. What the system
/// asked for leaves on its own once it went through every partition, and
/// offers where to go otherwise; what the user chose goes back to its page.
pub fn part_work(cx: &Ctx) -> Frame {
    use crate::app::{ItemEnd, ItemState, PartKind};
    let (t, f, s, l) = (cx.t, cx.f, cx.state, cx.state.lang);
    let work = s.part_work.as_ref().expect("the page of partition work shows work");
    let mut c = long_page(cx);
    let (mut out, mut rows) = (Vec::new(), None);
    let top = t.m.content_top;
    let verb = Verb::of(&work.kind);
    // Work on one partition fails; work through several may get partway.
    let (title, several) = match &work.kind {
        PartKind::Disable => (["Wipe partitions", "Очистить разделы"], true),
        PartKind::Unshare => (["Unshare blocks", "Разделить общие блоки"], true),
        PartKind::Repair => (["Repair filesystem", "Восстановить файловую систему"], false),
        PartKind::Remake(_) => (["Change filesystem", "Изменить файловую систему"], false),
    };
    let failed = |state: &ItemState| matches!(state, ItemState::Ended(ItemEnd::Failed(_)));
    let short = matches!(work.outcome, Some(Err(_))) || work.items.iter().any(|(_, state)| failed(state));
    let heading = match (&work.outcome, short, several) {
        (None, _, _) => verb.going(l),
        (Some(_), false, _) => verb.complete(l),
        (Some(_), true, true) => verb.incomplete(l),
        (Some(_), true, false) => verb.failed(l),
    };
    let mut y = widgets::section_first(&mut c, t, f, top, &heading);
    if !work.items.is_empty() {
        let states: Vec<String> = work
            .items
            .iter()
            .map(|(_, state)| match state {
                ItemState::Waiting if work.outcome.is_some() => verb.not_done(l),
                ItemState::Waiting => l.pick(WAITING_TURN).into(),
                ItemState::Working => verb.going(l),
                ItemState::Ended(end) => fmt::item_end(l, end, verb),
            })
            .collect();
        let held: Vec<Option<&str>> = work
            .items
            .iter()
            .map(|(_, state)| match state {
                ItemState::Ended(ItemEnd::NotShared(fs)) if !fs.is_empty() => Some(fs.as_str()),
                _ => None,
            })
            .collect();
        let shown: Vec<Row> = work
            .items
            .iter()
            .zip(&states)
            .zip(&held)
            .map(|(((name, _), state), held)| {
                let row = Row::new(name).trail(Trail::Text(state));
                match held {
                    Some(fs) => row.subtitle(fs),
                    None => row,
                }
            })
            .collect();
        y = widgets::list(&mut c, t, f, y, &shown).bottom;
    }
    let leaving = match s.asked.shutdown_after {
        true => ["Powers off on its own once done.", "Выключится самостоятельно по завершении."],
        false => ["Reboots on its own once done.", "Перезагрузится самостоятельно по завершении."],
    };
    let mut notes: Vec<String> = Vec::new();
    match &work.outcome {
        None if work.asked => notes.push(l.pick(leaving).into()),
        None => notes.push(verb.hold_on(l)),
        Some(Err(stop)) => {
            let (what, detail) = fmt::part_stop(l, stop, &cx.info.codename);
            notes.push(what);
            notes.extend(detail);
        }
        Some(Ok(())) => {}
    }
    for (name, state) in &work.items {
        if let ItemState::Ended(ItemEnd::Failed(Some(why))) = state {
            let failed = verb.failed_on(l, name);
            notes.push(tr!(l, "{failed}: {why}", "{failed}: {why}"));
        }
    }
    for note in &notes {
        y = widgets::footnote(&mut c, t, f, y, note);
    }
    if let (true, Some(reason)) = (work.asked, &s.asked.reason) {
        y = widgets::info_card(&mut c, t, f, y + t.m.dp(12.0), &[(l.pick(REASON), reason.clone())]);
    }
    match (&work.outcome, work.asked) {
        (None, _) => {}
        (Some(_), true) if !short => {}
        // The system's work ran without the slot's partitions, which going
        // on in the recovery brings up first, as leaving fastboot does.
        (Some(_), true) => {
            let below = widgets::section(&mut c, t, f, y, l.pick(ACTIONS));
            let rows: Vec<Row> = AFTERWARDS.iter().map(|&(title, _)| Row::new(l.pick(title)).trail(Trail::None)).collect();
            let actions = AFTERWARDS.map(|(_, action)| if action == Action::Dismiss { Action::EnterRecovery } else { action });
            let choices = widgets::list(&mut c, t, f, below, &rows);
            out.extend(hits(&choices.rows, &actions));
            y = choices.bottom;
        }
        (Some(_), false) => {
            let finish = widgets::list(&mut c, t, f, y + t.m.card_gap, &[Row::new(l.pick(DONE)).trail(Trail::None)]);
            out.extend(hits(&finish.rows, &[Action::Back]));
            y = finish.bottom;
        }
    }
    if !work.output.is_empty() {
        let below = widgets::section(&mut c, t, f, y, l.pick(["Program output", "Вывод программы"]));
        rows = Some(Rows { edges: widgets::output_edges(&c, t, f, below, &work.output), action: None });
        y = below;
    }
    let back = work.outcome.is_some() && !work.asked;
    Frame { rows, back, ..Frame::new(t, fit(c, t, y), out, l.pick(title), top) }
}

const ACTIONS: Text = ["Actions", "Действия"];

/// The system on the phone now; the newest system the phone maker's
/// server knows, and downloading its full package to install it; and
/// where an update the system installed stands, merging it into the slot it
/// went to: how far a merge under way has come, and a swipe to merge one
/// that waits there for the system to start it.
pub fn update(cx: &Ctx) -> Frame {
    let (t, f, s, i, l) = (cx.t, cx.f, cx.state, cx.info, cx.state.lang);
    let mut c = long_page(cx);
    let top = t.m.content_top;
    let mut out = Vec::new();
    if let Standing::Merging(done) = s.update {
        let share = done as f32;
        let p = percent(share);
        let detail = tr!(l, "{p}% done", "{p}% готово");
        let y = widgets::progress_card(&mut c, t, f, top, l.pick(["Merging the update", "Объединение обновления"]), &detail, Some(share));
        let note = l.pick([
            "Partitions in super aren't flashed until it merges. Merging goes on after going back.",
            "Разделы в super не прошиваются, пока не завершится объединение. После возврата объединение продолжится.",
        ]);
        let y = widgets::footnote(&mut c, t, f, y, note);
        return Frame::new(t, fit(c, t, y), out, l.pick(UPDATE), top);
    }
    let mut y = top;
    if let Some(v) = &i.os_version {
        y = widgets::section_first(&mut c, t, f, y, l.pick(["This phone", "Этот телефон"]));
        y = widgets::info_card(&mut c, t, f, y, &[(l.pick(["Current version", "Текущая версия"]), v.clone())]);
    }
    if s.update_server {
        let online_update = l.pick(["Online update", "Обновление по сети"]);
        y = if y == top { widgets::section_first(&mut c, t, f, y, online_update) } else { widgets::section(&mut c, t, f, y, online_update) };
        y = online(&mut c, cx, y, &mut out);
    }
    let next = match &s.boot_slots {
        Some(Ok(slots)) => Some(slots.active),
        _ => None,
    };
    let (title, detail) = fmt::update(l, &s.update, next);
    let color = match s.update {
        Standing::None => SLATE,
        Standing::Merged => GREEN,
        Standing::Failed(_) | Standing::Unknown => RED,
        _ => ORANGE,
    };
    let mut row = Row::new(title).icon(color, Icon::Layers).trail(Trail::None);
    if let Some(detail) = &detail {
        row = row.subtitle(detail);
    }
    let merging = l.pick(["Merging", "Объединение"]);
    y = if y == top { widgets::section_first(&mut c, t, f, y, merging) } else { widgets::section(&mut c, t, f, y, merging) };
    y = widgets::list(&mut c, t, f, y, &[row]).bottom;
    if s.update == Standing::Waiting {
        let (label, danger) = confirm_label(Confirm::Merge, cx.state);
        let track = widgets::confirm(&mut c, t, f, y + t.m.dp(28.0), label, danger, s.confirm);
        out.push(Hit { rect: track, action: Action::Confirm(Confirm::Merge) });
        let note = l.pick([
            "Once merged, the system from before the update can't come back.",
            "После объединения система, бывшая до обновления, вернуться не сможет.",
        ]);
        y = widgets::footnote(&mut c, t, f, track.y + track.h, note);
    }
    Frame::new(t, fit(c, t, y), out, l.pick(UPDATE), top)
}

/// What downloading or checking for updates needs first, while the phone
/// has joined no network that reaches the internet.
const WLAN_FIRST: Text = ["Connect to Wi-Fi first", "Сначала подключитесь к Wi-Fi"];
const CHECKING: Text = ["Checking for updates", "Проверка обновлений"];
const CHECK_FOR_UPDATES: Text = ["Check for updates", "Проверить обновления"];
const NO_RELEASE_TO_LOOK_FOR: Text = ["A build made to try something looks for no newer one", "Сборка для проверки не ищет более новых версий"];
const DOWNLOADING: Text = ["Downloading", "Загрузка"];
const CHECKING_PART: Text = ["Verifying what was downloaded", "Проверка загруженного"];

/// Asking the update server for the newest system, then its full package:
/// downloading it, how far that has come, and installing it once it is
/// whole; then what to note before installing it.
fn online(c: &mut Canvas, cx: &Ctx, y: f32, out: &mut Vec<Hit>) -> f32 {
    let (t, f, s, i, l) = (cx.t, cx.f, cx.state, cx.info, cx.state.lang);
    let release = match &s.check {
        Check::Asking => return widgets::list(c, t, f, y, &[Row::new(l.pick(CHECKING)).trail(Trail::None)]).bottom,
        Check::Answered(Some(r)) => r,
        other => {
            let mut row = Row::new(l.pick(CHECK_FOR_UPDATES));
            if !s.online() {
                row = row.subtitle(l.pick(WLAN_FIRST));
            }
            let check = widgets::list(c, t, f, y, &[row]);
            out.extend(hits(&check.rows, &[Action::CheckUpdate]));
            return match other {
                Check::Failed(why) => widgets::footnote(c, t, f, check.bottom, &fmt::unanswered(l, why)),
                Check::Answered(None) => {
                    let none = l.pick(["The server knows of no system update for this phone.", "Сервер не знает об обновлении системы для этого телефона."]);
                    widgets::footnote(c, t, f, check.bottom, none)
                }
                _ => check.bottom,
            };
        }
    };
    let newer = i.os_version.as_deref() != Some(release.version.as_str());
    let version = &release.version;
    let title = if newer { tr!(l, "New version {version}", "Новая версия {version}") } else { l.pick(["Up to date", "Актуально"]).to_string() };
    let size = match release.length {
        Some(length) => fmt::file_size(length),
        None => fmt::package_size(&release.size),
    };
    let package = tr!(l, "Full OTA package {size}", "Полный пакет OTA {size}");
    let mut y = widgets::list(c, t, f, y, &[Row::new(&title).icon(GREEN, Icon::Layers).subtitle(&package).trail(Trail::None)]).bottom;
    match (&s.download, s.have) {
        (Some(job), _) if job.outcome.is_none() => {
            let (heading, detail, share) = match job.progress {
                None => (l.pick(GETTING_READY), String::new(), None),
                Some(p) => downloaded(l, p, job.rate),
            };
            y = widgets::progress_card(c, t, f, y + t.m.card_gap, heading, &detail, share);
            let label = l.pick(if job.stopping { ["Pausing", "Приостановка"] } else { ["Pause", "Пауза"] });
            let stop = widgets::list(c, t, f, y + t.m.card_gap, &[Row::new(label).trail(Trail::None)]);
            if !job.stopping {
                out.extend(hits(&stop.rows, &[Action::StopDownload]));
            }
            y = stop.bottom;
        }
        (_, Have::Whole(_)) => {
            let row = Row::new(l.pick(["Install", "Установить"])).subtitle(l.pick(["Downloaded to internal storage", "Загружено во внутреннюю память"]));
            let install = widgets::list(c, t, f, y + t.m.card_gap, &[row]);
            out.extend(hits(&install.rows, &[Action::InstallDownloaded]));
            y = install.bottom;
        }
        (job, have) => {
            match job.as_ref().and_then(|j| j.outcome.as_ref()) {
                Some(DownloadEnd::Failed(why)) => y = widgets::footnote(c, t, f, y, &fmt::download_failure(l, why)),
                Some(DownloadEnd::Stopped) => y = widgets::footnote(c, t, f, y, l.pick(["Download paused.", "Загрузка приостановлена."])),
                _ => {}
            }
            let title = l.pick(if matches!(have, Have::Part(_)) { ["Resume download", "Продолжить загрузку"] } else { ["Download", "Загрузка"] });
            let had = match have {
                Have::Part(done) => {
                    let done = fmt::file_size(done);
                    Some(tr!(l, "{done} downloaded", "Загружено {done}"))
                }
                _ => None,
            };
            let sub = if !s.data.storage_open() {
                Some(l.pick(DECRYPT_NEEDED))
            } else if !s.online() {
                Some(l.pick(WLAN_FIRST))
            } else {
                had.as_deref()
            };
            let row = Row::new(title);
            let download = widgets::list(c, t, f, y + t.m.card_gap, &[sub.map_or(row, |sub| row.subtitle(sub))]);
            out.extend(hits(&download.rows, &[Action::Download]));
            y = download.bottom;
        }
    }
    let saved = tr!(
        l,
        "The full OTA package is saved in “/sdcard/{DOWNLOADS}”, and stays there after it's installed.",
        "Полный пакет OTA сохраняется в «/sdcard/{DOWNLOADS}» и остаётся там после установки.",
    );
    let points = [
        l.pick([
            "Installing a system update deletes no user data. Backing up first is still wise.",
            "Установка обновления системы не удаляет пользовательские данные. Но создать копию заранее всё же разумно.",
        ]),
        saved.as_str(),
        l.pick([
            "A download cut short can be resumed, keeping what was downloaded.",
            "Прерванную загрузку можно продолжить, сохранив уже загруженное.",
        ]),
        l.pick([
            "After the update the system optimizes itself in the background, and the phone may run warm or use more battery for 1 to 3 days.",
            "После обновления система оптимизируется в фоне, и телефон может нагреваться или расходовать больше батареи в течение 1–3 дней.",
        ]),
    ];
    widgets::notes(c, t, f, y + t.m.card_gap, l.pick(NOTES), &points)
}

const NOTES: Text = ["Notes", "Заметки"];

/// What a progress card says of a download, and the share of it done:
/// what an earlier download left being checked, or the rest coming at
/// `rate` bytes a second.
fn downloaded(l: Lang, progress: Progress, rate: u64) -> (&'static str, String, Option<f32>) {
    match progress {
        Progress::Checking { done, total } => (l.pick(CHECKING_PART), format!("{} / {}", fmt::file_size(done), fmt::file_size(total)), Some(share(done, total))),
        Progress::Downloading { done, total } => (
            l.pick(DOWNLOADING),
            format!("{} / {} · {}/s", fmt::file_size(done), fmt::file_size(total), fmt::file_size(rate)),
            Some(share(done, total)),
        ),
    }
}

/// Skyward's own update: the build running now; Skyward's newest release
/// on GitHub, and updating to it; then what to note before updating.
pub fn skyward_update(cx: &Ctx) -> Frame {
    let (t, f, l) = (cx.t, cx.f, cx.state.lang);
    let mut c = long_page(cx);
    let top = t.m.content_top;
    let mut out = Vec::new();
    let y = widgets::section_first(&mut c, t, f, top, l.pick(["This phone", "Этот телефон"]));
    let y = widgets::info_card(&mut c, t, f, y, &[(l.pick(["Current version", "Текущая версия"]), VERSION.to_string())]);
    let y = widgets::section(&mut c, t, f, y, l.pick(["Online update", "Обновление по сети"]));
    let y = skyward_online(&mut c, cx, y, &mut out);
    let points = [
        l.pick([
            "The new version is downloaded from GitHub and goes into the recovery partitions once its SHA-256 checks out. The system and user data are left as they are.",
            "Новая версия загружается с GitHub и записывается в разделы recovery, как только её SHA-256 подтвердится. Система и пользовательские данные остаются как есть.",
        ]),
        l.pick([
            "The new version goes into the recovery partition of the slot that starts next, and into the other slot's where it holds Skyward.",
            "Новая версия записывается в раздел recovery слота, который запустится следующим, и в раздел recovery другого слота, если в нём установлен Skyward.",
        ]),
        l.pick([
            "Once updated, reboot to recovery to use the new version.",
            "После обновления перезагрузитесь в recovery, чтобы использовать новую версию.",
        ]),
    ];
    let y = widgets::notes(&mut c, t, f, y + t.m.card_gap, l.pick(NOTES), &points);
    Frame::new(t, fit(c, t, y), out, l.pick(SKYWARD_UPDATE), top)
}

/// Asking GitHub for Skyward's newest release, then updating to it: its
/// build for the phone downloaded and written into the recovery partitions,
/// how far that has come, and how it ended.
fn skyward_online(c: &mut Canvas, cx: &Ctx, y: f32, out: &mut Vec<Hit>) -> f32 {
    let (t, f, s, l) = (cx.t, cx.f, cx.state, cx.state.lang);
    let gap = t.m.card_gap;
    let newest = match &s.skyward_check {
        SkywardCheck::Asking => return widgets::list(c, t, f, y, &[Row::new(l.pick(CHECKING)).trail(Trail::None)]).bottom,
        SkywardCheck::Development => return widgets::footnote(c, t, f, y, l.pick(NO_RELEASE_TO_LOOK_FOR)),
        SkywardCheck::Answered(n) => n,
        other => {
            let mut row = Row::new(l.pick(CHECK_FOR_UPDATES));
            if !s.online() {
                row = row.subtitle(l.pick(WLAN_FIRST));
            }
            let check = widgets::list(c, t, f, y, &[row]);
            out.extend(hits(&check.rows, &[Action::CheckSkyward]));
            return match other {
                SkywardCheck::Failed(why) => widgets::footnote(c, t, f, check.bottom, &fmt::unanswered(l, why)),
                _ => check.bottom,
            };
        }
    };
    let version = &newest.version;
    let (title, detail) = match (newest.newer, &newest.build) {
        (true, Some(build)) => {
            let size = fmt::file_size(build.size);
            (tr!(l, "New version {version}", "Новая версия {version}"), tr!(l, "Package {size}", "Пакет {size}"))
        }
        (true, None) => (
            tr!(l, "New version {version}", "Новая версия {version}"),
            l.pick(["No package for this phone", "Нет пакета для этого телефона"]).to_string(),
        ),
        (false, _) => (
            l.pick(["Up to date", "Актуально"]).to_string(),
            tr!(l, "The newest release is {version}", "Новейший выпуск — {version}"),
        ),
    };
    let y = widgets::list(c, t, f, y, &[Row::new(&title).icon(GREEN, Icon::Install).subtitle(&detail).trail(Trail::None)]).bottom;
    if !newest.newer || newest.build.is_none() {
        return y;
    }
    match &s.skyward_job {
        Some(job) if job.outcome.is_none() => {
            let (heading, detail, share) = match job.stage {
                None => (l.pick(GETTING_READY), String::new(), None),
                Some(SkywardStage::Download(p)) => downloaded(l, p, job.rate),
                Some(SkywardStage::Write { done, total }) => {
                    let part = share(done, total);
                    let p = percent(part);
                    (l.pick(["Writing the recovery partitions", "Запись разделов recovery"]), tr!(l, "{p}% done", "{p}% готово"), Some(part))
                }
            };
            let y = widgets::progress_card(c, t, f, y + gap, heading, &detail, share);
            if matches!(job.stage, Some(SkywardStage::Write { .. })) {
                return widgets::footnote(c, t, f, y, l.pick(["Don't power off until it's written.", "Не выключайте питание, пока не будет записано."]));
            }
            let stop = widgets::list(c, t, f, y + gap, &[Row::new(l.pick(if job.stopping { STOPPING } else { STOP })).trail(Trail::None)]);
            if !job.stopping {
                out.extend(hits(&stop.rows, &[Action::StopSkywardUpdate]));
            }
            stop.bottom
        }
        Some(SkywardJob { outcome: Some(SkywardEnd::Updated(slots)), .. }) => {
            let names = fmt::slots(l, slots);
            let detail = match (l, slots.len()) {
                (Lang::En, 1) => format!("Skyward {version} is in {names}'s recovery partition."),
                _ => tr!(
                    l,
                    "Skyward {version} is in the recovery partitions of {names}.",
                    "Skyward {version} находится в разделах recovery {names}.",
                ),
            };
            let y = widgets::list(c, t, f, y + gap, &[standing(l.pick(["Updated", "Обновлено"]), GREEN, Icon::Install, &detail)]).bottom;
            let restart = widgets::list(c, t, f, y + gap, &[Row::new(l.pick(["Reboot to recovery", "Перезагрузить в recovery"])).icon(ORANGE, Icon::Power)]);
            out.extend(hits(&restart.rows, &[Action::RebootTo(Target::Recovery)]));
            restart.bottom
        }
        job => {
            let mut y = y;
            match job.as_ref().and_then(|j| j.outcome.as_ref()) {
                Some(SkywardEnd::Stopped) => y = widgets::footnote(c, t, f, y, l.pick(["Download stopped.", "Загрузка остановлена."])),
                Some(SkywardEnd::Download(why)) => y = widgets::footnote(c, t, f, y, &fmt::skyward_download_failure(l, why)),
                Some(SkywardEnd::NotWritten(why)) => {
                    y = widgets::footnote(c, t, f, y, l.pick(["Couldn't update. Nothing was written.", "Не удалось обновить. Ничего не записано."]));
                    y = widgets::footnote(c, t, f, y, why);
                }
                Some(SkywardEnd::WriteFailed(slots, why)) => {
                    let names = fmt::slots(l, slots);
                    let warning = match (l, slots.len()) {
                        (Lang::En, 1) => format!(
                            "{}'s recovery partition was only partly written, so rebooting to recovery may not start Skyward. Update again, and reboot once it's done.",
                            capitalized(l, &names)
                        ),
                        _ => tr!(
                            l,
                            "The recovery partitions of {names} were only partly written, so rebooting to recovery may not start Skyward. Update again, and reboot once it's done.",
                            "Разделы recovery {names} были записаны лишь частично, поэтому перезагрузка в recovery может не запустить Skyward. Обновитесь снова и перезагрузитесь после завершения.",
                        ),
                    };
                    y = widgets::footnote(c, t, f, y, &warning);
                    y = widgets::footnote(c, t, f, y, why);
                }
                Some(SkywardEnd::Updated(_)) => unreachable!("an update that went in shows how it did"),
                None => {}
            }
            let mut row = Row::new(l.pick(["Download and update", "Загрузить и обновить"]));
            if !s.online() {
                row = row.subtitle(l.pick(WLAN_FIRST));
            }
            let update = widgets::list(c, t, f, y + gap, &[row]);
            out.extend(hits(&update.rows, &[Action::UpdateSkyward]));
            update.bottom
        }
    }
}

/// The WLAN switch; then the network chosen and how joining it goes, and
/// the networks in range, strongest first.
pub fn wlan(cx: &Ctx) -> Frame {
    let (t, f, s, l) = (cx.t, cx.f, cx.state, cx.state.lang);
    let w = s.wlan.as_ref().expect("the WLAN page shows on a phone whose WLAN is brought up");
    let mut c = long_page(cx);
    let mut out = Vec::new();
    let top = t.m.content_top;
    let on = !matches!(w.power, Power::Off | Power::Failed(_));
    let switch = widgets::list(&mut c, t, f, top, &[Row::new(l.pick(WLAN)).trail(Trail::Switch(on))]);
    out.extend(hits(&switch.rows, &[Action::Toggle(Switch::Wlan)]));
    let mut y = switch.bottom;
    // Whether it is brought up by itself at the next start, and whether the
    // password is kept, under the switch that turns it on.
    let kept = widgets::list(
        &mut c,
        t,
        f,
        y + t.m.card_gap,
        &[
            Row::new(l.pick(["Connect by itself", "Подключаться автоматически"]))
                .subtitle(l.pick(["Turns Wi-Fi on at the next start and joins the network joined last", "Включает Wi-Fi при следующем запуске и подключается к последней сети"]))
                .trail(Trail::Switch(s.wlan_auto)),
            Row::new(l.pick(["Keep the password", "Запоминать пароль"]))
                .subtitle(l.pick(["Keeps the network joined last and its password", "Хранит последнюю сеть и её пароль"]))
                .trail(Trail::Switch(s.wlan_keep)),
        ],
    );
    out.extend(hits(&kept.rows, &[Action::Toggle(Switch::WlanAuto), Action::Toggle(Switch::WlanKeep)]));
    y = kept.bottom;
    match &w.power {
        Power::Off => {
            let note = l.pick([
                "When on, system updates and new versions of Skyward can download over Wi-Fi.",
                "Когда включено, обновления системы и новые версии Skyward могут загружаться по Wi-Fi.",
            ]);
            y = widgets::footnote(&mut c, t, f, y, note)
        }
        Power::Failed(why) => y = widgets::footnote(&mut c, t, f, y, &tr!(l, "Couldn't turn on Wi-Fi: {why}", "Не удалось включить Wi-Fi: {why}")),
        Power::Starting => y = widgets::footnote(&mut c, t, f, y, l.pick(["Turning on Wi-Fi", "Включение Wi-Fi"])),
        Power::On => {
            let locked = |s: Security| !matches!(s, Security::Open | Security::Owe);
            // The network chosen stands out in the accent, as the stock
            // page shows the one it is on.
            if let Some(j) = &w.joined {
                let found = w.networks.iter().find(|n| n.ssid == j.ssid);
                let (signal, bands) = match (&j.stage, found) {
                    (Joining::Connected(link), found) => (Some(link.signal), found.map_or(Bands::of(link.frequency), |n| n.bands)),
                    (_, Some(n)) => (Some(n.signal), n.bands),
                    (_, None) => (None, Bands::default()),
                };
                let tag = fmt::band_tag(bands);
                let mut row = Row::new(&j.name).fan(signal.map_or(0, fmt::signal_level)).subtitle(fmt::joining(l, &j.stage));
                row = row.trail(Trail::Network { locked: locked(j.security) });
                if let Some(tag) = &tag {
                    row = row.tag(tag);
                }
                let joined = widgets::list_toned(&mut c, t, f, y + t.m.card_gap, &[row], &Tone::lit(t));
                out.extend(hits(&joined.rows, &[Action::OpenJoined]));
                out.push(Hit { rect: widgets::round_button(t, joined.rows[0]), action: Action::OpenJoined });
                y = joined.bottom;
                if let Joining::Connected(link) = &j.stage {
                    if let Some(note) = link.reach.as_ref().and_then(|r| fmt::reach(l, r)) {
                        y = widgets::footnote(&mut c, t, f, y, &note);
                    }
                }
            }
            let others: Vec<(usize, &Network)> =
                w.networks.iter().enumerate().filter(|(_, n)| w.joined.as_ref().is_none_or(|j| j.ssid != n.ssid)).collect();
            let nearby = l.pick(["Choose a nearby network", "Выберите сеть поблизости"]);
            let (below, rescan) = widgets::section_action(&mut c, t, f, y, nearby, l.pick(["Refresh", "Обновить"]));
            out.push(Hit { rect: rescan, action: Action::Rescan });
            y = below;
            if others.is_empty() {
                let looking = l.pick(["Looking for nearby networks", "Поиск сетей поблизости"]);
                y = widgets::footnote(&mut c, t, f, y - t.m.dp(10.0), looking) + t.m.card_gap;
            }
            let tags: Vec<Option<String>> = others.iter().map(|(_, n)| fmt::band_tag(n.bands)).collect();
            let mut rows: Vec<Row> = others
                .iter()
                .zip(&tags)
                .map(|((_, n), tag)| {
                    let row = Row::new(&n.name).fan(fmt::signal_level(n.signal)).trail(Trail::Network { locked: locked(n.security) });
                    match tag {
                        Some(tag) => row.tag(tag),
                        None => row,
                    }
                })
                .collect();
            // The list ends with adding a network by hand, as the stock one
            // does.
            rows.push(Row::new(l.pick(ADD_NETWORK)).tint(t.pal.accent).trail(Trail::None));
            let list = widgets::list(&mut c, t, f, y, &rows);
            let mut picks: Vec<Action> = others.iter().map(|(k, _)| Action::JoinNetwork(*k)).collect();
            picks.push(Action::AddNetwork);
            out.extend(hits(&list.rows, &picks));
            // The round buttons come after the rows they sit on, and a touch
            // finds the last control under it.
            for (rect, (k, _)) in list.rows.iter().zip(&others) {
                out.push(Hit { rect: widgets::round_button(t, *rect), action: Action::NetworkDetails(*k) });
            }
            y = list.bottom;
        }
    }
    Frame::new(t, fit(c, t, y), out, l.pick(WLAN), top)
}

const ADD_NETWORK: Text = ["Add network", "Добавить сеть"];
const SHOW_PASSWORD: Text = ["Show password", "Показать пароль"];
const CONNECT_KEY: Text = ["Connect", "Подключить"];
const SECURITY: Text = ["Security", "Безопасность"];

/// The password of the network chosen: its name, what is typed, a switch
/// to show it, and the keyboard.
pub fn wlan_password(cx: &Ctx) -> Frame {
    let (t, f, s, l) = (cx.t, cx.f, cx.state, cx.state.lang);
    let network = s.joining.as_ref().expect("the password page is for a network");
    let mut c = page(cx);
    let mut out = Vec::new();
    let top = t.m.content_top;
    let y = widgets::section_first(&mut c, t, f, top, &network.name);
    let shown = if s.show_password { s.password.clone() } else { "•".repeat(s.password.chars().count()) };
    let (y, caret) = widgets::text_field(&mut c, t, f, y, &shown);
    let show = widgets::list(&mut c, t, f, y + t.m.card_gap, &[Row::new(l.pick(SHOW_PASSWORD)).trail(Trail::Switch(s.show_password))]);
    out.extend(hits(&show.rows, &[Action::Toggle(Switch::ShowPassword)]));
    keyboard_hits(&mut c, t, f, s, l.pick(CONNECT_KEY), &mut out);
    Frame { caret: Some(caret), ..Frame::new(t, c, out, l.pick(["Enter password", "Введите пароль"]), top) }
}

/// A network added by hand: its name, how it is protected, its password
/// where it takes one with the switch that shows it, and the keyboard,
/// which types into the field touched. The protections open over the page
/// from their row, which alone then answers a touch.
pub fn wlan_add(cx: &Ctx) -> Frame {
    let (t, f, s, l) = (cx.t, cx.f, cx.state, cx.state.lang);
    let m = &t.m;
    let a = s.adding.as_ref().expect("the page adds a network");
    let mut c = page(cx);
    let mut out = Vec::new();
    let top = m.content_top;
    let field = |y: f32| Rect::new(m.margin, y, m.w as f32 - m.margin * 2.0, m.dp(56.0));
    let y = widgets::section_first(&mut c, t, f, top, l.pick(["Network name", "Имя сети"]));
    let (below, mut caret) = widgets::text_field(&mut c, t, f, y, &a.name);
    out.push(Hit { rect: field(y), action: Action::TypeInto(Field::Name) });
    // Held down while its list is open, as the stock row stays.
    let tone = if a.choosing { Tone::page(&t.pressed()) } else { Tone::page(t) };
    let protection = [Row::new(l.pick(SECURITY)).trail(Trail::Choice(fmt::security(l, a.security)))];
    let security = widgets::list_toned(&mut c, t, f, below + m.card_gap, &protection, &tone);
    out.extend(hits(&security.rows, &[Action::ChooseSecurity]));
    if a.security.password() {
        let y = widgets::section(&mut c, t, f, security.bottom, l.pick(["Password", "Пароль"]));
        let shown = if s.show_password { s.password.clone() } else { "•".repeat(s.password.chars().count()) };
        let (below, typed) = widgets::text_field(&mut c, t, f, y, &shown);
        out.push(Hit { rect: field(y), action: Action::TypeInto(Field::Password) });
        if a.field == Field::Password {
            caret = typed;
        }
        let show = widgets::list(&mut c, t, f, below + m.card_gap, &[Row::new(l.pick(SHOW_PASSWORD)).trail(Trail::Switch(s.show_password))]);
        out.extend(hits(&show.rows, &[Action::Toggle(Switch::ShowPassword)]));
    }
    keyboard_hits(&mut c, t, f, s, l.pick(CONNECT_KEY), &mut out);
    if !a.choosing {
        return Frame { caret: Some(caret), ..Frame::new(t, c, out, l.pick(ADD_NETWORK), top) };
    }
    // A touch anywhere off the list closes it.
    out.clear();
    out.push(Hit { rect: Rect::new(0.0, 0.0, m.w as f32, m.h as f32), action: Action::CloseChoices });
    let names = ADDABLE.map(|p| fmt::security(l, p));
    let chosen = ADDABLE.iter().position(|&p| p == a.security);
    let held = ADDABLE.iter().position(|&p| s.held == Some(Action::PickSecurity(p)));
    let list = widgets::choices(&mut c, t, f, security.rows[0], &names, chosen, held);
    out.extend(hits(&list, &ADDABLE.map(Action::PickSecurity)));
    Frame::new(t, c, out, l.pick(ADD_NETWORK), top)
}

/// A network's details, in the stock page's grid: for the one chosen, how
/// joining it goes and what it gave, and forgetting it; for any other, how
/// strong and how protected it is.
pub fn wlan_network(cx: &Ctx) -> Frame {
    let (t, f, s, l) = (cx.t, cx.f, cx.state, cx.state.lang);
    let mut c = long_page(cx);
    let mut out = Vec::new();
    let top = t.m.content_top;
    let network = s.details.as_ref().expect("the details page is of a network");
    let joined = s.wlan.as_ref().and_then(|w| w.joined.as_ref()).filter(|j| j.ssid == network.ssid);
    let y = widgets::section_first(&mut c, t, f, top, &network.name);
    let detail = |icon, title, value: String| Detail { icon, title, value };
    let security = detail(Outline::Shield, l.pick(SECURITY), fmt::security(l, network.security).to_string());
    let signal = l.pick(["Signal strength", "Уровень сигнала"]);
    let title = l.pick(["Network details", "Сведения о сети"]);
    let Some(j) = joined else {
        let items = [
            detail(Outline::Wifi, l.pick(["Bands", "Диапазоны"]), fmt::bands(l, network.bands)),
            detail(Outline::Signal, signal, fmt::signal(l, network.signal).to_string()),
            security,
        ];
        let y = widgets::detail_grid(&mut c, t, f, y, &items);
        return Frame::new(t, fit(c, t, y), out, title, top);
    };
    let mut items = vec![detail(Outline::Info, l.pick(["Status", "Состояние"]), fmt::joining(l, &j.stage).to_string())];
    match &j.stage {
        Joining::Connected(link) => {
            if let Some(n) = link.generation {
                items.push(detail(Outline::Wifi, l.pick(["Wi-Fi standard", "Стандарт Wi-Fi"]), fmt::generation(l, n)));
            }
            if let Some(speed) = link.speed {
                items.push(detail(Outline::Speed, l.pick(["Link speed", "Скорость соединения"]), format!("{speed} Mbps")));
            }
            items.push(detail(Outline::Signal, signal, fmt::signal(l, link.signal).to_string()));
            items.push(security);
            items.push(detail(Outline::Address, l.pick(["IP address", "IP-адрес"]), link.address.to_string()));
            items.push(detail(Outline::Mask, l.pick(["Subnet mask", "Маска подсети"]), link.mask.to_string()));
            if let Some(router) = link.router {
                items.push(detail(Outline::Router, l.pick(["Router", "Роутер"]), router.to_string()));
            }
        }
        _ => {
            items.push(detail(Outline::Signal, signal, fmt::signal(l, network.signal).to_string()));
            items.push(security);
        }
    }
    let y = widgets::detail_grid(&mut c, t, f, y, &items);
    let forget = widgets::list(&mut c, t, f, y + t.m.card_gap, &[Row::new(l.pick(["Forget network", "Забыть сеть"])).tint(t.pal.danger).trail(Trail::None)]);
    out.extend(hits(&forget.rows, &[Action::LeaveNetwork]));
    Frame::new(t, fit(c, t, forget.bottom), out, title, top)
}

/// The files in internal storage offered to the network the phone is on:
/// the switch, the address a browser there opens, and how many requests it
/// has answered.
pub fn wireless(cx: &Ctx) -> Frame {
    let (t, f, s, l) = (cx.t, cx.f, cx.state, cx.state.lang);
    let mut c = long_page(cx);
    let mut out = Vec::new();
    let top = t.m.content_top;
    let switch = widgets::list(&mut c, t, f, top, &[Row::new(l.pick(WIRELESS)).icon(GREEN, Icon::Wlan).trail(Trail::Switch(s.wireless))]);
    out.extend(hits(&switch.rows, &[Action::Toggle(Switch::Wireless)]));
    let mut y = switch.bottom;
    if !s.wireless {
        let note = l.pick([
            "When on, a device on the same network opens the address given here in a browser to see, take and put files in internal storage.",
            "Когда включено, устройство в той же сети открывает указанный здесь адрес в браузере, чтобы просматривать, забирать и добавлять файлы во внутреннюю память.",
        ]);
        y = widgets::footnote(&mut c, t, f, y, note);
    } else if !s.data.storage_open() {
        let note = l.pick([
            "Waiting for the data partition to open: it starts on its own once it does.",
            "Ожидание открытия раздела data: как только он откроется, всё запустится само.",
        ]);
        y = widgets::footnote(&mut c, t, f, y, note);
    } else if let Some(url) = &s.wireless_url {
        let served = s.wireless_served.to_string();
        let rows = [
            Row::new(url.as_str()).icon(BLUE, Icon::Globe).subtitle(l.pick(["Open it in a browser", "Открыть в браузере"])),
            Row::new(l.pick(["Answered", "Отвечено"])).trail(Trail::Value(served.as_str())),
        ];
        let list = widgets::list(&mut c, t, f, y, &rows);
        y = list.bottom;
        let note = l.pick([
            "Any device on the network can open it, and put files there too. It goes when the switch is turned off.",
            "Любое устройство в сети может открыть её и положить туда файлы. Она исчезнет при выключении переключателя.",
        ]);
        y = widgets::footnote(&mut c, t, f, y, note);
    } else {
        let note = l.pick([
            "The phone has no address yet: join a network in Settings, and the address shows here.",
            "У телефона пока нет адреса: подключитесь к сети в «Настройках», и адрес появится здесь.",
        ]);
        y = widgets::footnote(&mut c, t, f, y, note);
    }
    Frame::new(t, fit(c, t, y), out, l.pick(WIRELESS), top)
}

const WIRELESS: Text = ["Wireless transfer", "Беспроводная передача"];
const REMEMBER: Text = ["Remember the screen lock", "Запоминать пароль экрана"];

/// A file taken from an address typed on the screen's keyboard, into the
/// downloads folder: the address, how far it has come, and how it ended.
pub fn fetch(cx: &Ctx) -> Frame {
    let (t, f, s, l) = (cx.t, cx.f, cx.state, cx.state.lang);
    let m = &t.m;
    let mut c = page(cx);
    let mut out = Vec::new();
    let top = m.content_top;
    let (mut y, caret) = widgets::text_field(&mut c, t, f, top, &s.fetch_url);
    // The keys take the bottom of the screen: what the page puts above them
    // is planned against the room left over.
    let floor = m.h as f32 - widgets::keyboard_height(t) - m.dp(8.0);
    let running = matches!(s.fetch, Fetch::Running { .. });
    // The button, of its own width in the middle of the page.
    let (bh, bw) = (m.dp(48.0), m.dp(220.0).min(m.w as f32 - m.margin * 2.0));
    let (bx, by) = ((m.w as f32 - bw) / 2.0, y + m.dp(14.0));
    c.fill_round_rect(bx, by, bw, bh, bh / 2.0, t.pal.accent);
    let label = l.pick(if running { STOP_FETCH } else { START_FETCH });
    c.text_center(&f.heading, label, bx + bw / 2.0, by + bh / 2.0, m.row_title_px, Rgba::rgb(255, 255, 255));
    let button = Rect::new(bx, by, bw, bh);
    out.push(Hit { rect: button, action: if running { Action::StopFetch } else { Action::Type(Cap::Enter) } });
    y = by + bh;
    // A link too long to type can be put in a small file from a computer,
    // and taken from there with one tap.
    y += m.dp(12.0);
    let from_file = l.pick([
        "Take the link from /sdcard/Download/link.txt",
        "Взять ссылку из /sdcard/Download/link.txt",
    ]);
    c.text_center(&f.body, from_file, m.w as f32 / 2.0, y + m.dp(16.0), m.sub_px, t.pal.accent);
    out.push(Hit { rect: Rect::new(m.margin, y, m.w as f32 - m.margin * 2.0, m.dp(32.0)), action: Action::FetchFromFile });
    y += m.dp(32.0);
    // A link too long for this keyboard can be typed on another device,
    // which opens a page this recovery serves; the code below is where that
    // page is.
    if let Some(at) = &s.fetch_portal {
        let caption = m.dp(56.0);
        let room = (floor - y - caption - m.dp(14.0)).max(m.dp(72.0));
        let side = m.dp(168.0).min(m.w as f32 - m.margin * 2.0).min(room);
        let x0 = (m.w as f32 - side) / 2.0;
        let quiet = 2.0f32;
        if let Ok(code) = qrcodegen::QrCode::encode_text(at, qrcodegen::QrCodeEcc::Medium) {
            let n = code.size();
            let cell = side / (n as f32 + quiet * 2.0);
            c.fill_round_rect(x0 - m.dp(10.0), y - m.dp(4.0), side + m.dp(20.0), side + m.dp(20.0), m.dp(12.0), Rgba::rgb(255, 255, 255));
            for row in 0..n {
                for col in 0..n {
                    if code.get_module(col, row) {
                        let px = x0 + (col as f32 + quiet) * cell;
                        let py = y + m.dp(4.0) + (row as f32 + quiet) * cell;
                        c.fill_rect(px, py, cell + 0.5, cell + 0.5, Rgba::rgb(12, 12, 16));
                    }
                }
            }
        }
        y += side + m.dp(16.0);
        let how = l.pick([
            "Scan this on another device, type the address there, and it fills the field above",
            "Отсканируйте на другом устройстве, введите адрес там, и он появится в поле выше",
        ]);
        y = widgets::footnote(&mut c, t, f, y - m.dp(8.0), how);
    }
    match &s.fetch {
        Fetch::Idle => {}
        Fetch::Running { name, done, total, rate } => {
            y = widgets::section(&mut c, t, f, y + t.m.dp(10.0), l.pick(["Downloading", "Загрузка"]));
            widgets::progress_bar(&mut c, t, t.m.margin, y + t.m.dp(2.0), t.m.w as f32 - t.m.margin * 2.0, share(*done, *total));
            let (came, all, pace) = (fmt::file_size(*done), fmt::file_size(*total), fmt::file_size(*rate));
            let how = tr!(l, "{came} of {all}", "{came} из {all}");
            let speed = tr!(l, "{pace}/s", "{pace}/s");
            let rows = [Row::new(name.as_str()).icon(BLUE, Icon::Folder).subtitle(&how).trail(Trail::Value(speed.as_str()))];
            widgets::list(&mut c, t, f, y + t.m.dp(18.0), &rows);
        }
        Fetch::Done { name } => {
            let saved = l.pick(["Saved in Download in internal storage", "Сохранено в Download во внутренней памяти"]);
            widgets::list(&mut c, t, f, y + t.m.dp(10.0), &[Row::new(name.as_str()).icon(GREEN, Icon::Folder).subtitle(saved).trail(Trail::None)]);
        }
        Fetch::Stopped { name } => {
            let kept = l.pick([
                "Stopped: what came is kept, and the same address goes on from there.",
                "Остановлено: полученное сохранено, и загрузка продолжится с того же адреса.",
            ]);
            widgets::list(&mut c, t, f, y + t.m.dp(10.0), &[Row::new(name.as_str()).icon(SLATE, Icon::Folder).subtitle(kept).trail(Trail::None)]);
        }
        Fetch::Failed(why) => {
            widgets::footnote(&mut c, t, f, y + t.m.dp(10.0), why);
        }
    }
    keyboard_hits(&mut c, t, f, s, l.pick(START_FETCH), &mut out);
    Frame { caret: Some(caret), ..Frame::new(t, c, out, l.pick(FETCH), top) }
}

const FETCH: Text = ["Download", "Загрузка"];
const START_FETCH: Text = ["Download", "Скачать"];
const STOP_FETCH: Text = ["Stop", "Остановить"];

/// The folder the files taken from the web are saved in: the folders of
/// internal storage, one by one, and the one the picker shows to take.
pub fn sort(cx: &Ctx) -> Frame {
    let (t, f, s, l) = (cx.t, cx.f, cx.state, cx.state.lang);
    let mut c = long_page(cx);
    let mut out = Vec::new();
    let top = t.m.content_top;
    let y = widgets::section_first(&mut c, t, f, top, l.pick(["Sort by", "Сортировать по"]));
    let keys = widgets::list(&mut c, t, f, y, &[
        Row::new(l.pick(SORT_NAME)).trail(Trail::Check(s.sort == SortKey::Name)),
        Row::new(l.pick(SORT_SIZE)).trail(Trail::Check(s.sort == SortKey::Size)),
        Row::new(l.pick(SORT_TIME)).trail(Trail::Check(s.sort == SortKey::Modified)),
    ]);
    out.extend(hits(&keys.rows, &[Action::SortBy(SortKey::Name), Action::SortBy(SortKey::Size), Action::SortBy(SortKey::Modified)]));
    let y = widgets::section(&mut c, t, f, keys.bottom, l.pick(["Order", "Порядок"]));
    let order = widgets::list(&mut c, t, f, y, &[
        Row::new(l.pick(SORT_ASC)).trail(Trail::Check(!s.sort_desc)),
        Row::new(l.pick(SORT_DESC)).trail(Trail::Check(s.sort_desc)),
    ]);
    out.extend(hits(&order.rows, &[Action::SortDesc(false), Action::SortDesc(true)]));
    let y = widgets::section(&mut c, t, f, order.bottom, l.pick(["Folders", "Папки"]));
    let folders = widgets::list(&mut c, t, f, y, &[Row::new(l.pick(SORT_FOLDERS)).trail(Trail::Switch(s.folders_first))]);
    out.extend(hits(&folders.rows, &[Action::Toggle(Switch::FoldersFirst)]));
    let note = l.pick([
        "The file manager and the pages that choose a file list entries in this order, and it is kept.",
        "Файловый менеджер и страницы выбора файла показывают записи в этом порядке, и он сохраняется.",
    ]);
    let y = widgets::footnote(&mut c, t, f, folders.bottom, note);
    Frame::new(t, fit(c, t, y), out, l.pick(SORT), top)
}

const SORT: Text = ["File order", "Сортировка файлов"];
const SORT_NAME: Text = ["Name", "Имя"];
const SORT_SIZE: Text = ["Size", "Размер"];
const SORT_TIME: Text = ["Date modified", "Дата изменения"];
const SORT_ASC: Text = ["Ascending", "По возрастанию"];
const SORT_DESC: Text = ["Descending", "По убыванию"];
const SORT_FOLDERS: Text = ["Folders first", "Сначала папки"];

/// The file manager, every entry of the folder shown, folders first.
pub fn files(cx: &Ctx) -> Frame {
    let (t, f, s, l) = (cx.t, cx.f, cx.state, cx.state.lang);
    // An entry picked to copy or move goes to the folder shown, from the
    // bar held at the bottom.
    let dock = s.clip.as_ref().map(|clip| {
        let go = l.pick(match clip.op {
            FileOp::Copy => ["Copy here", "Скопировать сюда"],
            FileOp::Move => ["Move here", "Переместить сюда"],
        });
        let (canvas, rects) = widgets::dock_bar(t, f, files::page(t), &[l.pick(CANCEL), go]);
        Dock { canvas, hits: hits(&rects, &[Action::Unpick, Action::Paste]) }
    });
    let button = Some((widgets::Glyph::Terminal, Action::TerminalHere));
    let root = l.pick(["Root", "Корень"]);
    Frame { dock, button, ..listing(cx, &s.files, root, None, l.pick(["No files", "Нет файлов"]), Action::OpenEntry, l.pick(FILES)) }
}

const CANCEL: Text = ["Cancel", "Отмена"];

/// A place's folders and the files in them that can be installed, to
/// choose one. A USB drive's page waits for one to be plugged in, and the
/// drive's files show as soon as it is.
pub fn choose(cx: &Ctx, place: Place) -> Frame {
    let (s, l) = (cx.state, cx.state.lang);
    let unreadable;
    let absent = match (place, &s.usb_drive) {
        (Place::UsbDrive, None) => Some((
            l.pick(["No USB drive", "Нет USB-накопителя"]),
            l.pick([
                "Connect a USB drive through OTG to see its files here.",
                "Подключите USB-накопитель через OTG, чтобы увидеть его файлы здесь.",
            ]),
        )),
        (Place::UsbDrive, Some(UsbDrive::Unreadable(why))) => {
            unreadable = tr!(
                l,
                "Use a USB drive formatted as FAT32 or exFAT. {why}",
                "Используйте USB-накопитель, отформатированный в FAT32 или exFAT. {why}",
            );
            Some((l.pick(USB_DRIVE_UNREADABLE), unreadable.as_str()))
        }
        _ => None,
    };
    let empty = l.pick(["Nothing here to install", "Здесь нечего устанавливать"]);
    listing(cx, s.folder(Screen::Choose(place)), place_name(l, place), absent, empty, Action::ChooseEntry, l.pick(["Choose a file", "Выберите файл"]))
}

/// A folder laid out as the stock file manager lays one out: the path as
/// capsules from `top`, held under the bar once scrolled there, then the
/// entries in one list on the plain page, each row drawn as it scrolls into
/// view and touched for `action`. In place of the list, `absent` says why
/// nothing of the folder can be shown, and `empty` that it holds nothing.
fn listing(
    cx: &Ctx,
    folder: &Folder,
    top: &str,
    absent: Option<(&str, &str)>,
    empty: &str,
    action: fn(usize) -> Action,
    title: &'static str,
) -> Frame {
    let (t, f) = (cx.t, cx.f);
    let bg = files::page(t);
    let mut c = sheet(cx, t.m.h, bg);
    let rows = match (absent, &folder.entries) {
        (Some((note, why)), _) => {
            files::note(&mut c, t, f, note, Some(why));
            None
        }
        (None, Err(why)) => {
            files::note(&mut c, t, f, cx.state.lang.pick(["This folder can't be read", "Эту папку нельзя прочитать"]), Some(why));
            None
        }
        (None, Ok(entries)) if entries.is_empty() => {
            files::note(&mut c, t, f, empty, None);
            None
        }
        (None, Ok(entries)) => Some(files::rows(t, f, entries, action)),
    };
    let pinned = Some(files::strip(t, f, top, &folder.path));
    Frame { bg, rows, pinned, ..Frame::new(t, c, Vec::new(), title, t.m.dp(files::STRIP_TOP_DP)) }
}

/// Row `i` of the page's long list over the whole of `c`, sized for it,
/// drawn as the row comes into view.
pub fn row(screen: Screen, cx: &Ctx, i: usize, c: &mut Canvas) {
    match screen {
        Screen::Files | Screen::Choose(_) => {
            let folder = cx.state.folder(screen);
            let entries = folder.entries.as_ref().expect("a folder listed draws its rows");
            files::row(c, cx.t, cx.f, cx.state.lang, &entries[i], folder.listed, cx.state.time_zone);
        }
        Screen::Installing => widgets::output_row(c, cx.t, cx.f, &cx.state.script_output, i),
        Screen::PartWork => {
            let work = cx.state.part_work.as_ref().expect("the page of partition work shows work");
            widgets::output_row(c, cx.t, cx.f, &work.output, i)
        }
        other => unreachable!("{other:?} draws no list row by row"),
    }
}

/// One entry of a folder, a file or a folder: what it is, and what can be
/// done with it. Deleting asks first, with a swipe.
pub fn file_info(cx: &Ctx) -> Frame {
    let (t, f, s, l) = (cx.t, cx.f, cx.state, cx.state.lang);
    let e = s.file.as_ref().expect("the file page shows an entry");
    let mut c = long_page(cx);
    let top = t.m.content_top;
    let folder = matches!(e.kind, FileKind::Folder { .. });
    let mut facts = vec![
        (l.pick(NAME), e.name.clone()),
        (l.pick(["Location", "Местоположение"]), Path::new(&s.files.path).join(&e.name).to_string_lossy().into_owned()),
    ];
    let holds = l.pick(["Holds", "Содержит"]);
    match &e.kind {
        FileKind::Link { target, .. } => facts.push((l.pick(["Points to", "Указывает на"]), target.clone())),
        FileKind::Folder { items: Some(n) } => {
            let items = match (l, n) {
                (Lang::En, 1) => "1 item".to_string(),
                (_, n) => tr!(l, "{n} items", "{n} элементов"),
            };
            facts.push((holds, items))
        }
        FileKind::Folder { items: None } => facts.push((holds, l.pick(UNREADABLE).to_string())),
        FileKind::File | FileKind::Special => {
            let (size, bytes) = (fmt::file_size(e.size), e.size);
            facts.push((l.pick(["Size", "Размер"]), tr!(l, "{size} ({bytes} bytes)", "{size} ({bytes} байт)")))
        }
    }
    facts.push((l.pick(["Modified", "Изменено"]), fmt::datetime(l, e.modified, s.time_zone)));
    let (mode, uid, gid) = (fmt::mode(e.mode), e.uid, e.gid);
    facts.push((l.pick(PERMISSIONS), tr!(l, "{mode}, owner {uid}, group {gid}", "{mode}, владелец {uid}, группа {gid}")));
    let y = widgets::info_card(&mut c, t, f, top, &facts);
    let mut rows = Vec::new();
    let mut acts = Vec::new();
    // Images are flashed; packages and zips that install themselves are
    // installed.
    if installs(e) {
        let image = Path::new(&e.name).extension().is_some_and(|ext| ext.eq_ignore_ascii_case("img"));
        rows.push(Row::new(l.pick(if image { ["Flash", "Прошить"] } else { ["Install", "Установить"] })));
        acts.push(Action::InstallEntry);
    }
    if e.kind == FileKind::File {
        rows.push(Row::new(l.pick(["Edit", "Изменить"])));
        acts.push(Action::EditEntry);
    }
    rows.extend([Row::new(l.pick(["Copy", "Копировать"])), Row::new(l.pick(["Move", "Переместить"])), Row::new(l.pick(RENAME))]);
    acts.extend([Action::PickEntry(FileOp::Copy), Action::PickEntry(FileOp::Move), Action::RenameEntry]);
    // A link's own permissions mean nothing; its target is changed where
    // it is.
    if !matches!(e.kind, FileKind::Link { .. }) {
        rows.push(Row::new(l.pick(CHANGE_PERMISSIONS)));
        acts.push(Action::OpenPermissions);
    }
    rows.push(Row::new(l.pick(["Delete", "Удалить"])).tint(t.pal.danger).trail(Trail::None));
    acts.push(Action::AskDelete);
    let actions = widgets::list(&mut c, t, f, y + t.m.card_gap, &rows);
    let mut out = hits(&actions.rows, &acts);
    let mut bottom = actions.bottom;
    if s.asking_delete {
        let name = &e.name;
        let what = match folder {
            true => tr!(
                l,
                "Delete “{name}” and everything in it? It can't be brought back.",
                "Удалить «{name}» и всё внутри? Вернуть это будет нельзя.",
            ),
            false => tr!(l, "Delete “{name}”? It can't be brought back.", "Удалить «{name}»? Вернуть это будет нельзя."),
        };
        let y = widgets::footnote(&mut c, t, f, actions.bottom, &what);
        let (label, danger) = confirm_label(Confirm::DeleteEntry, cx.state);
        let track = widgets::confirm(&mut c, t, f, y + t.m.dp(24.0), label, danger, s.confirm);
        out.push(Hit { rect: track, action: Action::Confirm(Confirm::DeleteEntry) });
        bottom = track.y + track.h;
    }
    let title = l.pick(if folder { ["Folder info", "Сведения о папке"] } else { ["File info", "Сведения о файле"] });
    Frame::new(t, fit(c, t, bottom), out, title, top)
}

const NAME: Text = ["Name", "Имя"];
const PERMISSIONS: Text = ["Permissions", "Разрешения"];
const CHANGE_PERMISSIONS: Text = ["Change permissions", "Изменить разрешения"];
const RENAME: Text = ["Rename", "Переименовать"];

/// The permission bits of the entry the file page shows, a switch each that
/// takes effect at once: reading, writing and running for its owner, its
/// group and everyone else, then the set-ID and sticky bits.
pub fn permissions(cx: &Ctx) -> Frame {
    let (t, f, s, l) = (cx.t, cx.f, cx.state, cx.state.lang);
    let e = s.file.as_ref().expect("the permissions page shows an entry");
    let mut c = long_page(cx);
    let top = t.m.content_top;
    let mode = e.mode & 0o7777;
    let bits = fmt::mode(e.mode);
    let facts = [(l.pick(NAME), e.name.clone()), (l.pick(PERMISSIONS), tr!(l, "{bits}, {mode:04o}", "{bits}, {mode:04o}"))];
    let mut y = widgets::info_card(&mut c, t, f, top, &facts);
    let mut out = Vec::new();
    let classes = [(["Owner", "Владелец"], 6), (["Group", "Группа"], 3), (["Others", "Остальные"], 0)];
    let (read, write, run) = (l.pick(["Read", "Чтение"]), l.pick(["Write", "Запись"]), l.pick(["Execute", "Выполнение"]));
    let groups = classes.map(|(title, shift)| (l.pick(title), [(read, 4 << shift), (write, 2 << shift), (run, 1 << shift)]));
    let special = (l.pick(["Special", "Особые"]), [("SUID", 0o4000), ("SGID", 0o2000), (l.pick(["Sticky", "Sticky"]), 0o1000)]);
    for (title, bits) in groups.into_iter().chain([special]) {
        y = widgets::section(&mut c, t, f, y, title);
        let rows = bits.map(|(name, bit)| Row::new(name).trail(Trail::Switch(mode & bit != 0)));
        let list = widgets::list(&mut c, t, f, y, &rows);
        out.extend(hits(&list.rows, &bits.map(|(_, bit)| Action::ModeBit(bit))));
        y = list.bottom;
    }
    Frame::new(t, fit(c, t, y), out, l.pick(CHANGE_PERMISSIONS), top)
}

/// A new name for the entry the file page shows, typed on the keyboard.
pub fn rename(cx: &Ctx) -> Frame {
    let (t, f, s, l) = (cx.t, cx.f, cx.state, cx.state.lang);
    let mut c = page(cx);
    let mut out = Vec::new();
    let top = t.m.content_top;
    let (y, caret) = widgets::text_field(&mut c, t, f, top, &s.new_name);
    widgets::footnote(&mut c, t, f, y, l.pick(["A name can't hold “/”. Tap Done to save it.", "Имя не может содержать «/». Нажмите Готово, чтобы сохранить."]));
    keyboard_hits(&mut c, t, f, s, l.pick(DONE), &mut out);
    Frame { caret: Some(caret), ..Frame::new(t, c, out, l.pick(RENAME), top) }
}

/// The folder an entry goes to already holds its name: what to do.
pub fn clash(cx: &Ctx) -> Frame {
    let (t, f, s, l) = (cx.t, cx.f, cx.state, cx.state.lang);
    let (clip, into, _) = s.clash.as_ref().expect("the clash page shows a clash");
    let name = Path::new(&clip.path).file_name().map_or_else(|| clip.path.clone(), |n| n.to_string_lossy().into_owned());
    let verb = l.pick(match clip.op {
        FileOp::Copy => ["copy", "копирует"],
        FileOp::Move => ["move", "перемещает"],
    });
    let mut c = page(cx);
    let top = t.m.content_top;
    let there = tr!(l, "“{into}” already holds an entry named “{name}”.", "В «{into}» уже есть запись с именем «{name}».");
    let y = widgets::paragraph(&mut c, t, f, top, &there);
    let skip = tr!(l, "Keeps the entry there, and doesn't {verb} this one", "Оставляет запись на месте и не {verb} эту");
    let choices = widgets::list(&mut c, t, f, y + t.m.dp(12.0), &[
        Row::new(l.pick(["Replace", "Заменить"])).subtitle(l.pick(["Writes over the entry there", "Перезаписывает запись на месте"])),
        Row::new(l.pick(["Skip", "Пропустить"])).subtitle(&skip),
        Row::new(l.pick(["Keep both", "Оставить оба"])).subtitle(l.pick(["Gives this one a new name", "Даёт этой записи новое имя"])),
    ]);
    let out = hits(&choices.rows, &[
        Action::Resolve(ClashChoice::Replace),
        Action::Resolve(ClashChoice::Skip),
        Action::Resolve(ClashChoice::KeepBoth),
    ]);
    Frame::new(t, c, out, l.pick(["Name taken", "Имя занято"]), top)
}

/// How work on files is told of: the words for what is done and for doing
/// it; the page's titles while it goes on and once it ended short; and
/// what stopping it may have left.
struct FileWords {
    done_en: &'static str,
    done_ru: &'static str,
    base_en: &'static str,
    base_ru: &'static str,
    titles: (Text, Text),
    stopped: Text,
}

fn file_words(task: &FileTask) -> FileWords {
    match task {
        FileTask::Copy { .. } => FileWords {
            done_en: "copied",
            done_ru: "скопировано",
            base_en: "copy",
            base_ru: "скопировать",
            titles: (["Copying", "Копирование"], ["Copy incomplete", "Копирование не завершено"]),
            stopped: ["Stopped. Part of it may have been copied into the folder.", "Остановлено. Часть могла быть скопирована в папку."],
        },
        FileTask::Move { .. } => FileWords {
            done_en: "moved",
            done_ru: "перемещено",
            base_en: "move",
            base_ru: "переместить",
            titles: (["Moving", "Перемещение"], ["Move incomplete", "Перемещение не завершено"]),
            stopped: ["Stopped. Part of it may have been moved into the folder.", "Остановлено. Часть могла быть перемещена в папку."],
        },
        FileTask::Delete { .. } => FileWords {
            done_en: "deleted",
            done_ru: "удалено",
            base_en: "delete",
            base_ru: "удалить",
            titles: (["Deleting", "Удаление"], ["Delete incomplete", "Удаление не завершено"]),
            stopped: ["Stopped. Part of it may have been deleted.", "Остановлено. Часть могла быть удалена."],
        },
    }
}

/// Copying, moving or deleting: how far it has come and a way to stop it,
/// then, if it did not finish, why.
pub fn file_job(cx: &Ctx) -> Frame {
    let (t, f, s, l) = (cx.t, cx.f, cx.state, cx.state.lang);
    let job = s.job.as_ref().expect("the job page shows work on files");
    let path = match &job.task {
        FileTask::Copy { path, .. } | FileTask::Move { path, .. } | FileTask::Delete { path } => path,
    };
    let words = file_words(&job.task);
    let (done_en, done_ru) = (words.done_en, words.done_ru);
    let (base_en, base_ru) = (words.base_en, words.base_ru);
    let name = Path::new(path).file_name().map_or_else(|| path.clone(), |n| n.to_string_lossy().into_owned());
    let share = |(done, total): (u64, u64)| if total == 0 { 0.0 } else { done as f32 / total as f32 };
    // The bar stays while the work measures, so that the button under it
    // does not move. Deleting, and copying entries that hold no bytes, is
    // counted in entries.
    let (detail, done) = match (job.progress, &job.task) {
        (None, _) => (l.pick(GETTING_READY).to_string(), 0.0),
        (Some((bytes, _)), FileTask::Copy { .. } | FileTask::Move { .. }) if bytes.1 > 0 => {
            let (did, total) = (fmt::file_size(bytes.0), fmt::file_size(bytes.1));
            (tr!(l, "{did} of {total} {done_en}", "{did} из {total} {done_ru}"), share(bytes))
        }
        (Some((_, entries)), _) => {
            let (did, total) = entries;
            (tr!(l, "{did} of {total} items {done_en}", "{did} из {total} элементов {done_ru}"), share(entries))
        }
    };
    let mut c = page(cx);
    let top = t.m.content_top;
    let y = widgets::progress_card(&mut c, t, f, top, &name, &detail, Some(done));
    let mut out = Vec::new();
    match &job.outcome {
        None => {
            let label = l.pick(if job.stopping { STOPPING } else { STOP });
            let stop = widgets::list(&mut c, t, f, y + t.m.card_gap, &[Row::new(label).trail(Trail::None)]);
            if !job.stopping {
                out.extend(hits(&stop.rows, &[Action::StopJob]));
            }
        }
        Some(end) => {
            let why = match end {
                JobEnd::Stopped => l.pick(words.stopped).to_string(),
                JobEnd::Failed(why) => tr!(l, "Couldn't {base_en} it: {why}", "Не удалось {base_ru}: {why}"),
                JobEnd::Done => unreachable!("work done leaves its page"),
            };
            let y = widgets::footnote(&mut c, t, f, y, &why);
            let finish = widgets::list(&mut c, t, f, y + t.m.card_gap, &[Row::new(l.pick(DONE)).trail(Trail::None)]);
            out.extend(hits(&finish.rows, &[Action::Back]));
        }
    }
    let title = l.pick(if job.outcome.is_none() { words.titles.0 } else { words.titles.1 });
    Frame { back: job.outcome.is_some(), ..Frame::new(t, c, out, title, top) }
}

/// A shell on a terminal: the screen it writes, the keys a phone keyboard
/// lacks, and the keyboard, every key of which goes to the shell.
pub fn terminal(cx: &Ctx) -> Frame {
    let (t, f, s) = (cx.t, cx.f, cx.state);
    let mut c = page(cx);
    let mut out = Vec::new();
    let top = t.m.content_top;
    keyboard_hits(&mut c, t, f, s, s.lang.pick(["Enter", "Ввод"]), &mut out);
    let l = term::layout(t, f, top);
    term::draw(&mut c, t, f, &l, cx.term);
    let held = cx.term.is_some_and(|term| term.ctrl);
    let keys = term::keys(&mut c, t, f, &l, &term::TERM_KEYS, held);
    out.extend(keys.into_iter().map(|(rect, key)| Hit { rect, action: Action::TermKey(key) }));
    Frame { scrollback: Some((l.card, l.pitch)), ..Frame::new(t, c, out, s.lang.pick(TERMINAL), top) }
}

/// A text file being edited: where it is, its lines wrapped on a card, the
/// keys a phone keyboard lacks for moving the cursor, and the keyboard.
/// Once it is changed, the round button saves it.
pub fn editor(cx: &Ctx) -> Frame {
    let (t, f, s) = (cx.t, cx.f, cx.state);
    let ed = s.editor.as_ref().expect("the editor page shows a file");
    let mut c = page(cx);
    let mut out = Vec::new();
    keyboard_hits(&mut c, t, f, s, s.lang.pick(["Return", "Ввод"]), &mut out);
    editor::caption(&mut c, t, f, &ed.path);
    let l = editor::layout(t, f);
    let caret = editor::draw(&mut c, t, f, &l, ed);
    let keys = term::keys(&mut c, t, f, &l, &editor::EDITOR_KEYS, false);
    out.extend(keys.into_iter().map(|(rect, key)| Hit { rect, action: Action::TermKey(key) }));
    let button = ed.changed.then_some((widgets::Glyph::Done, Action::SaveText));
    Frame { scrollback: Some((l.card, l.pitch)), caret, button, ..Frame::new(t, c, out, s.lang.pick(["Edit", "Изменить"]), t.m.content_top) }
}

/// The languages the pages speak, each named in itself, to speak another.
pub fn language(cx: &Ctx) -> Frame {
    let (t, f, l) = (cx.t, cx.f, cx.state.lang);
    let mut c = page(cx);
    let top = t.m.content_top;
    let rows: Vec<Row> = Lang::ALL.iter().map(|&k| Row::new(k.name()).trail(Trail::Check(k == l))).collect();
    let list = widgets::list(&mut c, t, f, top, &rows);
    let out = hits(&list.rows, &Lang::ALL.map(Action::Language));
    Frame::new(t, c, out, l.pick(LANGUAGE), top)
}

/// The place a zone is found by on the Settings page, short enough to sit
/// beside its name: the first city it lists.
fn zone_short(l: Lang, at: usize) -> &'static str {
    l.pick(TIME_ZONES[at].1).split(", ").next().unwrap_or_default()
}

/// The time zone the status bar's clock and the times on the pages keep.
pub fn time_zone(cx: &Ctx) -> Frame {
    let (t, f, s, l) = (cx.t, cx.f, cx.state, cx.state.lang);
    let mut c = long_page(cx);
    let top = t.m.content_top;
    let rows: Vec<Row> = TIME_ZONES
        .iter()
        .enumerate()
        .map(|(k, &(name, cities, _))| Row::new(l.pick(name)).subtitle(l.pick(cities)).trail(Trail::Check(k == s.time_zone)))
        .collect();
    let list = widgets::list(&mut c, t, f, top, &rows);
    let picks: Vec<Action> = (0..TIME_ZONES.len()).map(Action::TimeZone).collect();
    let out = hits(&list.rows, &picks);
    let note = l.pick([
        "The status bar and the pages tell times in the zone chosen.",
        "Строка состояния и страницы показывают время в выбранном часовом поясе.",
    ]);
    let bottom = widgets::footnote(&mut c, t, f, list.bottom, note);
    Frame::new(t, fit(c, t, bottom), out, l.pick(TIME_ZONE), top)
}

/// Saving the logs, as a bug report asks for them.
pub fn logs(cx: &Ctx) -> Frame {
    let (t, f, s, l) = (cx.t, cx.f, cx.state, cx.state.lang);
    let mut c = page(cx);
    let mut out = Vec::new();
    let top = t.m.content_top;
    let y = widgets::section_first(&mut c, t, f, top, l.pick(["Logs", "Журналы"]));
    let parts = widgets::list(&mut c, t, f, y, &[
        Row::new(l.pick(["Recovery log", "Журнал recovery"]))
            .subtitle(l.pick(["What the pages, installs and mounts did", "Что делали страницы, установки и монтирования"]))
            .trail(Trail::Check(s.logs[0])),
        Row::new(l.pick(["Kernel log", "Журнал ядра"]))
            .subtitle(l.pick(["The kernel's messages since this start", "Сообщения ядра с момента этого запуска"]))
            .trail(Trail::Check(s.logs[1])),
    ]);
    out.extend(hits(&parts.rows, &[Action::LogPart(0), Action::LogPart(1)]));
    let y = widgets::section(&mut c, t, f, parts.bottom, l.pick(["Save to", "Сохранить в"]));
    let storage = l.pick(match cx.state.data.storage_open() {
        true => ["The Skyward/logs folder", "Папка Skyward/logs"],
        false => DECRYPT_NEEDED,
    });
    let places = widgets::list(&mut c, t, f, y, &[
        Row::new(l.pick(["Temporary folder", "Временная папка"]))
            .subtitle(l.pick(["/skyward/run/logs, which adb pull copies to a computer", "/skyward/run/logs, которые adb pull копирует на компьютер"]))
            .trail(Trail::Check(s.logs_to == 0)),
        Row::new(place_name(l, Place::Storage)).subtitle(storage).trail(Trail::Check(s.logs_to == 1)),
    ]);
    out.extend(hits(&places.rows, &[Action::LogsTo(0), Action::LogsTo(1)]));
    let note = l.pick([
        "Each export goes in a folder named by when it was made. A reboot empties the temporary folder.",
        "Каждый экспорт сохраняется в папку с именем по времени создания. Перезагрузка очищает временную папку.",
    ]);
    let y = widgets::footnote(&mut c, t, f, places.bottom, note);
    let export = widgets::button(&mut c, t, f, y + t.m.dp(24.0), l.pick(["Export", "Экспорт"]));
    out.push(Hit { rect: export, action: Action::ExportLogs });
    Frame::new(t, c, out, l.pick(LOGS), top)
}

/// Skyward's licence notice in Russian, ahead of the words the licence gives
/// for a program's notice; English reads those words alone.
const LICENSE_NOTICE_RU: &str = "Skyward — свободное программное обеспечение: вы можете распространять и/или изменять его на условиях GNU General Public License, опубликованной Фондом свободного программного обеспечения, версии 3 или (по вашему выбору) любой более поздней версии.\n\n\
Skyward распространяется в надежде, что он будет полезен, но БЕЗ КАКИХ-ЛИБО ГАРАНТИЙ, даже без подразумеваемых гарантий КОММЕРЧЕСКОЙ ЦЕННОСТИ или ПРИГОДНОСТИ ДЛЯ КОНКРЕТНОЙ ЦЕЛИ. Подробности см. в GNU General Public License.";

const LICENSE_ORIGINAL: &str = "Skyward\nCopyright (C) 2026 Kirk Lin\nCopyright (C) 2026 RaeBaeXXX (the Xiaomi 13 fork)\n\n\
This program is free software: you can redistribute it and/or modify it under the terms of the GNU General Public License as published by the Free Software Foundation, either version 3 of the License, or (at your option) any later version.\n\n\
This program is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU General Public License for more details.\n\n\
You should have received a copy of the GNU General Public License along with this program. If not, see <https://www.gnu.org/licenses/>.";

const LICENSE_TERMS: Text = ["License", "Лицензия"];

/// Skyward's own licence.
pub fn license(cx: &Ctx) -> Frame {
    let (t, f, l) = (cx.t, cx.f, cx.state.lang);
    let mut c = long_page(cx);
    let top = t.m.content_top;
    let terms = l.pick([
        "GNU General Public License, version 3 or later (GPL-3.0-or-later)",
        "GNU General Public License, версия 3 или новее (GPL-3.0-or-later)",
    ]);
    let y = widgets::info_card(&mut c, t, f, top, &[(l.pick(LICENSE_TERMS), terms.to_string())]);
    let bottom = match l {
        Lang::Ru => {
            let y = widgets::paragraph(&mut c, t, f, y + t.m.dp(4.0), LICENSE_NOTICE_RU);
            let y = widgets::section(&mut c, t, f, y, "Оригинал");
            widgets::paragraph(&mut c, t, f, y - t.m.dp(16.0), LICENSE_ORIGINAL)
        }
        Lang::En => widgets::paragraph(&mut c, t, f, y + t.m.dp(4.0), LICENSE_ORIGINAL),
    };
    Frame::new(t, fit(c, t, bottom), Vec::new(), l.pick(LICENSE), top)
}

/// The monospaced typeface's licence, which names the copyright holders on
/// its first lines; the image carries the same file beside the font.
const MONO_LICENSE: &str = include_str!("../../../resources/fonts/KirkMono-OFL.txt");

/// The typefaces, as they name themselves, and the terms each is used under.
pub fn font_license(cx: &Ctx) -> Frame {
    let (t, f, l) = (cx.t, cx.f, cx.state.lang);
    let mut c = long_page(cx);
    let top = t.m.content_top;
    let (typeface, version, copyright) = (l.pick(["Typeface", "Шрифт"]), l.pick(["Version", "Версия"]), l.pick(["Copyright", "Авторские права"]));
    let y = widgets::section_first(&mut c, t, f, top, l.pick(["Interface text", "Текст интерфейса"]));
    let y = widgets::info_card(&mut c, t, f, y, &[
        (typeface, f.info.family.clone()),
        (version, f.info.version.clone()),
        (copyright, f.info.copyright.clone().expect("MiSans names its copyright holder")),
    ]);
    let misans = l.pick([
        "The interface text is set in Xiaomi's MiSans, used free of charge under the MiSans Font Intellectual Property License Agreement. The font files are unchanged, and come with Skyward alone, never on their own.\n\nThe agreement in full is on the MiSans website.",
        "Текст интерфейса набран в MiSans от Xiaomi, используемом бесплатно по Лицензионному соглашению на интеллектуальную собственность шрифта MiSans. Файлы шрифта не изменены и поставляются только вместе со Skyward, никогда отдельно.\n\nСоглашение полностью — на сайте MiSans.",
    ]);
    let y = widgets::paragraph(&mut c, t, f, y + t.m.dp(4.0), misans);
    let y = widgets::section(&mut c, t, f, y, l.pick(["Terminal text", "Текст терминала"]));
    let holders: Vec<&str> = MONO_LICENSE.lines().take_while(|l| l.starts_with("Copyright")).collect();
    let y = widgets::info_card(&mut c, t, f, y, &[
        (typeface, f.mono_info.family.clone()),
        (version, f.mono_info.version.clone()),
        (copyright, holders.join("\n")),
        (l.pick(LICENSE_TERMS), l.pick(["SIL Open Font License, version 1.1 (OFL-1.1)", "SIL Open Font License, версия 1.1 (OFL-1.1)"]).to_string()),
    ]);
    let mono = l.pick([
        "The terminal text is set in the monospaced Kirk Mono. Its license lets the font be used, changed and passed on freely, but not sold on its own.\n\nThe license in full is at /skyward/KirkMono-OFL.txt, and on openfontlicense.org.",
        "Текст терминала набран моноширинным Kirk Mono. Его лицензия позволяет свободно использовать, изменять и передавать шрифт, но не продавать его отдельно.\n\nЛицензия полностью — в /skyward/KirkMono-OFL.txt и на openfontlicense.org.",
    ]);
    let bottom = widgets::paragraph(&mut c, t, f, y + t.m.dp(4.0), mono);
    Frame::new(t, fit(c, t, bottom), Vec::new(), l.pick(FONTS), top)
}

const BOOT_SLOT: Text = ["Boot slot", "Загрузочный слот"];

/// Which slot the phone starts from next, as the boot control holds it,
/// and what it holds of each; picking the other and swiping switches.
/// While an update is in flight the slot stays as the update set it.
pub fn slot(cx: &Ctx) -> Frame {
    let (t, f, s, i, l) = (cx.t, cx.f, cx.state, cx.info, cx.state.lang);
    let mut c = page(cx);
    let mut out = Vec::new();
    let top = t.m.content_top;
    let title = l.pick(BOOT_SLOT);
    let Some(current) = i.slot else {
        widgets::list(&mut c, t, f, top, &[Row::new(l.pick(["This phone has no A/B slots", "На этом телефоне нет слотов A/B"])).trail(Trail::None)]);
        return Frame::new(t, c, out, title, top);
    };
    let slots = match &s.boot_slots {
        Some(Ok(slots)) => *slots,
        Some(Err(why)) => {
            let list = widgets::list(&mut c, t, f, top, &[Row::new(l.pick(["Couldn't read the boot slots", "Не удалось прочитать слоты загрузки"])).trail(Trail::None)]);
            widgets::footnote(&mut c, t, f, list.bottom, why);
            return Frame::new(t, c, out, title, top);
        }
        None => return Frame::new(t, c, out, title, top),
    };
    let chosen = s.next_slot.unwrap_or(slots.active);
    let y = widgets::section_first(&mut c, t, f, top, l.pick(["Next boot", "При следующей загрузке"]));
    let titles = [Slot::A, Slot::B].map(|slot| {
        let slot = slot.letter();
        tr!(l, "Slot {slot}", "Слот {slot}")
    });
    let subs = [Slot::A, Slot::B].map(|slot| {
        let k = slot.number() as usize;
        let which = l.pick(if slot == current { ["The system running now", "Система, работающая сейчас"] } else { ["The other slot", "Другой слот"] });
        match (slots.bootable[k], slots.successful[k]) {
            (false, _) => tr!(l, "{which}, marked unbootable", "{which}, помечен как незагружаемый"),
            (true, false) => tr!(l, "{which}, not yet fully started", "{which}, ещё не полностью запущен"),
            (true, true) => which.to_string(),
        }
    });
    let rows: Vec<Row> =
        [Slot::A, Slot::B].iter().enumerate().map(|(k, &slot)| Row::new(&titles[k]).subtitle(&subs[k]).trail(Trail::Check(slot == chosen))).collect();
    let list = widgets::list(&mut c, t, f, y, &rows);
    out.extend(hits(&list.rows, &[Slot::A, Slot::B].map(Action::PickSlot)));
    if let Some(note) = fmt::snapshot_slot_note(l, &i.snapshot) {
        widgets::footnote(&mut c, t, f, list.bottom, note);
        return Frame::new(t, c, out, title, top);
    }
    let note = l.pick([
        "If the other slot holds no whole system, the phone won't start after switching.",
        "Если в другом слоте нет целой системы, телефон не запустится после переключения.",
    ]);
    let y = widgets::footnote(&mut c, t, f, list.bottom, note);
    let (label, danger) = confirm_label(Confirm::Slot, cx.state);
    let track = widgets::confirm(&mut c, t, f, y + t.m.dp(24.0), label, danger, s.confirm);
    out.push(Hit { rect: track, action: Action::Confirm(Confirm::Slot) });
    Frame::new(t, c, out, title, top)
}

const MAIN_SYSTEM: Text = ["Main system", "Основная система"];
const SECOND_SYSTEM: Text = ["Second system", "Вторая система"];

/// What the home page and the systems page say once the second system's
/// start stopped.
const SECOND_FAILED: Text = ["The second system couldn't start", "Вторая система не смогла запуститься"];

/// The two systems as big cards one over the other, laid out as the boot
/// slot page lays out its slots: the main system, in the phone's own
/// partitions, and the second, whose files lie in the data partition, or
/// the place to install one, the one that starts next checked. Picking the
/// other and swiping writes its boot images into the slot that starts
/// next, and restarts into it. Under them, the second system, and removing
/// it.
pub fn systems(cx: &Ctx) -> Frame {
    let (t, f, s, i, l) = (cx.t, cx.f, cx.state, cx.info, cx.state.lang);
    let mut c = long_page(cx);
    let mut out = Vec::new();
    let top = t.m.content_top;
    let title = l.pick(DUAL_BOOT);
    // While the boot partitions are written, or the second system removed,
    // the page only says so: of a switch the system asked for, perhaps
    // before the systems are read.
    if let Some(work) = s.systems_work {
        let (heading, detail) = match work {
            SystemsWork::Switching(to) => {
                let name = match (&s.systems, to) {
                    (Some(Ok(systems)), Who::Installed) => Some(systems.installed.name.as_str()),
                    (Some(Ok(systems)), Who::Other) => systems.other.as_ref().map(|o| o.name.as_str()),
                    _ => None,
                };
                let heading = match name {
                    Some(name) => tr!(l, "Switching to {name}", "Переключение на {name}"),
                    None => l.pick(["Switching systems", "Переключение систем"]).to_string(),
                };
                (heading, l.pick(["Writing its boot images", "Запись загрузочных образов"]))
            }
            SystemsWork::Removing => (
                l.pick(["Removing the second system", "Удаление второй системы"]).to_string(),
                l.pick(["Giving its room back to the data partition", "Возвращает его место разделу data"]),
            ),
        };
        let y = widgets::progress_card(&mut c, t, f, top, &heading, detail, None);
        let y = widgets::footnote(&mut c, t, f, y, l.pick(["Don't power off until it's done.", "Не выключайте питание, пока не завершится."]));
        return Frame::new(t, fit(c, t, y), out, title, top);
    }
    let systems = match &s.systems {
        Some(Ok(systems)) => systems,
        Some(Err(why)) => {
            let list = widgets::list(&mut c, t, f, top, &[Row::new(l.pick(["Couldn't read the systems", "Не удалось прочитать системы"])).trail(Trail::None)]);
            let y = widgets::footnote(&mut c, t, f, list.bottom, why);
            return Frame::new(t, fit(c, t, y), out, title, top);
        }
        None => return Frame::new(t, fit(c, t, top), out, title, top),
    };
    let starts = starting(systems);
    let chosen = s.system_pick.unwrap_or(starts);
    let second_role = l.pick(SECOND_SYSTEM);
    let main = SystemCard {
        role: l.pick(MAIN_SYSTEM),
        name: &systems.installed.name,
        detail: systems.installed.version.as_deref().unwrap_or_default(),
        mark: SystemMark::Check(chosen == Who::Installed),
    };
    let second = match &systems.other {
        Some(other) => SystemCard { role: second_role, name: &other.name, detail: &other.version, mark: SystemMark::Check(chosen == Who::Other) },
        None => SystemCard {
            role: second_role,
            name: l.pick(["Install a system", "Установить систему"]),
            detail: l.pick(["Choose the second system's package", "Выберите пакет второй системы"]),
            mark: SystemMark::Opens,
        },
    };
    let y = widgets::section_first(&mut c, t, f, top, l.pick(["Next boot", "При следующей загрузке"]));
    let shown = widgets::system_cards(&mut c, t, f, y, &[main, second]);
    let second_opens = match systems.other {
        Some(_) => Action::PickSystem(Who::Other),
        None => Action::Open(Screen::Install),
    };
    out.extend(hits(&shown.rows, &[Action::PickSystem(Who::Installed), second_opens]));
    let mut y = shown.bottom + t.m.dp(12.0);
    if let Some(why) = &s.system_notice {
        y = widgets::info_card(&mut c, t, f, y, &[(l.pick(SECOND_FAILED), why.clone())]);
    }
    if chosen != starts {
        match (chosen, fmt::snapshot_system_note(l, &i.snapshot)) {
            (Who::Other, Some(note)) => y = widgets::footnote(&mut c, t, f, y, note),
            _ => {
                let (label, danger) = confirm_label(Confirm::SwitchSystem, s);
                let track = widgets::confirm(&mut c, t, f, y + t.m.dp(12.0), label, danger, s.confirm);
                out.push(Hit { rect: track, action: Action::Confirm(Confirm::SwitchSystem) });
                y = track.y + track.h;
            }
        }
    }
    let what = l.pick([
        "Tap a card to pick a system. Switching rewrites only the boot and init_boot partitions of the slot that starts next, and leaves the main system's other partitions as they are. The second system's files are kept in the data partition.",
        "Нажмите на карточку, чтобы выбрать систему. Переключение перезаписывает только разделы boot и init_boot слота, который запустится следующим, и оставляет остальные разделы основной системы как есть. Файлы второй системы хранятся в разделе data.",
    ]);
    y = widgets::footnote(&mut c, t, f, y, what);
    if systems.other.is_some() && systems.changed {
        let note = l.pick([
            "The main system's vendor_boot or dtbo partition has changed since the second system was installed, so the second system may not start properly.",
            "Раздел vendor_boot или dtbo основной системы изменился после установки второй системы, поэтому вторая система может не запуститься правильно.",
        ]);
        y = widgets::footnote(&mut c, t, f, y, note);
    }
    if let Some(other) = &systems.other {
        y = widgets::section(&mut c, t, f, y, second_role);
        let size = fmt::bytes(other.size);
        let facts = widgets::list(&mut c, t, f, y, &[Row::new(l.pick(["Space used", "Использовано места"])).trail(Trail::Text(&size))]);
        let mounted = matches!(s.data, Data::Open | Data::Locked { .. });
        let remove = Row::new(l.pick(["Remove the second system", "Удалить вторую систему"])).tint(t.pal.danger);
        let (remove, act) = match mounted {
            true => (remove.trail(Trail::None), Action::AskDelete),
            false => (remove.subtitle(l.pick(DECRYPT_NEEDED)), Action::Open(Screen::Decrypt)),
        };
        let removing = widgets::list(&mut c, t, f, facts.bottom + t.m.card_gap, &[remove]);
        out.extend(hits(&removing.rows, &[act]));
        y = removing.bottom;
        if s.asking_delete && mounted {
            let ask = match starts {
                Who::Other => l.pick([
                    "The phone goes back to the main system first, then the second system and all its data are removed for good.",
                    "Телефон сначала возвращается к основной системе, затем вторая система и все её данные удаляются навсегда.",
                ]),
                Who::Installed => l.pick([
                    "Remove the second system and all its data? It can't be brought back.",
                    "Удалить вторую систему и все её данные? Вернуть их будет нельзя.",
                ]),
            };
            let y2 = widgets::footnote(&mut c, t, f, y, ask);
            let (label, danger) = confirm_label(Confirm::RemoveSystem, s);
            let track = widgets::confirm(&mut c, t, f, y2 + t.m.dp(24.0), label, danger, s.confirm);
            out.push(Hit { rect: track, action: Action::Confirm(Confirm::RemoveSystem) });
            y = track.y + track.h;
        }
    }
    Frame::new(t, fit(c, t, y), out, title, top)
}

/// The phone's own fastbootd serving the computer: whether it runs, how far
/// the computer has the port, and where to go from here, chosen with the
/// keys and lit where they are.
pub fn fastboot(cx: &Ctx) -> Frame {
    let (t, f, s, i, l) = (cx.t, cx.f, cx.state, cx.info, cx.state.lang);
    let mut c = page(cx);
    let top = t.m.content_top;
    let running = l.pick(if s.fastboot.running { ["Running", "Работает"] } else { ["Not running", "Не запущено"] });
    let link = l.pick(match s.fastboot.link {
        UsbLink::Connected => ["Computer connected", "Компьютер подключён"],
        UsbLink::Connecting => ["Connecting", "Подключение"],
        UsbLink::Unplugged => ["No computer", "Нет компьютера"],
    });
    let slot = i.slot.map(|s| s.letter());
    let mut facts = vec![
        Row::new("fastbootd").trail(Trail::Text(running)),
        Row::new("USB").trail(Trail::Text(link)),
        Row::new(l.pick(["Codename", "Кодовое имя"])).trail(Trail::Text(&i.codename)),
    ];
    if let Some(letter) = slot {
        facts.push(Row::new(l.pick(CURRENT_SLOT)).trail(Trail::Text(letter)));
    }
    let y = widgets::section_first(&mut c, t, f, top, l.pick(["Status", "Состояние"]));
    let state = widgets::list(&mut c, t, f, y, &facts);
    let note = l.pick([
        "fastboot commands on the computer can flash partitions, the logical ones in super among them.",
        "команды fastboot на компьютере могут прошивать разделы, в том числе логические в super.",
    ]);
    let y = widgets::footnote(&mut c, t, f, state.bottom, note);
    let y = widgets::section(&mut c, t, f, y, l.pick(["Leave fastboot", "Выйти из fastboot"]));
    let rows: Vec<Row> = FASTBOOT_CHOICES.iter().map(|&(title, _)| Row::new(l.pick(title)).trail(Trail::None)).collect();
    let choices = widgets::list(&mut c, t, f, y, &rows);
    let actions: Vec<Action> = FASTBOOT_CHOICES.iter().map(|&(_, action)| action).collect();
    let out = hits(&choices.rows, &actions);
    widgets::footnote(&mut c, t, f, choices.bottom, l.pick([
        "Choose with the volume keys, and press the power key to confirm.",
        "Выбирайте кнопками громкости, а для подтверждения нажмите кнопку питания.",
    ]));
    Frame { back: false, ..Frame::new(t, c, out, "Fastboot", top) }
}

/// Where to go once what the system asked for has stopped or cannot be
/// done, in the order the pages list them.
const AFTERWARDS: [(Text, Action); 3] = [
    (["Reboot to system", "Перезагрузить в систему"], Action::RebootTo(Target::System)),
    (["Enter recovery", "Войти в recovery"], Action::Dismiss),
    (["Power off", "Выключить"], Action::RebootTo(Target::PowerOff)),
];

/// What work the system asked for says while it goes on of how the phone
/// leaves once it is done.
const POWERS_OFF_WHEN_DONE: Text = ["Powers off on its own once done.", "Выключится самостоятельно по завершении."];
const REBOOTS_WHEN_DONE: Text = ["Reboots on its own once done.", "Перезагрузится самостоятельно по завершении."];

/// The list of [`AFTERWARDS`] at `y`, and where it ends. Work the user
/// chose in the recovery goes back to its home page, rather than into it.
fn afterwards(c: &mut Canvas, t: &Theme, f: &Fonts, l: Lang, y: f32, asked: bool, out: &mut Vec<Hit>) -> f32 {
    let rows: Vec<Row> = AFTERWARDS
        .iter()
        .map(|&(title, action)| Row::new(l.pick(if action == Action::Dismiss && !asked { ["Back to home", "Вернуться на главный экран"] } else { title })).trail(Trail::None))
        .collect();
    let choices = widgets::list(c, t, f, y, &rows);
    out.extend(hits(&choices.rows, &AFTERWARDS.map(|(_, action)| action)));
    choices.bottom
}

/// Erasing: each part and how far it has come, and once erasing has
/// stopped, why. What the system asked for leaves on its own once it is
/// done, and offers where to go only if it stops; what the user chose
/// offers where to go once it ends either way.
pub fn erase(cx: &Ctx) -> Frame {
    use std::cmp::Ordering;
    let (t, f, s, l) = (cx.t, cx.f, cx.state, cx.state.lang);
    let e = &s.erase;
    let mut c = long_page(cx);
    let mut out = Vec::new();
    let top = t.m.content_top;
    let failed = matches!(e.outcome, Some(Err(_)));
    // Only a part that went wrong was under way; whatever else stopped
    // erasing stopped it before any part began.
    let begun = !failed || matches!(e.outcome, Some(Err(EraseStop::Failed(..))));
    let wiped = ["Wiped", "Очищено"];
    let heading = l.pick(match &e.outcome {
        None => WIPING,
        Some(Ok(())) => wiped,
        Some(Err(_)) => WIPE_INCOMPLETE,
    });
    let mut y = widgets::section_first(&mut c, t, f, top, heading);
    if !e.parts.is_empty() {
        let states: Vec<String> = (0..e.parts.len())
            .map(|k| match (k.cmp(&e.done), failed) {
                (Ordering::Less, _) => l.pick(wiped).into(),
                (Ordering::Equal, false) => match e.progress {
                    Some((done, total)) if total > 0 => format!("{} {}%", l.pick(WIPING), percent(share(done, total))),
                    _ => l.pick(WIPING).into(),
                },
                (Ordering::Equal, true) if begun => l.pick(["Unfinished", "Незавершено"]).into(),
                (Ordering::Greater, false) => l.pick(WAITING_TURN).into(),
                (Ordering::Equal | Ordering::Greater, true) => l.pick(["Not wiped", "Не очищено"]).into(),
            })
            .collect();
        let rows: Vec<Row> = e.parts.iter().zip(&states).map(|(&part, state)| Row::new(fmt::part(l, part)).trail(Trail::Text(state))).collect();
        y = widgets::list(&mut c, t, f, y, &rows).bottom;
    }
    let notes: Vec<String> = match (&e.outcome, e.asked, e.reset) {
        (Some(Err(stop)), _, _) => {
            let (what, detail) = fmt::erase_stop(l, stop);
            std::iter::once(what).chain(detail).collect()
        }
        (_, true, true) => {
            let leaving = match s.asked.shutdown_after {
                true => [
                    "All of this phone's data is being wiped, and it powers off on its own once done.",
                    "Все данные этого телефона стираются, по завершении он выключится сам.",
                ],
                false => [
                    "All of this phone's data is being wiped, and it reboots on its own once done.",
                    "Все данные этого телефона стираются, по завершении он перезагрузится сам.",
                ],
            };
            vec![l.pick(leaving).into()]
        }
        (_, true, false) => vec![l.pick(if s.asked.shutdown_after { POWERS_OFF_WHEN_DONE } else { REBOOTS_WHEN_DONE }).into()],
        (None, false, _) => vec![l.pick(KEEP_POWER).into()],
        (Some(Ok(())), false, true) => {
            vec![l.pick([
                "The data partition is made anew at the next start, which then takes longer.",
                "Раздел data создаётся заново при следующем запуске, который из-за этого займёт больше времени.",
            ]).into()]
        }
        (Some(Ok(())), false, false) => Vec::new(),
    };
    for note in &notes {
        y = widgets::footnote(&mut c, t, f, y, note);
    }
    if let (true, Some(reason)) = (e.asked, &s.asked.reason) {
        y = widgets::info_card(&mut c, t, f, y + t.m.dp(12.0), &[(l.pick(REASON), reason.clone())]);
    }
    if failed || (!e.asked && e.outcome.is_some()) {
        let below = widgets::section(&mut c, t, f, y, l.pick(ACTIONS));
        y = afterwards(&mut c, t, f, l, below, e.asked, &mut out);
    }
    let title = l.pick(match (e.reset, e.asked) {
        (true, _) => FACTORY_RESET,
        (false, true) => ["Wipe cache", "Очистить cache"],
        (false, false) => ["Wipe data", "Очистить данные"],
    });
    Frame { back: false, ..Frame::new(t, fit(c, t, y), out, title, top) }
}

const FACTORY_RESET: Text = ["Factory reset", "Сброс к заводским настройкам"];

/// The system could not start Android and asks whether to erase all user
/// data: start it again, look around the recovery first, or erase.
pub fn cant_start(cx: &Ctx) -> Frame {
    let (t, f, s, l) = (cx.t, cx.f, cx.state, cx.state.lang);
    let mut c = page(cx);
    let mut out = Vec::new();
    let top = t.m.content_top;
    let what = l.pick([
        "The system couldn't start, and this phone's data may be damaged. If every start ends on this page, a factory reset may help, though it wipes all of this phone's data.",
        "Система не смогла запуститься, и данные этого телефона могут быть повреждены. Если каждый запуск заканчивается этой страницей, может помочь сброс к заводским настройкам, хотя он сотрёт все данные этого телефона.",
    ]);
    let mut y = widgets::paragraph(&mut c, t, f, top, what);
    if let Some(reason) = &s.asked.reason {
        y = widgets::info_card(&mut c, t, f, y + t.m.dp(12.0), &[(l.pick(REASON), reason.clone())]);
    }
    y = widgets::section(&mut c, t, f, y, l.pick(ACTIONS));
    let rows = [
        Row::new(l.pick(["Try again", "Повторить"])).subtitle(l.pick(["Starts the system again", "Запускает систему заново"])).trail(Trail::None),
        Row::new(l.pick(["Enter recovery", "Войти в recovery"])).subtitle(l.pick(["Wipes nothing", "Ничего не очищает"])).trail(Trail::None),
    ];
    let choices = widgets::list(&mut c, t, f, y, &rows);
    out.extend(hits(&choices.rows, &[Action::RebootTo(Target::System), Action::Dismiss]));
    let y = widgets::footnote(&mut c, t, f, choices.bottom, l.pick(["Data a factory reset wipes can't be brought back.", "Данные, которые стирает сброс к заводским настройкам, вернуть нельзя."]));
    let (label, danger) = confirm_label(Confirm::FactoryReset, cx.state);
    let track = widgets::confirm(&mut c, t, f, y + t.m.dp(24.0), label, danger, s.confirm);
    out.push(Hit { rect: track, action: Action::Confirm(Confirm::FactoryReset) });
    Frame { back: false, ..Frame::new(t, c, out, l.pick(["Can't start the system", "Не удаётся запустить систему"]), top) }
}

/// Something the system asked of the recovery that it does not do: what
/// was asked, and where to go instead.
pub fn unhandled(cx: &Ctx) -> Frame {
    let (t, f, s, l) = (cx.t, cx.f, cx.state, cx.state.lang);
    let mut c = page(cx);
    let mut out = Vec::new();
    let top = t.m.content_top;
    let mut items = vec![(l.pick(["Request", "Запрос"]), s.asked.unhandled.clone().unwrap_or_default())];
    if let Some(reason) = &s.asked.reason {
        items.push((l.pick(REASON), reason.clone()));
    }
    let y = widgets::info_card(&mut c, t, f, top, &items);
    let y = widgets::footnote(&mut c, t, f, y, l.pick([
        "Skyward doesn't support this request, and changed nothing on this phone.",
        "Skyward не поддерживает этот запрос и ничего не изменил на этом телефоне.",
    ]));
    let y = widgets::section(&mut c, t, f, y, l.pick(ACTIONS));
    afterwards(&mut c, t, f, l, y, true, &mut out);
    Frame { back: false, ..Frame::new(t, c, out, l.pick(["Can't handle the request", "Не удаётся обработать запрос"]), top) }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::BackupRefusal;

    fn job(restore: bool, checks: bool) -> BackupJob {
        let plan = vec![("boot".into(), 200), ("system".into(), 800)];
        BackupJob { restore, place: Place::Storage, checks, plan, at: None, outcome: None, stopping: false }
    }

    fn at(item: usize, done: u64, of: u64, checking: bool) -> BackupAt {
        BackupAt { item, done, of, checking }
    }

    #[test]
    fn a_script_shows_its_last_line_until_skyward_goes_back() {
        let said = ["- Flashing boot".to_string(), "  ".to_string()];
        let l = Lang::Ru;
        assert_eq!(script_stage(l, None, &[]), ("Запуск скрипта установки", None));
        assert_eq!(script_stage(Lang::En, None, &[]), ("Running the install script", None));
        assert_eq!(script_stage(l, Some(&Stage::Script { thousandths: 250 }), &said), ("- Flashing boot", Some(0.25)), "past a blank line");
        assert_eq!(script_stage(l, Some(&Stage::Reflashing { done: 1, total: 4 }), &said), ("Повторная прошивка Skyward", Some(0.25)));
        assert_eq!(update_stage(Lang::En, Some(&Stage::Reflashing { done: 3, total: 4 })), ("Flashing Skyward again".to_string(), Some(0.75)));
        assert_eq!(update_stage(Lang::Ru, Some(&Stage::Writing { done: 1, total: 4 })).0, "Шаг 1 из 3: запись новой системы 25%");
    }

    #[test]
    fn a_backup_checks_its_parts_last_and_a_restore_first() {
        // Halfway through writing system, 600 of the 1000 bytes: the first
        // of two passes, then the second.
        assert_eq!(job_progress(&job(false, true), &at(1, 400, 800, false)), (0.3, 600));
        assert_eq!(job_progress(&job(false, true), &at(1, 400, 800, true)), (0.8, 600));
        assert_eq!(job_progress(&job(false, false), &at(1, 400, 800, false)), (0.6, 600), "without a check, writing is all");
        assert_eq!(job_progress(&job(true, true), &at(0, 100, 200, true)), (0.05, 100));
        assert_eq!(job_progress(&job(true, true), &at(0, 100, 200, false)), (0.55, 100));
    }

    #[test]
    fn each_part_says_where_it_stands() {
        let states = |j: &BackupJob| (0..2).map(|k| part_state(Lang::Ru, j, k)).collect::<Vec<_>>();
        let mut b = job(false, true);
        assert_eq!(states(&b), ["Ожидание", "Ожидание"]);
        b.at = Some(at(1, 400, 800, false));
        assert_eq!(states(&b), ["Скопировано", "Копирование 50%"]);
        b.at = Some(at(0, 50, 200, true));
        assert_eq!(states(&b), ["Проверка 25%", "Скопировано"]);
        b.outcome = Some(BackupEnd::Failed("the backup does not hold what was stored".into()));
        assert_eq!(states(&b), ["Незавершено", "Не проверено"]);
        b.outcome = Some(BackupEnd::Saved { name: "20261021-141639".into(), skipped: 0 });
        assert_eq!(states(&b), ["Проверено", "Проверено"]);
        let mut r = job(true, true);
        r.at = Some(at(1, 0, 800, true));
        assert_eq!(states(&r), ["Проверено", "Проверка 0%"]);
        r.at = Some(at(0, 200, 400, false));
        assert_eq!(states(&r), ["Восстановление 50%", "Проверено"]);
        r.outcome = Some(BackupEnd::Failed("write /dev/block/by-name/boot_b".into()));
        assert_eq!(states(&r), ["Незавершено", "Не восстановлено"]);
        r.at = None;
        r.outcome = Some(BackupEnd::Refused(BackupRefusal::DataReset));
        assert_eq!(states(&r), ["Не начато", "Не начато"]);
    }

    #[test]
    fn each_part_says_where_it_stands_in_the_language() {
        let mut b = job(false, true);
        b.at = Some(at(1, 400, 800, false));
        assert_eq!((0..2).map(|k| part_state(Lang::En, &b, k)).collect::<Vec<_>>(), ["Backed up", "Backing up 50%"]);
        let mut r = job(true, true);
        r.at = Some(at(0, 200, 400, false));
        assert_eq!((0..2).map(|k| part_state(Lang::Ru, &r, k)).collect::<Vec<_>>(), ["Восстановление 50%", "Проверено"]);
    }

    #[test]
    fn a_phrase_that_opens_a_line_starts_with_a_capital_in_english() {
        assert_eq!(capitalized(Lang::En, "empties the data partition"), "Empties the data partition");
        assert_eq!(capitalized(Lang::Ru, "очищает раздел data"), "очищает раздел data");
    }
}
