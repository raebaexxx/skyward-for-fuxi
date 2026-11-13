use crate::app::{
    BackupRefusal, Data, EraseStop, FileKind, FileRow, ItemEnd, MountFailure, Part, PartKind, PartStop, Place, Source, TermEnd, BACKUP_PARTS,
    TIME_ZONES,
};
use crate::lang::{Lang, Text};
use crate::tr;
use jiff::civil::DateTime;
use jiff::tz::TimeZone;
use jiff::Timestamp;
use skyward_info::{Battery, Charge, Cpu, PatchLevel, Storage};
use skyward_install::ota::Trouble;
use skyward_install::{Failure, ImageKind, Ramdisk, Refusal};
use skyward_net::download::Failure as DownloadFailure;
use skyward_net::reach::Reach;
use skyward_net::release::Unanswered;
use skyward_net::supplicant::Security;
use skyward_net::wlan::{Bands, Joined, Power, Stage as WlanStage, Wlan};
use skyward_vab::{Hold, Slot, SnapshotState, Standing};
use std::sync::OnceLock;

const GIB: f64 = 1024.0 * 1024.0 * 1024.0;

/// Marketed RAM size: the smallest standard module size that holds `MemTotal`.
pub fn ram(kib: u64) -> String {
    const SIZES: [f64; 16] = [1.0, 1.5, 2.0, 3.0, 4.0, 6.0, 8.0, 10.0, 12.0, 16.0, 18.0, 20.0, 24.0, 32.0, 48.0, 64.0];
    let gib = kib as f64 / (1024.0 * 1024.0);
    let size = SIZES.iter().copied().find(|&s| s >= gib).unwrap_or(gib.ceil());
    format!("{size:.1} GB")
}

/// Marketed storage size in GB: the smallest power of two (in GiB) that holds
/// the userdata partition.
pub fn capacity_gb(total: u64) -> u64 {
    let gib = total as f64 / GIB;
    let mut gb = 8;
    while (gb as f64) < gib {
        gb *= 2;
    }
    gb
}

pub fn capacity(total: u64) -> String {
    match capacity_gb(total) {
        gb if gb >= 1024 => format!("{}TB", gb / 1024),
        gb => format!("{gb}GB"),
    }
}

pub fn gb(bytes: u64) -> String {
    format!("{:.1}GB", bytes as f64 / 1e9)
}

/// `used/total` as the stock settings count it: everything that is not free
/// is used, against the marketed size.
pub fn storage_used(s: &Storage) -> Option<String> {
    let free = s.free?;
    let used = capacity_gb(s.total) as f64 - free as f64 / 1e9;
    Some(format!("{used:.1}GB/{}", capacity(s.total)))
}

/// A Russian count's noun in the form the number takes: 1 секунда,
/// 2 секунды, 5 секунд.
pub fn ru_plural<'a>(n: u64, one: &'a str, few: &'a str, many: &'a str) -> &'a str {
    let (last_two, last) = (n % 100, n % 10);
    if (11..=14).contains(&last_two) {
        many
    } else {
        match last {
            1 => one,
            2..=4 => few,
            _ => many,
        }
    }
}

/// `Восьмиядерный, до 3.19GHz`, `Octa-core, up to 3.19GHz`
pub fn cpu(l: Lang, c: &Cpu) -> String {
    let cores = match (l, c.cores) {
        (Lang::En, 1) => "Single-core".into(),
        (Lang::En, 2) => "Dual-core".into(),
        (Lang::En, 4) => "Quad-core".into(),
        (Lang::En, 6) => "Hexa-core".into(),
        (Lang::En, 8) => "Octa-core".into(),
        (Lang::En, n) => format!("{n}-core"),
        (Lang::Ru, 1) => "Одноядерный".into(),
        (Lang::Ru, 2) => "Двухъядерный".into(),
        (Lang::Ru, 4) => "Четырёхъядерный".into(),
        (Lang::Ru, 6) => "Шестиъядерный".into(),
        (Lang::Ru, 8) => "Восьмиядерный".into(),
        (Lang::Ru, n) => format!("{n}-ядерный"),
    };
    match c.max_khz {
        Some(khz) => {
            let ghz = format!("{:.2}", khz as f64 / 1e6);
            tr!(l, "{cores}, up to {ghz}GHz", "{cores}, до {ghz}GHz")
        }
        None => cores,
    }
}

/// A marketing name laid out the way the stock settings stack it in large
/// type: an edition credit ("by Leica") starts its own line.
pub fn model_lines(name: &str) -> String {
    name.replacen(" by ", "\nby ", 1)
}

pub fn panel((w, h): (u32, u32)) -> String {
    format!("{} x {}", w.max(h), w.min(h))
}

pub fn inches(d: f32) -> String {
    format!("{d}″")
}

pub fn mah(uah: u64) -> String {
    format!("{}mAh", uah / 1000)
}

pub fn charge(l: Lang, c: Charge) -> &'static str {
    l.pick(match c {
        Charge::Charging => ["Charging", "Зарядка"],
        Charge::Discharging | Charge::NotCharging => ["Not charging", "Не заряжается"],
        Charge::Full => ["Full", "Заряжен"],
        Charge::Unknown => ["Unknown", "Неизвестно"],
    })
}

/// `74% · Charging`
pub fn level(l: Lang, b: &Battery) -> String {
    format!("{}% · {}", b.level, charge(l, b.charge))
}

pub fn celsius(tenths: i32) -> String {
    format!("{}°C", degrees(tenths))
}

/// A temperature given in tenths of a degree, to the tenth: `35.2`.
pub fn degrees(tenths: i32) -> String {
    format!("{:.1}", tenths as f64 / 10.0)
}

pub fn volts(uv: u64) -> String {
    format!("{:.2}V", uv as f64 / 1e6)
}

/// The kernel's battery health in the language; a state it does not list
/// shows as reported.
pub fn health(l: Lang, h: &str) -> String {
    let text = match h {
        "Good" => ["Good", "Хорошо"],
        "Overheat" | "Hot" => ["Overheated", "Перегрев"],
        "Warm" => ["Warm", "Тёплый"],
        "Cool" => ["Cool", "Прохладно"],
        "Cold" => ["Cold", "Холодно"],
        "Dead" => ["Dead", "Разряжен"],
        "Over voltage" => ["Over voltage", "Перенапряжение"],
        "Over current" => ["Over current", "Перегрузка по току"],
        "Unspecified failure" => ["Failure", "Ошибка"],
        "Calibration required" => ["Needs calibration", "Требуется калибровка"],
        "No battery" => ["No battery", "Нет батареи"],
        "Unknown" => ["Unknown", "Неизвестно"],
        other => return other.to_string(),
    };
    l.pick(text).to_string()
}

/// Health and remaining capacity against design, e.g. `Good · Maximum capacity 93%`.
pub fn wear(l: Lang, b: &Battery) -> Option<String> {
    let state = b.health.as_deref().map(|h| health(l, h));
    let left = match (b.full, b.design) {
        (Some(full), Some(design)) if design > 0 => {
            let share = (full * 100 + design / 2) / design;
            Some(tr!(l, "Maximum capacity {share}%", "Максимальная ёмкость {share}%"))
        }
        _ => None,
    };
    match (state, left) {
        (Some(s), Some(l)) => Some(format!("{s} · {l}")),
        (s, l) => s.or(l),
    }
}

pub fn android(l: Lang, release: &str, patch_level: Option<&PatchLevel>) -> String {
    match patch_level {
        Some(p) => {
            let date = patch(l, p);
            tr!(l, "{release} | Android security update: {date}", "{release} | Обновление безопасности Android: {date}")
        }
        None => release.to_string(),
    }
}

pub fn verified(l: Lang, state: &str) -> String {
    let text = match state {
        "green" => ["Green · Verified with the official key", "Зелёный · Подтверждено официальным ключом"],
        "yellow" => ["Yellow · Verified with a custom key", "Жёлтый · проверено пользовательским ключом"],
        "orange" => ["Orange · Unlocked, so the system isn't verified", "Оранжевый · Разблокировано, поэтому система не проверена"],
        "red" => ["Red · Verification failed", "Красный · Проверка не пройдена"],
        other => return other.to_string(),
    };
    l.pick(text).to_string()
}

pub fn snapshot(l: Lang, s: &SnapshotState) -> &'static str {
    l.pick(match s {
        SnapshotState::None => ["No update waiting to merge", "Нет обновления, ожидающего объединения"],
        SnapshotState::Snapshotted => ["Update written, waiting to merge", "Обновление записано, ожидает объединения"],
        SnapshotState::Merging => ["Merging the update", "Объединение обновления"],
        SnapshotState::MergeFailed => ["The update failed to merge", "Не удалось объединить обновление"],
        SnapshotState::Unknown => ["Couldn't read the update's status", "Не удалось прочитать состояние обновления"],
    })
}

/// Why super is off limits while an update is in flight; `None` when it is not.
pub fn snapshot_note(l: Lang, s: &SnapshotState) -> Option<&'static str> {
    let text = match s {
        SnapshotState::None => return None,
        SnapshotState::Snapshotted | SnapshotState::Merging => {
            ["Partitions in super aren't flashed until the update merges", "Разделы в super не прошиваются, пока обновление не объединится"]
        }
        SnapshotState::MergeFailed => [
            "Partitions in super aren't flashed until the system deals with the update",
            "Разделы в super не прошиваются, пока система не обработает обновление",
        ],
        SnapshotState::Unknown => [
            "Partitions in super aren't flashed until the update's status can be read",
            "Разделы в super не прошиваются, пока не удастся прочитать состояние обновления",
        ],
    };
    Some(l.pick(text))
}

/// The home page's warning of a battery too low to install on: how low it
/// is, and what installing waits for.
pub fn battery_low(l: Lang, level: u8, charging: bool) -> (String, String) {
    let least = skyward_install::least_charge(charging);
    (
        tr!(l, "Battery at {level}%", "Заряд батареи {level}%"),
        tr!(l, "Charge it past {least}% to install", "Зарядите его выше {least}%, чтобы установить"),
    )
}

/// The home page's word that the phone restarts from the slot it does not
/// run from now.
pub fn next_boot(l: Lang, next: Slot, now: Slot) -> (String, String) {
    let (next, now) = (next.letter(), now.letter());
    (
        tr!(l, "Restarts from slot {next}", "Перезапустится со слота {next}"),
        tr!(l, "Running from slot {now} now", "Сейчас запущено со слота {now}"),
    )
}

/// Why the slot to start next stays as an update in flight set it; `None`
/// when none is.
pub fn snapshot_slot_note(l: Lang, s: &SnapshotState) -> Option<&'static str> {
    let text = match s {
        SnapshotState::None => return None,
        SnapshotState::Snapshotted | SnapshotState::Merging => {
            ["The boot slot isn't switched until the update merges.", "Загрузочный слот не переключается, пока обновление не объединится."]
        }
        SnapshotState::MergeFailed => {
            [
                "The boot slot isn't switched until the system deals with the update.",
                "Загрузочный слот не переключается, пока система не обработает обновление.",
            ]
        }
        SnapshotState::Unknown => {
            [
                "The boot slot isn't switched until the update's status can be read.",
                "Загрузочный слот не переключается, пока не удастся прочитать состояние обновления.",
            ]
        }
    };
    Some(l.pick(text))
}

/// What the recovery says as it starts, of a switch between the systems
/// that a restart or a loss of power cut off: finished, or why not.
pub fn switch_resumed(l: Lang) -> &'static str {
    l.pick(["The system switch interrupted last time is finished", "Переключение систем, прерванное в прошлый раз, завершено"])
}

pub fn switch_unfinished(l: Lang, why: &str) -> String {
    tr!(
        l,
        "The system switch interrupted last time couldn't be finished: {why}",
        "Переключение систем, прерванное в прошлый раз, не удалось завершить: {why}",
    )
}

/// Why the second system does not start yet while an update is in flight,
/// which the main system has to start once more to merge; `None` when none
/// is.
pub fn snapshot_system_note(l: Lang, s: &SnapshotState) -> Option<&'static str> {
    let text = match s {
        SnapshotState::None => return None,
        SnapshotState::Snapshotted | SnapshotState::Merging => [
            "The second system doesn't start until the system update merges. Start the main system first, and let the update merge.",
            "Вторая система не запускается, пока обновление системы не объединится. Сначала запустите основную систему и дайте обновлению объединиться.",
        ],
        SnapshotState::MergeFailed => [
            "The second system doesn't start until the main system deals with the update.",
            "Вторая система не запускается, пока основная система не обработает обновление.",
        ],
        SnapshotState::Unknown => [
            "The second system doesn't start until the update's status can be read.",
            "Вторая система не запускается, пока не удастся прочитать состояние обновления.",
        ],
    };
    Some(l.pick(text))
}

/// What an install script that writes to super does to an update in
/// flight; `None` when none is known to be.
pub fn snapshot_script_note(l: Lang, s: &SnapshotState) -> Option<&'static str> {
    let text = match s {
        SnapshotState::None | SnapshotState::Unknown => return None,
        SnapshotState::Snapshotted | SnapshotState::Merging => [
            "A system update hasn't finished merging. A flashable zip that rewrites partitions in super keeps it from ever finishing.",
            "Объединение обновления системы не завершено. zip-архив для прошивки, перезаписывающий разделы в super, не даёт ему завершиться.",
        ],
        SnapshotState::MergeFailed => [
            "A system update failed to merge, and the system has to deal with it. A flashable zip that rewrites partitions in super keeps it from ever finishing.",
            "Не удалось объединить обновление системы, и система должна с этим разобраться. zip-архив для прошивки, перезаписывающий разделы в super, не даёт этому завершиться.",
        ],
    };
    Some(l.pick(text))
}

/// Why the slot's partitions in super are not backed up while an update is
/// in flight; `None` when none is.
pub fn snapshot_backup_note(l: Lang, s: &SnapshotState) -> Option<&'static str> {
    let text = match s {
        SnapshotState::None => return None,
        SnapshotState::Snapshotted | SnapshotState::Merging => {
            [
                "System partitions aren't backed up until the update merges",
                "Системные разделы не копируются, пока обновление не объединится",
            ]
        }
        SnapshotState::MergeFailed => {
            [
                "System partitions aren't backed up until the system deals with the update",
                "Системные разделы не копируются, пока система не обработает обновление",
            ]
        }
        SnapshotState::Unknown => {
            [
                "System partitions aren't backed up until the update's status can be read",
                "Системные разделы не копируются, пока не удастся прочитать состояние обновления",
            ]
        }
    };
    Some(l.pick(text))
}

/// Why backing up, or restoring, did not start.
pub fn backup_refusal(l: Lang, r: &BackupRefusal, restore: bool) -> String {
    match r {
        BackupRefusal::Room { needed, free } => {
            let (needed, free) = (gb(*needed), gb(*free));
            tr!(
                l,
                "Not enough storage. The backup needs {needed}, and {free} is free.",
                "Недостаточно памяти. Для копии нужно {needed}, а свободно {free}.",
            )
        }
        BackupRefusal::Update(state) => {
            let what = l.pick(match restore {
                true => ["restored", "восстановлено"],
                false => ["backed up", "скопировано"],
            });
            let parts = if restore { "The boot and system partitions" } else { "System partitions" };
            match state {
                SnapshotState::Snapshotted | SnapshotState::Merging => tr!(
                    l,
                    "{parts} aren't {what} until the update merges. You can see it in Advanced > System update.",
                    "{parts} не {what}, пока обновление не объединится. Это можно увидеть в разделе «Дополнительно» > «Обновление системы».",
                ),
                SnapshotState::MergeFailed => tr!(
                    l,
                    "The update failed to merge. {parts} aren't {what} until the system deals with it.",
                    "Не удалось объединить обновление. {parts} не {what}, пока система с этим не разберётся.",
                ),
                SnapshotState::Unknown => tr!(
                    l,
                    "{parts} aren't {what} while the update's status can't be read.",
                    "{parts} не {what}, пока не удаётся прочитать состояние обновления.",
                ),
                SnapshotState::None => unreachable!("no update in flight holds anything back"),
            }
        }
        BackupRefusal::OtherPhone(device) => tr!(
            l,
            "This backup is from another device ({device}) and can't be restored to this phone.",
            "Эта резервная копия с другого устройства ({device}), и её нельзя восстановить на этот телефон.",
        ),
        BackupRefusal::Resized(name) => tr!(
            l,
            "{name} isn't the size it was at the backup, so the system may have been updated. The backup's system partitions can't be restored to the current system.",
            "Размер {name} не совпадает с тем, что был при создании копии, поэтому система, возможно, обновлялась. Системные разделы из резервной копии нельзя восстановить на текущую систему.",
        ),
        BackupRefusal::DataReset => l
            .pick([
                "The data partition was made anew after the backup, as a factory reset does, so the backup's data partition can't be restored.",
                "Раздел data был создан заново после создания копии, как при сбросе к заводским настройкам, поэтому раздел data из копии восстановить нельзя.",
            ])
            .into(),
    }
}

/// What the update page says of an update standing so, and the line under
/// it on what comes next; `next` is the slot the bootloader starts next,
/// where the boot control could tell.
pub fn update(l: Lang, s: &Standing, next: Option<Slot>) -> (&'static str, Option<String>) {
    let line = |text: Text| Some(l.pick(text).to_string());
    match s {
        Standing::None => (l.pick(["No update waiting to merge", "Нет обновления, ожидающего объединения"]), None),
        Standing::Unknown => (
            l.pick(["Couldn't read the update's status", "Не удалось прочитать состояние обновления"]),
            line(["Couldn't read the update's records in /metadata", "Не удалось прочитать записи обновления в /metadata"]),
        ),
        Standing::Writing => (
            l.pick(["The update isn't fully written", "Обновление записано не полностью"]),
            line(["Installing goes on once the system starts", "Установка продолжится после запуска системы"]),
        ),
        Standing::Installed(installed) => {
            let slot = installed.letter();
            let detail = match next {
                Some(next) if next == *installed => tr!(
                    l,
                    "After a reboot the new system starts from slot {slot}, and merges once it has started",
                    "После перезагрузки новая система запустится со слота {slot} и объединит обновление после запуска",
                ),
                Some(_) => tr!(
                    l,
                    "Reboot from the system's System update to start slot {slot}'s new system",
                    "Перезагрузитесь из «Обновления системы» в системе, чтобы запустить новую систему слота {slot}",
                ),
                None => tr!(l, "Merges once slot {slot}'s new system starts", "Объединится, когда запустится новая система слота {slot}"),
            };
            (l.pick(["Update installed", "Обновление установлено"]), Some(detail))
        }
        Standing::Waiting => (
            l.pick(["Update waiting to merge", "Обновление ожидает объединения"]),
            line([
                "Merges once the new system starts, or can be merged here",
                "Объединится после запуска новой системы, или можно объединить здесь",
            ]),
        ),
        Standing::Compressed => (
            l.pick(["Update waiting to merge", "Обновление ожидает объединения"]),
            line([
                "An update of this kind merges in the system, once it starts",
                "Обновление такого типа объединяется в системе, как только начнётся",
            ]),
        ),
        Standing::Merging(_) => (l.pick(["Merging the update", "Объединение обновления"]), None),
        Standing::Failed(code) => (
            l.pick(["The update failed to merge", "Не удалось объединить обновление"]),
            Some(tr!(l, "Merging goes on after a reboot ({code:?})", "Объединение продолжится после перезагрузки ({code:?})")),
        ),
        Standing::Merged => (
            l.pick(["Update merged", "Обновление объединено"]),
            line([
                "The files the update left are cleared when the system next starts",
                "Файлы, оставленные обновлением, удаляются при следующем запуске системы",
            ]),
        ),
    }
}

/// Where an update standing so is, beside the entry of its page.
pub fn update_value(l: Lang, s: &Standing) -> &'static str {
    l.pick(match s {
        Standing::None => ["None", "Нет"],
        Standing::Unknown => ["Unreadable", "Не читается"],
        Standing::Writing => ["Not fully written", "Записано не полностью"],
        Standing::Installed(_) => ["Installed", "Установлено"],
        Standing::Waiting | Standing::Compressed => ["Waiting to merge", "Ожидание объединения"],
        Standing::Merging(_) => ["Merging", "Объединение"],
        Standing::Failed(_) => ["Merge failed", "Не удалось объединить"],
        Standing::Merged => ["Merged", "Объединено"],
    })
}

pub fn data(l: Lang, d: &Data) -> &'static str {
    l.pick(match d {
        Data::Closed | Data::Opening | Data::Failed(_) => ["Not mounted", "Не смонтировано"],
        Data::Locked { .. } => ["Locked", "Заблокировано"],
        Data::Open => ["Decrypted", "Расшифровано"],
        Data::Erased => ["Wiped", "Очищено"],
    })
}

/// A part of the phone erasing goes through, as its page names it.
pub fn part(l: Lang, p: Part) -> &'static str {
    l.pick(match p {
        Part::Data => ["data partition", "раздел data"],
        Part::Cache => ["cache partition", "раздел cache"],
        Part::Metadata => ["metadata partition", "раздел metadata"],
        Part::AppData => ["App data", "Данные приложений"],
        Part::Media => ["Internal storage", "Внутренняя память"],
        Part::AppCaches => ["App caches", "Кэш приложений"],
        Part::SystemCaches => ["System caches", "Системные кэши"],
        Part::Art => ["ART cache", "Кэш ART"],
        Part::Logs => ["Logs", "Журналы"],
        Part::Updates => ["Downloaded updates", "Загруженные обновления"],
    })
}

/// A part of the phone erasing goes through, as an English sentence names
/// it.
fn part_in_sentence(p: Part) -> &'static str {
    match p {
        Part::Data => "the data partition",
        Part::Cache => "the cache partition",
        Part::Metadata => "the metadata partition",
        Part::AppData => "app data",
        Part::Media => "internal storage",
        Part::AppCaches => "app caches",
        Part::SystemCaches => "system caches",
        Part::Art => "the ART cache",
        Part::Logs => "the logs",
        Part::Updates => "the downloaded updates",
    }
}

/// A name that opens an English title, as it reads within a sentence: its
/// first letter small, unless the word is written all in capitals.
fn in_sentence(name: &str) -> String {
    let mut chars = name.chars();
    match (chars.next(), chars.next()) {
        (Some(first), Some(second)) if first.is_uppercase() && !second.is_uppercase() => {
            first.to_lowercase().chain(name[first.len_utf8()..].chars()).collect()
        }
        _ => name.to_string(),
    }
}

/// Things named in a row: with commas, each after the first as a sentence
/// names it.
pub fn list<'a>(l: Lang, names: impl IntoIterator<Item = &'a str>) -> String {
    let names: Vec<String> = names.into_iter().enumerate().map(|(k, n)| if k > 0 { in_sentence(n) } else { n.to_string() }).collect();
    names.join(l.pick([", ", ", "]))
}

/// Users by how the pages call them: the owner, then the others by number,
/// as "the owner and user 999".
pub fn users(l: Lang, users: &[u32]) -> String {
    let names: Vec<String> = users
        .iter()
        .map(|&u| match u {
            0 => l.pick(["the owner", "владелец"]).to_string(),
            u => tr!(l, "user {u}", "пользователь {u}"),
        })
        .collect();
    match names.as_slice() {
        [.., last] if names.len() > 1 => format!("{} {} {last}", names[..names.len() - 1].join(", "), l.pick(["and", "и"])),
        _ => names.join(""),
    }
}

/// Why erasing did not start or stopped short, and what the system said of
/// it when there is that.
pub fn erase_stop(l: Lang, s: &EraseStop) -> (String, Option<String>) {
    let said = |text: Text| l.pick(text).to_string();
    match s {
        EraseStop::Update(Hold::Unconfirmed) => (
            said([
                "The current slot's system update hasn't finished, so nothing can be wiped yet. Merge it in Advanced > System update, then try again.",
                "Обновление системы текущего слота не завершено, поэтому пока ничего нельзя очистить. Объедините его в «Дополнительно > Обновление системы» и повторите попытку.",
            ]),
            None,
        ),
        EraseStop::Update(Hold::Merging) => (
            said([
                "A system update is merging, so nothing can be wiped yet. You can see it in Advanced > System update; try again once it has merged.",
                "Обновление системы объединяется, поэтому пока ничего нельзя очистить. Это видно в разделе «Дополнительно > Обновление системы»; повторите попытку после завершения объединения.",
            ]),
            None,
        ),
        EraseStop::Update(Hold::MergeFailed) => (
            said([
                "A system update failed to merge, so nothing can be wiped yet. Start the system to deal with it, then try again.",
                "Не удалось объединить обновление системы, поэтому пока ничего нельзя очистить. Запустите систему, чтобы разобраться, и повторите попытку.",
            ]),
            None,
        ),
        EraseStop::NotRebuilt => (
            said([
                "This phone's system doesn't make an emptied data partition anew when it starts, so it can't be wiped.",
                "Система этого телефона не создаёт очищенный раздел data заново при запуске, поэтому его нельзя очистить.",
            ]),
            None,
        ),
        EraseStop::NoCache => (said(["This phone has no cache partition.", "В этом телефоне нет раздела cache."]), None),
        EraseStop::Erased => (
            said([
                "The data partition is empty and will be made anew when the phone next starts. There's nothing to wipe.",
                "Раздел data пуст и будет создан заново при следующем запуске телефона. Очищать нечего.",
            ]),
            None,
        ),
        EraseStop::Closed(why) => (
            said(["Couldn't open the data partition, so nothing was wiped.", "Не удалось открыть раздел data, поэтому ничего не очищено."]),
            Some(why.clone()),
        ),
        EraseStop::Locked(locked) => {
            let who = users(l, locked);
            let what = tr!(
                l,
                "The data of {who} isn't decrypted, so nothing was wiped. Enter the screen lock in Decrypt, then try again.",
                "Данные ({who}) не расшифрованы, поэтому ничего не очищено. Введите экранный пароль в разделе «Расшифровка» и повторите попытку.",
            );
            (what, None)
        }
        EraseStop::Failed(p, why) => {
            let what = match l {
                Lang::En => format!("Couldn't wipe {}.", part_in_sentence(*p)),
                Lang::Ru => format!("Не удалось очистить: {}.", part(l, *p)),
            };
            (what, Some(why.clone()))
        }
    }
}

/// How work on whole partitions of a kind is told of: the verb its
/// sentences are built on, and the words Russian's own sentences use.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Verb {
    /// Under way: Wiping.
    going: &'static str,
    /// Done to a partition: wiped.
    done: &'static str,
    /// The work as a thing: Wipe, as in Wipe failed.
    noun: &'static str,
    /// The verb itself: wipe, as in Couldn't wipe vendor.
    base: &'static str,
    going_ru: &'static str,
    done_ru: &'static str,
    noun_ru: &'static str,
    base_ru: &'static str,
    failed_ru: &'static str,
    not_done_ru: &'static str,
    complete_ru: &'static str,
    incomplete_ru: &'static str,
}

impl Verb {
    pub fn of(kind: &PartKind) -> Verb {
        let (going, done, noun, base) = match kind {
            PartKind::Disable => ("Wiping", "wiped", "Wipe", "wipe"),
            PartKind::Unshare => ("Unsharing", "unshared", "Unsharing", "unshare"),
            PartKind::Repair => ("Repairing", "repaired", "Repair", "repair"),
            PartKind::Remake(_) => ("Formatting", "formatted", "Format", "format"),
        };
        let (going_ru, done_ru, noun_ru, base_ru, failed_ru, not_done_ru, complete_ru, incomplete_ru) = match kind {
            PartKind::Disable => (
                "Идёт очистка",
                "очищено",
                "Очистка",
                "очистить",
                "Не удалось очистить",
                "Не очищено",
                "Очистка завершена",
                "Очистка не завершена",
            ),
            PartKind::Unshare => (
                "Идёт разделение",
                "разделено",
                "Разделение",
                "разделить",
                "Не удалось разделить",
                "Не разделено",
                "Разделение завершено",
                "Разделение не завершено",
            ),
            PartKind::Repair => (
                "Идёт проверка",
                "проверено",
                "Проверка",
                "проверить",
                "Не удалось проверить",
                "Не проверено",
                "Проверка завершена",
                "Проверка не завершена",
            ),
            PartKind::Remake(_) => (
                "Идёт форматирование",
                "отформатировано",
                "Форматирование",
                "отформатировать",
                "Не удалось отформатировать",
                "Не отформатировано",
                "Форматирование завершено",
                "Форматирование не завершено",
            ),
        };
        Verb { going, done, noun, base, going_ru, done_ru, noun_ru, base_ru, failed_ru, not_done_ru, complete_ru, incomplete_ru }
    }

    /// Under way: Repairing.
    pub fn going(self, l: Lang) -> String {
        let (going, going_ru) = (self.going, self.going_ru);
        tr!(l, "{going}", "{going_ru}")
    }

    /// Ended, with every partition done: Repair complete.
    pub fn complete(self, l: Lang) -> String {
        let (noun, complete_ru) = (self.noun, self.complete_ru);
        tr!(l, "{noun} complete", "{complete_ru}")
    }

    /// Ended partway through the partitions: Wipe incomplete.
    pub fn incomplete(self, l: Lang) -> String {
        let (noun, incomplete_ru) = (self.noun, self.incomplete_ru);
        tr!(l, "{noun} incomplete", "{incomplete_ru}")
    }

    /// Gone wrong: Repair failed.
    pub fn failed(self, l: Lang) -> String {
        let (noun, failed_ru) = (self.noun, self.failed_ru);
        tr!(l, "{noun} failed", "{failed_ru}")
    }

    /// A partition the work ended before: Not repaired.
    pub fn not_done(self, l: Lang) -> String {
        let (done, not_done_ru) = (self.done, self.not_done_ru);
        tr!(l, "Not {done}", "{not_done_ru}")
    }

    /// What the page asks while the work goes on.
    pub fn hold_on(self, l: Lang) -> String {
        tr!(l, "Don't power off until it's done.", "Не выключайте телефон до завершения.")
    }

    /// What could not be done to the partition `name`: Couldn't repair
    /// vendor.
    pub fn failed_on(self, l: Lang, name: &str) -> String {
        let (base, base_ru) = (self.base, self.base_ru);
        tr!(l, "Couldn't {base} {name}", "Не удалось {base_ru} {name}")
    }
}

/// How a partition's work ended, as its row says it, `verb` telling of the
/// work: Repaired, Repair failed.
pub fn item_end(l: Lang, end: &ItemEnd, verb: Verb) -> String {
    let text = match end {
        ItemEnd::Erased => ["Wiped", "Очищено"],
        ItemEnd::Unshared => ["Unshared", "Общие блоки разделены"],
        ItemEnd::NotShared(_) => ["Nothing to do", "Нечего выполнять"],
        ItemEnd::Clean => ["No errors found", "Ошибок не найдено"],
        ItemEnd::Fixed => ["Repaired", "Восстановлено"],
        ItemEnd::Made => ["Formatted", "Отформатировано"],
        ItemEnd::Failed(_) => return verb.failed(l),
    };
    l.pick(text).to_string()
}

/// Why work on whole partitions stopped before going through them, on a
/// phone whose codename is `codename`, and the details behind it when
/// there are any.
pub fn part_stop(l: Lang, stop: &PartStop, codename: &str) -> (String, Option<String>) {
    match stop {
        PartStop::Refused(r) => (erase_request_refusal(l, r, codename), None),
        PartStop::Unmounted(base) => (
            tr!(
                l,
                "Mount the {base} partition in Mount, then try again.",
                "Смонтируйте раздел {base} на экране «Монтирование», затем повторите попытку.",
            ),
            None,
        ),
        PartStop::Failed(why) => (l.pick(UNEXPECTED).into(), Some(why.clone())),
    }
}

/// What the pages say of an error they have no words of their own for,
/// above what went wrong.
const UNEXPECTED: Text = ["Something went wrong.", "Что-то пошло не так."];

/// A file size, `7.1GB` or `96.0MB`.
pub fn bytes(n: u64) -> String {
    match n {
        n if n >= 1_000_000_000 => format!("{:.1}GB", n as f64 / 1e9),
        n if n >= 1_000_000 => format!("{:.1}MB", n as f64 / 1e6),
        n if n >= 1_000 => format!("{:.0}KB", n as f64 / 1e3),
        n => format!("{n}B"),
    }
}

/// The months, as English names them.
const MONTHS: [&str; 12] = ["January", "February", "March", "April", "May", "June", "July", "August", "September", "October", "November", "December"];

/// The months in Russian, in the form a day takes: 8 мая 2026.
const MONTHS_RU: [&str; 12] = [
    "января", "февраля", "марта", "апреля", "мая", "июня", "июля", "августа", "сентября", "октября", "ноября", "декабря",
];

/// A day of the calendar as the language writes it: `May 8, 2026`,
/// `8 мая 2026`.
fn day(l: Lang, year: i32, month: i32, d: i32) -> String {
    let name = MONTHS[(month - 1) as usize];
    let name_ru = MONTHS_RU[(month - 1) as usize];
    tr!(l, "{name} {d}, {year}", "{d} {name_ru} {year}")
}

/// `April 1, 2026`
pub fn patch(l: Lang, p: &PatchLevel) -> String {
    day(l, p.year.into(), p.month.into(), p.day.into())
}

/// The rules of each of [`TIME_ZONES`], read once from the ones the
/// recovery carries.
pub fn rules(zone: usize) -> &'static TimeZone {
    static RULES: OnceLock<Vec<TimeZone>> = OnceLock::new();
    let all = RULES.get_or_init(|| {
        TIME_ZONES.iter().map(|&(_, _, name)| TimeZone::get(name).expect("the recovery carries the rules of every zone listed")).collect()
    });
    &all[zone]
}

/// A Unix time on the calendar and clock of the zone at `zone` in
/// [`TIME_ZONES`]. A time past either end of the calendar, as a damaged
/// file's can be, is taken as that end.
fn local(unix: i64, zone: usize) -> DateTime {
    let unix = unix.clamp(Timestamp::MIN.as_second(), Timestamp::MAX.as_second());
    let at = Timestamp::from_second(unix).expect("a time within the calendar");
    at.to_zoned(rules(zone).clone()).datetime()
}

/// The day a Unix time falls on in the zone at `zone` in [`TIME_ZONES`],
/// `May 8, 2026`.
pub fn date(l: Lang, unix: i64, zone: usize) -> String {
    let d = local(unix, zone);
    day(l, d.year().into(), d.month().into(), d.day().into())
}

/// When a list entry last changed, as the stock file manager tells it
/// against `now`, when the folder was read: the time for today, month and
/// day for a file from this year, and the whole date for a folder from this
/// year or anything older; all in the zone at `zone` in [`TIME_ZONES`].
pub fn file_time(unix: i64, folder: bool, now: i64, zone: usize) -> String {
    let (day, today) = (local(unix, zone).date(), local(now, zone).date());
    if day == today {
        clock(unix, zone)
    } else if day.year() == today.year() && !folder {
        format!("{:02}/{:02}", day.month(), day.day())
    } else {
        format!("{}/{:02}/{:02}", day.year(), day.month(), day.day())
    }
}

/// A file's size as the stock file manager writes it: in steps of 1000,
/// going up a unit past 900, with at most two decimals and no trailing
/// zeros.
pub fn file_size(bytes: u64) -> String {
    const UNITS: [&str; 6] = ["B", "KB", "MB", "GB", "TB", "PB"];
    let (mut v, mut unit) = (bytes as f64, 0);
    while v > 900.0 && unit < UNITS.len() - 1 {
        v /= 1000.0;
        unit += 1;
    }
    let hundredths = (v * 100.0).round() as u64;
    let n = match hundredths {
        h if h % 100 == 0 => format!("{v:.0}"),
        h if h % 10 == 0 => format!("{v:.1}"),
        _ => format!("{v:.2}"),
    };
    format!("{n} {}", UNITS[unit])
}

/// A Unix time in the zone at `zone` in [`TIME_ZONES`], to the minute:
/// `October 19, 2026, 02:31`.
pub fn datetime(l: Lang, unix: i64, zone: usize) -> String {
    let (date, clock) = (date(l, unix, zone), clock(unix, zone));
    tr!(l, "{date}, {clock}", "{date}, {clock}")
}

/// The time of day in the zone at `zone` in [`TIME_ZONES`], `HH:MM`, as
/// the status bar shows it.
pub fn clock(unix: i64, zone: usize) -> String {
    let t = local(unix, zone);
    format!("{:02}:{:02}", t.hour(), t.minute())
}

/// A Unix time in the zone at `zone` in [`TIME_ZONES`], to the second, as
/// the name of what is saved then, which sorts by it: `20261021-133000`.
pub fn stamp(unix: i64, zone: usize) -> String {
    let t = local(unix, zone);
    format!("{:04}{:02}{:02}-{:02}{:02}{:02}", t.year(), t.month(), t.day(), t.hour(), t.minute(), t.second())
}

/// A file's type and permissions as `ls -l` prints them: `drwxr-xr-x`.
pub fn mode(mode: u32) -> String {
    let kind = match mode & 0o170_000 {
        0o040_000 => 'd',
        0o120_000 => 'l',
        0o020_000 => 'c',
        0o060_000 => 'b',
        0o010_000 => 'p',
        0o140_000 => 's',
        _ => '-',
    };
    let mut out = String::from(kind);
    // The set-user, set-group and sticky bits show in each class's x.
    for (shift, special, set, unset) in [(6, 0o4000, 's', 'S'), (3, 0o2000, 's', 'S'), (0, 0o1000, 't', 'T')] {
        let bits = (mode >> shift) & 7;
        out.push(if bits & 4 != 0 { 'r' } else { '-' });
        out.push(if bits & 2 != 0 { 'w' } else { '-' });
        out.push(match (bits & 1 != 0, mode & special != 0) {
            (true, true) => set,
            (false, true) => unset,
            (true, false) => 'x',
            (false, false) => '-',
        });
    }
    out
}

/// The line under an entry's name in the file list, laid out as the stock
/// file manager lays it out, with dates told against `now`, when the folder
/// was read, in the zone at `zone` in [`TIME_ZONES`].
pub fn file_line(l: Lang, f: &FileRow, now: i64, zone: usize) -> String {
    const DIVIDER: &str = "  |  ";
    match &f.kind {
        FileKind::Folder { items: Some(n) } => {
            let items = match (l, n) {
                (Lang::En, 1) => "1 item".to_string(),
                (Lang::En, _) => format!("{n} items"),
                (Lang::Ru, _) => format!("{n} {}", ru_plural(*n as u64, "элемент", "элемента", "элементов")),
            };
            format!("{}{DIVIDER}{items}", file_time(f.modified, true, now, zone))
        }
        FileKind::Folder { items: None } => l.pick(["Unreadable", "Не читается"]).to_string(),
        FileKind::File => format!("{}{DIVIDER}{}", file_time(f.modified, false, now, zone), file_size(f.size)),
        FileKind::Link { target, .. } => tr!(l, "Link to {target}", "Ссылка на {target}"),
        FileKind::Special => {
            let what = l.pick(match f.mode & 0o170_000 {
                0o020_000 => ["Character device", "Символьное устройство"],
                0o060_000 => ["Block device", "Блочное устройство"],
                0o010_000 => ["Pipe", "Канал"],
                _ => ["Socket", "Сокет"],
            });
            format!("{}{DIVIDER}{what}", file_time(f.modified, false, now, zone))
        }
    }
}

/// The parts a backup holds: `Boot partitions, system partitions`.
pub fn parts(l: Lang, held: &[bool; 4]) -> String {
    list(l, BACKUP_PARTS.iter().zip(held).filter(|(_, &on)| on).map(|(name, _)| l.pick(*name)))
}

/// What an image is, and what it holds.
pub fn image_kind(l: Lang, kind: ImageKind) -> (&'static str, &'static str) {
    const RECOVERY: Text = ["Recovery image", "Образ recovery"];
    const BOOT: Text = ["Boot image", "Загрузочный образ"];
    let (name, holds) = match kind {
        ImageKind::Boot { kernel: true, ramdisk: Ramdisk::Recovery } => {
            (RECOVERY, ["Holds a kernel and a recovery ramdisk", "Содержит ядро и ramdisk восстановления"])
        }
        ImageKind::Boot { kernel: false, ramdisk: Ramdisk::Recovery } => {
            (RECOVERY, ["Holds a recovery ramdisk alone", "Содержит только ramdisk восстановления"])
        }
        ImageKind::Boot { kernel: true, ramdisk: Ramdisk::Generic } => {
            (BOOT, ["Holds a kernel and a generic ramdisk", "Содержит ядро и универсальный ramdisk"])
        }
        ImageKind::Boot { kernel: true, ramdisk: Ramdisk::None } => (BOOT, ["Holds a kernel alone", "Содержит только ядро"]),
        ImageKind::Boot { kernel: false, .. } => (BOOT, ["Holds a generic ramdisk alone", "Содержит только универсальный ramdisk"]),
        ImageKind::VendorBoot => {
            (["vendor_boot image", "образ vendor_boot"], ["Holds the vendor ramdisk and device tree", "Содержит ramdisk вендора и дерево устройств"])
        }
        ImageKind::Dtbo => (["dtbo image", "образ dtbo"], ["Holds device tree overlays", "Содержит оверлеи дерева устройств"]),
        ImageKind::Vbmeta => (["vbmeta image", "образ vbmeta"], ["Holds verified boot data", "Содержит данные проверенной загрузки"]),
    };
    (l.pick(name), l.pick(holds))
}

/// What a partition an image can go to is for.
pub fn partition_role(l: Lang, name: &str) -> &'static str {
    l.pick(match name {
        "boot" => ["Kernel", "Ядро"],
        "init_boot" => ["Generic ramdisk", "Универсальный ramdisk"],
        "recovery" => ["Recovery", "Recovery"],
        "vendor_boot" => ["Vendor ramdisk and device tree", "Ramdisk вендора и дерево устройств"],
        "dtbo" => ["Device tree overlays", "Оверлеи дерева устройств"],
        "vbmeta" => ["Verified boot data", "Данные Verified Boot"],
        "vbmeta_system" => ["Verification data of the system partitions", "Данные проверки системных разделов"],
        "vbmeta_vendor" => ["Verification data of the vendor partitions", "Данные проверки разделов вендора"],
        _ => ["Partition", "Раздел"],
    })
}

/// `slot B`, `slots A and B`, `слот B`, `слоты A и B`
pub fn slots(l: Lang, slots: &[Slot]) -> String {
    let letters: Vec<&str> = slots.iter().map(|s| s.letter()).collect();
    let joined = letters.join(l.pick([" and ", " и "]));
    match (l, slots.len()) {
        (Lang::En, 1) => format!("slot {joined}"),
        (Lang::En, _) => format!("slots {joined}"),
        (Lang::Ru, 1) => format!("слот {joined}"),
        (Lang::Ru, _) => format!("слоты {joined}"),
    }
}

/// How long the screen stays on with nothing touched: `30 seconds`,
/// `2 minutes`, `Never`.
pub fn screen_off(l: Lang, after_ms: Option<u64>) -> String {
    match after_ms {
        None => l.pick(["Never", "Никогда"]).into(),
        Some(ms) if ms < 60_000 => match (l, ms / 1000) {
            (Lang::En, 1) => "1 second".into(),
            (Lang::En, n) => format!("{n} seconds"),
            (Lang::Ru, n) => format!("{n} {}", ru_plural(n, "секунда", "секунды", "секунд")),
        },
        Some(ms) => match (l, ms / 60_000) {
            (Lang::En, 1) => "1 minute".into(),
            (Lang::En, n) => format!("{n} minutes"),
            (Lang::Ru, n) => format!("{n} {}", ru_plural(n, "минута", "минуты", "минут")),
        },
    }
}

/// A mount or unmount that failed, with why: `Couldn't mount vendor: …`
pub fn mount_failure(l: Lang, m: &MountFailure) -> String {
    let (base, detail) = (&m.base, &m.detail);
    match m.mounting {
        true => tr!(l, "Couldn't mount {base}: {detail}", "Не удалось смонтировать {base}: {detail}"),
        false => tr!(l, "Couldn't unmount {base}: {detail}", "Не удалось отмонтировать {base}: {detail}"),
    }
}

/// How another package is given to the installer from where this one came
/// from: the computer sends it, or another file is chosen, as one is in
/// place of a package the system named. English starts its sentence with it.
fn give(l: Lang, source: Source) -> &'static str {
    l.pick(match source {
        Source::Computer | Source::Rescue => ["Send", "Отправьте"],
        Source::File(_) | Source::Opened { .. } | Source::Asked { .. } => ["Choose", "Выберите"],
    })
}

/// Why a package from `source` was not installed, for a phone with this
/// codename, its dates in the zone at `zone` in [`TIME_ZONES`].
pub fn refusal(l: Lang, r: &Refusal, codename: &str, source: Source, zone: usize) -> String {
    let give = give(l, source);
    match r {
        Refusal::Unknown => tr!(
            l,
            "This file isn't recognized. {give} an A/B full OTA package, a flashable zip or a partition image.",
            "Этот файл не распознан. {give} полный пакет OTA A/B, zip-архив для прошивки или образ раздела.",
        ),
        Refusal::Image => tr!(
            l,
            "A partition image can't be installed this way. {give} an A/B full OTA package or a flashable zip.",
            "Образ раздела нельзя установить так. {give} полный пакет OTA A/B или zip-архив для прошивки.",
        ),
        Refusal::NotAb(kind) if kind.is_empty() => tr!(
            l,
            "This package doesn't say its type. {give} an A/B full OTA package.",
            "В этом пакете не указан его тип. {give} полный пакет OTA A/B.",
        ),
        Refusal::NotAb(kind) => tr!(l, "This package's type is {kind}. {give} an A/B full OTA package.", "Тип этого пакета — {kind}. {give} полный пакет OTA A/B."),
        Refusal::OtherDevice(devices) => tr!(
            l,
            "This package is for {devices}, and this phone is {codename}.",
            "Этот пакет предназначен для {devices}, а этот телефон — {codename}.",
        ),
        Refusal::OtherSerial => l
            .pick([
                "This package is only for devices with certain serial numbers, and this phone isn't one of them.",
                "Этот пакет только для устройств с определёнными серийными номерами, а этот телефон к ним не относится.",
            ])
            .into(),
        Refusal::Incremental => tr!(
            l,
            "An incremental OTA package can't be installed. {give} a full OTA package.",
            "Инкрементальный пакет OTA нельзя установить. {give} полный пакет OTA.",
        ),
        Refusal::Older { package, installed } => {
            let (package, installed) = (date(l, *package, zone), date(l, *installed, zone));
            tr!(
                l,
                "This package was built on {package}, before the current system, built on {installed}.",
                "Этот пакет собран на {package} — раньше текущей системы, собранной на {installed}.",
            )
        }
        Refusal::OlderPatch { package, installed } => tr!(
            l,
            "This package's security patch is {package}, older than the current system's {installed}.",
            "Патч безопасности этого пакета — {package}, старше, чем {installed} у текущей системы.",
        ),
        Refusal::NoPartition => l.pick(["This phone has no partition this image can be flashed to.", "На этом телефоне нет раздела, в который можно прошить этот образ."]).into(),
        Refusal::Battery { level, charging } => {
            let least = skyward_install::least_charge(*charging);
            tr!(
                l,
                "The battery is at {level}%. Charge it past {least}%, then try again.",
                "Заряд батареи {level}%. Зарядите её выше {least}%, затем повторите попытку.",
            )
        }
        Refusal::Pending(state) => {
            let standing = snapshot(l, state);
            tr!(
                l,
                "{standing}. Reboot to the system to finish the last update, then try again.",
                "{standing}. Перезагрузитесь в систему, чтобы завершить последнее обновление, затем повторите попытку.",
            )
        }
        Refusal::OldHost => l
            .pick([
                "The computer's adb is too old. Update Android SDK Platform-Tools, then try again.",
                "На компьютере слишком старая версия adb. Обновите Android SDK Platform-Tools, затем повторите попытку.",
            ])
            .into(),
        Refusal::Unsigned => l
            .pick([
                "This zip isn't signed. Turn off “Verify zip signatures”, then try again.",
                "Этот zip-архив не подписан. Отключите «Проверять подписи zip», затем повторите попытку.",
            ])
            .into(),
        Refusal::Unverified => l
            .pick([
                "This zip's signature didn't verify. Turn off “Verify zip signatures”, then try again.",
                "Подпись этого zip-архива не прошла проверку. Отключите «Проверять подписи zip», затем повторите попытку.",
            ])
            .into(),
        Refusal::Panicked => l
            .pick([
                "This update can't be installed now. Reboot to the system, then try again.",
                "Это обновление нельзя установить сейчас. Перезагрузитесь в систему и повторите попытку.",
            ])
            .into(),
        Refusal::SystemAsked => l
            .pick([
                "The system asked for another system to be installed, which only goes in by hand on this phone.",
                "Система запросила установку другой системы, которая на этом телефоне ставится только вручную.",
            ])
            .into(),
        Refusal::DataClosed => l
            .pick([
                "The other system goes into the data partition. Decrypt it first.",
                "Другая система размещается в разделе data. Сначала расшифруйте его.",
            ])
            .into(),
        Refusal::SystemInstalled(name) => tr!(
            l,
            "{name} is installed already. Remove it in Dual boot, then try again.",
            "{name} уже установлена. Удалите её в разделе «Вторая система», затем повторите попытку.",
        ),
        Refusal::NoRoom { least, room } => {
            let (least, room) = (bytes(*least), bytes(*room));
            tr!(
                l,
                "This system needs at least {least}, and the data partition has {room} to spare.",
                "Этой системе нужно не менее {least}, а в разделе data свободно {room}.",
            )
        }
        Refusal::NotBrick(_) | Refusal::NoSerial => unreachable!("only a request to erase partitions is refused as {r:?}"),
    }
}

/// Why the system's request to erase the partitions it lists was not
/// carried out, on a phone whose codename is `codename`.
pub fn erase_request_refusal(l: Lang, r: &Refusal, codename: &str) -> String {
    match r {
        Refusal::Unsigned => l.pick(["The request isn't signed.", "Запрос не подписан."]).into(),
        Refusal::Unverified => l.pick(["The request's signature didn't verify.", "Подпись запроса не подтвердилась."]).into(),
        Refusal::NotBrick(kind) if kind.is_empty() => l.pick(["The request doesn't say its type.", "В запросе не указан его тип."]).into(),
        Refusal::NotBrick(kind) => tr!(l, "The request's type is {kind}.", "Тип запроса — {kind}."),
        Refusal::OtherDevice(devices) => {
            tr!(
                l,
                "The request is for {devices}, and this phone is {codename}.",
                "Запрос предназначен для {devices}, а этот телефон — {codename}.",
            )
        }
        Refusal::OtherSerial => l
            .pick([
                "The request is only for devices with certain serial numbers, and this phone isn't one of them.",
                "Запрос предназначен только для устройств с определёнными серийными номерами, и этот телефон не из них.",
            ])
            .into(),
        Refusal::NoSerial => l.pick(["The request names no serial number.", "В запросе не указан серийный номер."]).into(),
        other => unreachable!("a request to erase partitions is not refused as {other:?}"),
    }
}

/// What went wrong installing a package from `source`, and the details
/// behind it when there are any.
pub fn failure(l: Lang, f: &Failure, source: Source) -> (String, Option<String>) {
    let said = |text: Text| l.pick(text).to_string();
    match f {
        Failure::Engine(code) => {
            let what = match (Trouble::of(*code), source) {
                (Trouble::Signature, _) => {
                    let give = give(l, source);
                    tr!(
                        l,
                        "The package's signature didn't verify. {give} an official package.",
                        "Подпись пакета не подтвердилась. {give} официальный пакет.",
                    )
                }
                (Trouble::Damaged, _) => said(["The package is damaged. Download it again.", "Пакет повреждён. Загрузите его снова."]),
                (Trouble::Older, _) => said(["The package is too old.", "Пакет слишком старый."]),
                (Trouble::NoSpace, _) => said(["Not enough space in the super partition.", "Недостаточно места в разделе super."]),
                (Trouble::Transfer, Source::Computer | Source::Rescue) => said([
                    "Couldn't read the package. Check the cable, then send it again.",
                    "Не удалось прочитать пакет. Проверьте кабель и отправьте его снова.",
                ]),
                (Trouble::Transfer, Source::File(Place::Storage) | Source::Opened { drive: false } | Source::Asked { .. }) => {
                    said(["Couldn't read the package.", "Не удалось прочитать пакет."])
                }
                (Trouble::Transfer, Source::File(Place::UsbDrive) | Source::Opened { drive: true }) => said([
                    "Couldn't read the package. Check the USB drive, then try again.",
                    "Не удалось прочитать пакет. Проверьте USB-накопитель и повторите попытку.",
                ]),
                (Trouble::Write, _) => said(["Couldn't write a partition.", "Не удалось записать раздел."]),
                (Trouble::Verify, _) => said(["A partition didn't verify.", "Раздел не прошёл проверку."]),
                (Trouble::PostInstall, _) => said(["Couldn't finish installing.", "Не удалось завершить установку."]),
                (Trouble::Unsupported, _) => said(["This package's format isn't supported.", "Формат этого пакета не поддерживается."]),
                (Trouble::Other, _) => said(["The updater ran into an error.", "В программе обновления произошла ошибка."]),
            };
            (what, Some(tr!(l, "Error code: {code}", "Код ошибки: {code}")))
        }
        Failure::Script(code) => {
            (tr!(l, "The install script failed with exit code {code}.", "Скрипт установки завершился с кодом {code}."), None)
        }
        Failure::Disconnected => (
            said([
                "The computer stopped sending. Check the cable, then send it again.",
                "Компьютер перестал передавать. Проверьте кабель и отправьте снова.",
            ]),
            None,
        ),
        Failure::Unreadable(why) => match source {
            Source::File(Place::UsbDrive) | Source::Opened { drive: true } => (
                said([
                    "Couldn't read the file. Check the USB drive, then try again.",
                    "Не удалось прочитать файл. Проверьте USB-накопитель и повторите попытку.",
                ]),
                Some(why.clone()),
            ),
            _ => (said(["Couldn't read the file.", "Не удалось прочитать файл."]), Some(why.clone())),
        },
        Failure::Changed => match source {
            Source::Computer | Source::Rescue => {
                (said(["The file changed while it was sent. Send it again.", "Файл изменился во время передачи. Отправьте его снова."]), None)
            }
            Source::File(_) | Source::Opened { .. } | Source::Asked { .. } => {
                (said(["The file changed while it was read. Choose it again.", "Файл изменился во время чтения. Выберите его снова."]), None)
            }
        },
        Failure::Error(detail) => (said(UNEXPECTED), Some(detail.clone())),
    }
}

/// How many of a WLAN fan's three arcs a signal of `dbm` lights.
pub fn signal_level(dbm: i32) -> u8 {
    match dbm {
        d if d >= -60 => 3,
        d if d >= -70 => 2,
        d if d >= -80 => 1,
        _ => 0,
    }
}

/// How fast data goes, `bytes` a second, as the stock status bar writes it
/// over its unit: in KB/s up to 999 and in MB/s past that, to three
/// figures, two of them after the point below 10 and one below 100.
pub fn speed(bytes: u64) -> (String, &'static str) {
    let kb = bytes as f64 / 1024.0;
    let (v, unit) = if kb < 999.5 { (kb, "KB/s") } else { (kb / 1024.0, "MB/s") };
    let figures = if v < 9.995 {
        format!("{v:.2}")
    } else if v < 99.95 {
        format!("{v:.1}")
    } else {
        format!("{v:.0}")
    };
    (figures, unit)
}

/// A signal's strength in words, as the stock network details say it.
pub fn signal(l: Lang, dbm: i32) -> &'static str {
    l.pick(match signal_level(dbm) {
        3 => ["Excellent", "Отлично"],
        2 => ["Good", "Хорошо"],
        1 => ["Fair", "Удовлетворительно"],
        _ => ["Weak", "Слабый"],
    })
}

/// The bands a network is on, as the tag after its name: none for 2.4 GHz
/// alone, as the stock WLAN page shows it.
pub fn band_tag(b: Bands) -> Option<String> {
    let names: Vec<&str> = [(b.ghz2, "2.4G"), (b.ghz5, "5G"), (b.ghz6, "6G")].into_iter().filter_map(|(on, n)| on.then_some(n)).collect();
    (names != ["2.4G"] && !names.is_empty()).then(|| names.join("/"))
}

/// The bands a network is on, in full.
pub fn bands(l: Lang, b: Bands) -> String {
    list(l, [(b.ghz2, "2.4 GHz"), (b.ghz5, "5 GHz"), (b.ghz6, "6 GHz")].into_iter().filter_map(|(on, n)| on.then_some(n)))
}

/// A generation of Wi-Fi, as the stock network details say it.
pub fn generation(l: Lang, n: u8) -> String {
    tr!(l, "Wi-Fi {n}", "Wi-Fi {n}")
}

/// How a network is protected, as the system's network details name it.
pub fn security(l: Lang, s: Security) -> &'static str {
    match s {
        Security::Open => l.pick(["None", "Нет"]),
        Security::Owe => l.pick(["Enhanced Open", "Enhanced Open"]),
        Security::Wep => "WEP",
        Security::Psk => "WPA/WPA2-Personal",
        Security::Sae => "WPA3-Personal",
        Security::PskSae => "WPA2/WPA3-Personal",
        Security::Enterprise => "WPA/WPA2-Enterprise",
    }
}

/// What the home page says beside its WLAN entry: the network joined, or
/// how the WLAN stands.
pub fn wlan_value(l: Lang, w: &Wlan) -> &str {
    match (&w.power, &w.joined) {
        (Power::Off | Power::Failed(_), _) => l.pick(["Off", "Выкл."]),
        (Power::Starting, _) => l.pick(["Turning on", "Включение"]),
        (Power::On, Some(Joined { stage: WlanStage::Connected(_), name, .. })) => name,
        (Power::On, Some(_)) => l.pick(["Connecting", "Подключение"]),
        (Power::On, None) => l.pick(["Not connected", "Не подключено"]),
    }
}

/// How joining the network chosen goes, under its name; joined, how far it
/// reaches, as the system's WLAN page says it.
pub fn joining(l: Lang, stage: &WlanStage) -> &'static str {
    l.pick(match stage {
        WlanStage::Joining => ["Connecting", "Подключение"],
        WlanStage::Addressing => ["Getting an IP address", "Получение IP-адреса"],
        WlanStage::Connected(link) => match link.reach {
            Some(Reach::Login(_)) => ["Sign-in required", "Требуется вход"],
            Some(Reach::Offline(_)) => ["Connected, no internet", "Подключено, без интернета"],
            Some(Reach::Online) | None => ["Connected", "Подключено"],
        },
        WlanStage::WrongPassword => ["Wrong password", "Неверный пароль"],
        WlanStage::NotFound => ["Network not found", "Сеть не найдена"],
        WlanStage::Failed => ["Couldn't connect", "Не удалось подключиться"],
        WlanStage::NoAddress(_) => ["Couldn't get an IP address", "Не удалось получить IP-адрес"],
    })
}

/// What a network that wants a login in a browser first leaves to do,
/// with a recovery that has no browser to log in with.
const LOGIN: Text = [
    "needs a sign-in in a browser before it reaches the internet, and Skyward has no browser to sign in with. Use a network that needs no sign-in instead, such as a phone's hotspot.",
    "требует входа в браузере, прежде чем получит доступ к интернету, а в Skyward нет браузера для входа. Используйте сеть, не требующую входа, например точку доступа телефона.",
];

/// What the WLAN page says under the network joined of how far it reaches,
/// where it reaches short of the internet.
pub fn reach(l: Lang, reach: &Reach) -> Option<String> {
    let login = l.pick(LOGIN);
    match reach {
        Reach::Online => None,
        Reach::Login(_) => Some(tr!(l, "This network {login}", "Эта сеть {login}")),
        Reach::Offline(why) => Some(tr!(l, "This network doesn't reach the internet: {why}", "Эта сеть не имеет доступа в интернет: {why}")),
    }
}

/// Why the update server gave no answer, as the update page says it.
pub fn unanswered(l: Lang, why: &Unanswered) -> String {
    let login = l.pick(LOGIN);
    match why {
        Unanswered::Login => tr!(
            l,
            "Couldn't check for updates. The Wi-Fi network joined {login}",
            "Не удалось проверить обновления. Сеть Wi-Fi, к которой выполнено подключение, {login}",
        ),
        Unanswered::Offline(why) => tr!(
            l,
            "Couldn't check for updates. The Wi-Fi network joined doesn't reach the internet: {why}",
            "Не удалось проверить обновления. Сеть Wi-Fi, к которой выполнено подключение, не имеет доступа к интернету: {why}",
        ),
        Unanswered::Failed(why) => tr!(l, "Couldn't check for updates: {why}", "Не удалось проверить обновления: {why}"),
    }
}

/// A size the update server gives, such as 7.1G, with its unit spelled
/// out; one in another form shows as it came.
pub fn package_size(size: &str) -> String {
    match size.char_indices().last() {
        Some((at, unit @ ('K' | 'M' | 'G' | 'T'))) if size[..at].parse::<f64>().is_ok() => format!("{} {unit}B", &size[..at]),
        _ => size.to_string(),
    }
}

/// Why downloading the full package did not end with it whole.
pub fn download_failure(l: Lang, f: &DownloadFailure) -> String {
    match f {
        DownloadFailure::NoRoom { needed, free } => {
            let (needed, free) = (file_size(*needed), file_size(*free));
            tr!(
                l,
                "Not enough internal storage: {needed} has to be free, and {free} is.",
                "Недостаточно внутренней памяти: свободными должны быть {needed}, а свободно {free}.",
            )
        }
        DownloadFailure::Damaged => l
            .pick([
                "The full OTA package didn't verify and was deleted. Download it again.",
                "Полный пакет OTA не прошёл проверку и был удалён. Загрузите его снова.",
            ])
            .into(),
        DownloadFailure::Changed => {
            l.pick([
                "The server has a newer version now. Check for updates again.",
                "На сервере теперь более новая версия. Проверьте обновления снова.",
            ]).into()
        }
        DownloadFailure::Other(why) => tr!(l, "Download failed: {why}", "Не удалось загрузить: {why}"),
    }
}

/// Why downloading Skyward's build did not end with it whole. It goes into
/// the recovery's own folder, which is in memory.
pub fn skyward_download_failure(l: Lang, f: &DownloadFailure) -> String {
    match f {
        DownloadFailure::NoRoom { needed, free } => {
            let (needed, free) = (file_size(*needed), file_size(*free));
            tr!(
                l,
                "Not enough memory: {needed} has to be free, and {free} is.",
                "Недостаточно памяти: свободными должны быть {needed}, а свободно {free}.",
            )
        }
        DownloadFailure::Damaged => {
            l.pick(["The package didn't verify and was deleted. Update again.", "Пакет не прошёл проверку и был удалён. Обновитесь снова."]).into()
        }
        DownloadFailure::Changed => unreachable!("GitHub is not asked again where a build is"),
        DownloadFailure::Other(why) => tr!(l, "Download failed: {why}", "Не удалось загрузить: {why}"),
    }
}

/// How the terminal page's shell ended.
pub fn term_end(l: Lang, end: &TermEnd) -> String {
    match end {
        TermEnd::Exited(code) => tr!(l, "The shell exited with code {code}", "Оболочка завершилась с кодом {code}"),
        TermEnd::Signalled(signal) => tr!(l, "Signal {signal} stopped the shell", "Сигнал {signal} остановил оболочку"),
        TermEnd::Other(status) => tr!(l, "The shell ended: {status}", "Оболочка завершилась: {status}"),
    }
}

/// What a computer is told internal storage is called, as the stock system
/// names it.
pub fn storage_name(l: Lang) -> &'static str {
    l.pick(["Internal shared storage", "Внутренняя общая память"])
}

/// File transfer kept internal storage from work that needed it back, as
/// the system says why.
pub fn transfer_kept(l: Lang, why: &str) -> String {
    tr!(l, "Couldn't stop file transfer: {why}", "Не удалось остановить передачу файлов: {why}")
}

/// File transfer did not turn on, as the system says why.
pub fn transfer_failed(l: Lang, why: &str) -> String {
    tr!(l, "Couldn't turn on file transfer: {why}", "Не удалось включить передачу файлов: {why}")
}

/// File transfer did not turn off, as the system says why.
pub fn transfer_left_on(l: Lang, why: &str) -> String {
    tr!(l, "Couldn't turn off file transfer: {why}", "Не удалось выключить передачу файлов: {why}")
}

/// The server for the files did not start, as the system says why.
pub fn wireless_failed(l: Lang, why: &str) -> String {
    tr!(l, "Couldn't start wireless transfer: {why}", "Не удалось запустить беспроводную передачу: {why}")
}

/// File transfer stopped on its own, as the system says why.
pub fn transfer_stopped(l: Lang, why: &str) -> String {
    tr!(l, "File transfer stopped: {why}", "Передача файлов остановлена: {why}")
}

/// Merging the update failed, as the system says why.
pub fn merge_failed(l: Lang, why: &str) -> String {
    tr!(l, "Couldn't merge the update: {why}", "Не удалось объединить обновление: {why}")
}

/// What work that takes a program of the installed system's `base`
/// partition asks while that partition is not mounted.
pub fn mount_first(l: Lang, base: &str) -> String {
    tr!(l, "Mount the {base} partition in Mount first", "Сначала смонтируйте раздел {base} на экране «Монтирование»")
}

/// The installed system's SELinux rules could not be read, as the system
/// says why.
pub fn rules_unread(l: Lang, why: &str) -> String {
    tr!(l, "Couldn't read the system's SELinux rules: {why}", "Не удалось прочитать правила SELinux системы: {why}")
}

/// The logs were saved in `folder`.
pub fn exported(l: Lang, folder: &str) -> String {
    tr!(l, "Exported to {folder}", "Экспортировано в {folder}")
}

/// The logs were not saved, as the system says why.
pub fn export_failed(l: Lang, why: &str) -> String {
    tr!(l, "Couldn't export: {why}", "Не удалось экспортировать: {why}")
}

/// A screenshot went into the gallery's screenshots.
pub const SCREENSHOT_SAVED: Text = ["Screenshot saved to the gallery", "Снимок экрана сохранён в галерее"];

/// A screenshot is held until the owner's storage opens, and then goes into
/// the gallery's.
pub const SCREENSHOT_HELD: Text = ["Screenshot kept. It goes to the gallery once decrypted", "Снимок экрана сохранён. Он появится в галерее после расшифровки"];

/// A screenshot was not saved, as the system says why.
pub fn screenshot_failed(l: Lang, why: &str) -> String {
    tr!(l, "Couldn't save the screenshot: {why}", "Не удалось сохранить снимок экрана: {why}")
}

/// The screenshots held while the owner's storage was closed did not go into
/// the gallery's, as the system says why.
pub fn screenshots_not_kept(l: Lang, why: &str) -> String {
    tr!(l, "Kept screenshots couldn't go to the gallery: {why}", "Сохранённые скриншоты не попали в галерею: {why}")
}

/// The name a screenshot taken `unix_ms` milliseconds into Unix time is saved
/// under, in the zone at `zone` in [`TIME_ZONES`]: as the system names its
/// own, the time to the millisecond and then where it was taken,
/// `Screenshot_2026-10-26-01-45-00-123_Skyward.png`.
pub fn screenshot_name(unix_ms: i64, zone: usize) -> String {
    let t = local(unix_ms.div_euclid(1000), zone);
    format!(
        "Screenshot_{:04}-{:02}-{:02}-{:02}-{:02}-{:02}-{:03}_Skyward.png",
        t.year(),
        t.month(),
        t.day(),
        t.hour(),
        t.minute(),
        t.second(),
        unix_ms.rem_euclid(1000)
    )
}

/// What the terminal is told while a command it runs holds it.
pub const TERMINAL_BUSY: Text = ["The terminal is running a command. End it, then try again", "В терминале выполняется команда. Завершите её и повторите попытку"];

#[cfg(test)]
mod tests {
    use super::*;

    const RU: Lang = Lang::Ru;

    // Xiaomi 13 Ultra (ishtar, 16 GB + 512 GB): readings from `skyward info`
    // and the strings its stock About pages print for them.
    // Xiaomi 17 Ultra (nezha, 16 GB + 1 TB): MemTotal from its boot log and
    // the userdata size from `fastboot getvar all`.
    #[test]
    fn ram_rounds_to_module_size() {
        assert_eq!(ram(15_556_972), "16.0 GB");
        assert_eq!(ram(15_472_464), "16.0 GB");
    }

    #[test]
    fn storage_matches_the_stock_settings() {
        let ishtar = Storage { total: 496_550_785_024, free: Some(61_300_000_000) };
        assert_eq!(capacity(ishtar.total), "512GB");
        assert_eq!(storage_used(&ishtar).unwrap(), "450.7GB/512GB");
        assert_eq!(capacity(0xF2_091F_B000), "1TB");
        assert_eq!(storage_used(&Storage { total: 1, free: None }), None);
    }

    #[test]
    fn cpu_summary() {
        let nezha = Cpu { cores: 8, max_khz: Some(4_608_000) };
        assert_eq!(cpu(RU, &Cpu { cores: 8, max_khz: Some(3_187_200) }), "Восьмиядерный, до 3.19GHz");
        assert_eq!(cpu(RU, &nezha), "Восьмиядерный, до 4.61GHz");
        assert_eq!(cpu(RU, &Cpu { cores: 2, max_khz: None }), "Двухъядерный");
        assert_eq!(cpu(RU, &Cpu { cores: 12, max_khz: None }), "12-ядерный");
        assert_eq!(cpu(Lang::En, &nezha), "Octa-core, up to 4.61GHz");
        assert_eq!(cpu(Lang::En, &Cpu { cores: 12, max_khz: None }), "12-core");
        assert_eq!(ru_plural(1, "секунда", "секунды", "секунд"), "секунда");
        assert_eq!(ru_plural(21, "секунда", "секунды", "секунд"), "секунда");
        assert_eq!(ru_plural(12, "секунда", "секунды", "секунд"), "секунд");
    }

    // Xiaomi 13 Ultra battery as `skyward info` read it.
    #[test]
    fn battery_lines() {
        let b = Battery {
            level: 100,
            charge: Charge::Full,
            temp: Some(338),
            health: Some("Good".into()),
            cycles: Some(169),
            design: Some(5_500_000),
            full: Some(5_095_000),
            voltage: Some(4_482_000),
            quick: false,
            plugged: true,
        };
        assert_eq!(level(RU, &b), "100% · Заряжен");
        assert_eq!(level(Lang::En, &b), "100% · Full");
        assert_eq!(celsius(b.temp.unwrap()), "33.8°C");
        assert_eq!([degrees(352), degrees(400), degrees(-5)], ["35.2", "40.0", "-0.5"], "to the tenth, a whole degree too");
        assert_eq!(wear(RU, &b).unwrap(), "Хорошо · Максимальная ёмкость 93%");
        assert_eq!(wear(Lang::En, &b).unwrap(), "Good · Maximum capacity 93%");
        assert_eq!(mah(b.design.unwrap()), "5500mAh");
        assert_eq!(volts(b.voltage.unwrap()), "4.48V");
    }

    // Xiaomi 17 Ultra build and panel, as its stock settings print them.
    #[test]
    fn versions_and_states() {
        let patch = PatchLevel { year: 2026, month: 10, day: 1 };
        assert_eq!(android(RU, "17", Some(&patch)), "17 | Обновление безопасности Android: 1 октября 2026");
        assert_eq!(android(Lang::En, "17", Some(&patch)), "17 | Android security update: October 1, 2026");
        assert_eq!(panel((1200, 2608)), "2608 x 1200");
        assert_eq!(model_lines("Xiaomi 17 Ultra by Leica"), "Xiaomi 17 Ultra\nby Leica");
        assert_eq!(model_lines("Xiaomi 17 Ultra"), "Xiaomi 17 Ultra");
        assert_eq!(inches(6.9), "6.9″");
        assert_eq!(verified(RU, "orange"), "Оранжевый · Разблокировано, поэтому система не проверена");
        assert_eq!(health(Lang::En, "Watchdog timer expire"), "Watchdog timer expire");
    }

    #[test]
    fn sizes_and_dates() {
        assert_eq!(bytes(7_103_041_280), "7.1GB");
        assert_eq!(bytes(100_663_296), "100.7MB");
        assert_eq!(bytes(131_072), "131KB");
        assert_eq!(bytes(64), "64B");
        // The Xiaomi 13 Ultra's installed system, by its ro.build.date.utc.
        assert_eq!(date(RU, 1_778_243_655, zone("Europe/Moscow")), "8 мая 2026");
        assert_eq!(date(Lang::En, 1_778_243_655, zone("Europe/Moscow")), "May 8, 2026");
        assert_eq!(date(RU, 0, zone("UTC")), "1 января 1970");
        // Midnight on a leap day in UTC is three in the morning in Moscow.
        assert_eq!(date(RU, 1_709_164_800, zone("Europe/Moscow")), "29 февраля 2024");
        assert_eq!(date(RU, 1_709_164_800 - 3 * 3600 - 1, zone("Europe/Moscow")), "28 февраля 2024");
        assert_eq!(patch(RU, &PatchLevel { year: 2026, month: 4, day: 1 }), "1 апреля 2026");
        assert_eq!(slots(RU, &[Slot::B]), "слот B");
        assert_eq!(slots(RU, &[Slot::A, Slot::B]), "слоты A и B");
        assert_eq!(slots(Lang::En, &[Slot::B]), "slot B");
        assert_eq!(slots(Lang::En, &[Slot::A, Slot::B]), "slots A and B");
    }

    #[test]
    fn screen_off_times() {
        let labels = |l| crate::app::SCREEN_OFF.iter().map(|&ms| screen_off(l, ms)).collect::<Vec<String>>();
        assert_eq!(labels(RU), ["15 секунд", "30 секунд", "1 минута", "2 минуты", "5 минут", "10 минут", "Никогда"]);
        assert_eq!(labels(Lang::En), ["15 seconds", "30 seconds", "1 minute", "2 minutes", "5 minutes", "10 minutes", "Never"]);
    }

    #[test]
    fn times_to_the_minute_in_moscow() {
        // 2026-10-18 18:31 UTC is 21:31 the same evening in Moscow.
        assert_eq!(datetime(RU, 1_792_348_260, zone("Europe/Moscow")), "18 октября 2026, 21:31");
        assert_eq!(datetime(Lang::En, 1_792_348_260, zone("Europe/Moscow")), "October 18, 2026, 21:31");
        assert_eq!(clock(1_792_348_260, zone("Europe/Moscow")), "21:31");
        // Midnight in Moscow is 21:00 UTC the day before.
        assert_eq!(clock(1_792_357_200, zone("Europe/Moscow")), "00:00");
        assert_eq!(stamp(1_792_348_265, zone("Europe/Moscow")), "20261018-213105");
    }

    #[test]
    fn a_screenshot_is_named_as_the_system_names_its_own() {
        assert_eq!(screenshot_name(1_792_348_265_007, zone("Europe/Moscow")), "Screenshot_2026-10-18-21-31-05-007_Skyward.png");
        assert_eq!(screenshot_name(1_792_348_265_999, zone("UTC")), "Screenshot_2026-10-18-18-31-05-999_Skyward.png", "in the zone chosen");
    }

    /// Where a zone named in the tz database is in [`TIME_ZONES`].
    fn zone(name: &str) -> usize {
        TIME_ZONES.iter().position(|&(_, _, n)| n == name).expect("a zone listed")
    }

    #[test]
    fn every_zone_listed_has_its_rules() {
        for at in 0..TIME_ZONES.len() {
            let _ = rules(at);
        }
    }

    #[test]
    fn zones_keep_their_own_summer_time() {
        // 2026-07-01 12:00 UTC, and 2026-01-15 12:00 UTC.
        let (summer, winter) = (1_782_907_200, 1_768_478_400);
        assert_eq!(clock(summer, zone("UTC")), "12:00");
        assert_eq!(clock(summer, zone("Europe/London")), "13:00", "British Summer Time");
        assert_eq!(clock(winter, zone("Europe/London")), "12:00");
        assert_eq!(clock(summer, zone("Europe/Berlin")), "14:00");
        assert_eq!(clock(winter, zone("Europe/Berlin")), "13:00");
        assert_eq!(clock(summer, zone("Europe/Moscow")), "15:00", "Moscow keeps no summer time");
        assert_eq!(clock(winter, zone("Europe/Moscow")), "15:00");
        assert_eq!(clock(summer, zone("America/New_York")), "08:00");
        assert_eq!(clock(winter, zone("America/New_York")), "07:00");
        assert_eq!(datetime(RU, summer, zone("America/Los_Angeles")), "1 июля 2026, 05:00");
        assert_eq!(clock(summer, zone("Asia/Tokyo")), "21:00");
        assert_eq!(clock(winter, zone("Asia/Shanghai")), "20:00", "China keeps no summer time");
        // New York goes over to summer time at 2:00 on 2026-03-08.
        let before = 1_772_953_200 - 60;
        assert_eq!((clock(before, zone("America/New_York")), clock(before + 60, zone("America/New_York"))), ("01:59".into(), "03:00".into()));
    }

    #[test]
    fn a_time_past_the_calendar_shows_as_its_end() {
        assert_eq!(date(RU, i64::MAX, zone("Europe/Moscow")), "31 декабря 9999");
        assert_eq!(date(RU, i64::MIN, zone("UTC")), "2 января -9999");
        assert_eq!(date(Lang::En, i64::MAX, zone("Europe/Moscow")), "December 31, 9999");
    }

    #[test]
    fn modes_read_as_ls_prints_them() {
        assert_eq!(mode(0o040_755), "drwxr-xr-x");
        assert_eq!(mode(0o100_644), "-rw-r--r--");
        assert_eq!(mode(0o120_777), "lrwxrwxrwx");
        assert_eq!(mode(0o041_777), "drwxrwxrwt", "the sticky /tmp");
        assert_eq!(mode(0o104_755), "-rwsr-xr-x", "set-user on an executable");
        assert_eq!(mode(0o020_600), "crw-------");
    }

    #[test]
    fn entries_say_what_they_are() {
        // Changed at 2026-10-18 21:31 in Moscow; listed a week later.
        let row = |kind, size, mode| FileRow { name: "x".into(), kind, size, modified: 1_792_348_260, mode, uid: 0, gid: 0 };
        let later = 1_792_348_260 + 7 * 86_400;
        let moscow = zone("Europe/Moscow");
        assert_eq!(file_line(RU, &row(FileKind::Folder { items: Some(6) }, 4096, 0o040_755), later, moscow), "2026/10/18  |  6 элементов");
        assert_eq!(file_line(RU, &row(FileKind::Folder { items: Some(1) }, 4096, 0o040_755), later, moscow), "2026/10/18  |  1 элемент");
        assert_eq!(file_line(Lang::En, &row(FileKind::Folder { items: Some(6) }, 4096, 0o040_755), later, moscow), "2026/10/18  |  6 items");
        assert_eq!(file_line(Lang::En, &row(FileKind::Folder { items: Some(1) }, 4096, 0o040_755), later, moscow), "2026/10/18  |  1 item");
        assert_eq!(file_line(RU, &row(FileKind::Folder { items: None }, 4096, 0o040_700), later, moscow), "Не читается");
        assert_eq!(file_line(RU, &row(FileKind::File, 26_950, 0o100_644), later, moscow), "10/18  |  26.95 KB");
        let link = FileKind::Link { target: "/storage/self/primary".into(), folder: true };
        assert_eq!(file_line(RU, &row(link, 21, 0o120_777), later, moscow), "Ссылка на /storage/self/primary");
        assert_eq!(file_line(RU, &row(FileKind::Special, 0, 0o060_600), later, moscow), "10/18  |  Блочное устройство");
        assert_eq!(file_line(Lang::En, &row(FileKind::Special, 0, 0o060_600), later, moscow), "10/18  |  Block device");
    }

    #[test]
    fn list_dates_are_told_against_the_day_the_folder_was_read() {
        // 2026-10-18 21:31 in Moscow.
        let at = 1_792_348_260;
        let moscow = zone("Europe/Moscow");
        assert_eq!(file_time(at, false, at + 3600, moscow), "21:31", "today: the time");
        assert_eq!(file_time(at, true, at + 3600, moscow), "21:31", "a folder today too");
        assert_eq!(file_time(at, false, at + 86_400, moscow), "10/18", "this year: a file's month and day");
        assert_eq!(file_time(at, true, at + 86_400, moscow), "2026/10/18", "this year: a folder's whole date");
        assert_eq!(file_time(at, false, at + 90 * 86_400, moscow), "2026/10/18", "another year: the whole date");
        // Two hours on, still the same evening in Moscow.
        assert_eq!(file_time(at + 2 * 3600, false, at, moscow), "23:31");
        // In London the file changed on the evening of the 18th, the day
        // before.
        assert_eq!(file_time(1_792_348_260, false, 1_792_367_940, zone("Europe/London")), "10/18");
    }

    #[test]
    fn sizes_go_up_a_unit_past_900_with_two_decimals_at_most() {
        assert_eq!(file_size(0), "0 B");
        assert_eq!(file_size(900), "900 B");
        assert_eq!(file_size(901), "0.9 KB");
        assert_eq!(file_size(42_633), "42.63 KB");
        assert_eq!(file_size(26_950), "26.95 KB");
        assert_eq!(file_size(1_090_000), "1.09 MB");
        assert_eq!(file_size(634_960), "634.96 KB");
        assert_eq!(file_size(2_000_000_000), "2 GB");
        assert_eq!(file_size(1_500_000), "1.5 MB");
    }

    #[test]
    fn a_backup_lists_its_parts() {
        assert_eq!(parts(RU, &[true, true, false, true]), "Загрузочные разделы, системные разделы, модем и EFS");
        assert_eq!(parts(Lang::En, &[true, true, true, true]), "Boot partitions, system partitions, data partition, modem and EFS");
    }

    #[test]
    fn a_list_names_what_follows_the_first_as_a_sentence_does() {
        assert_eq!(list(Lang::En, ["ART cache", "app data", "logs"]), "ART cache, app data, logs");
        assert_eq!(list(RU, ["Журналы", "Данные"]), "Журналы, данные");
        assert_eq!(bands(Lang::En, Bands { ghz2: true, ghz5: true, ghz6: false }), "2.4 GHz, 5 GHz");
        assert_eq!(bands(RU, Bands { ghz2: true, ghz5: true, ghz6: true }), "2.4 GHz, 5 GHz, 6 GHz");
    }

    #[test]
    fn mount_failures_name_the_partition_and_why() {
        let unmounting = MountFailure {
            mounting: false,
            base: "vendor".into(),
            detail: "unmount /vendor: Device or resource busy (os error 16)".into(),
        };
        assert_eq!(mount_failure(RU, &unmounting), "Не удалось отмонтировать vendor: unmount /vendor: Device or resource busy (os error 16)");
        assert_eq!(mount_failure(Lang::En, &unmounting), "Couldn't unmount vendor: unmount /vendor: Device or resource busy (os error 16)");
        let mounting = MountFailure {
            mounting: true,
            base: "odm".into(),
            detail: "mount odm_b: /dev/block/dm-8 holds none of erofs, ext4, f2fs".into(),
        };
        assert_eq!(mount_failure(RU, &mounting), "Не удалось смонтировать odm: mount odm_b: /dev/block/dm-8 holds none of erofs, ext4, f2fs");
    }

    #[test]
    fn reasons_read_as_sentences() {
        let pc = Source::Computer;
        assert_eq!(refusal(RU, &Refusal::OtherDevice("nezha".into()), "ishtar", pc, 0), "Этот пакет предназначен для nezha, а этот телефон — ishtar.");
        assert_eq!(
            refusal(RU, &Refusal::Older { package: 1_770_000_000, installed: 1_778_243_655 }, "ishtar", pc, 0),
            "Этот пакет собран на 2 февраля 2026 — раньше текущей системы, собранной на 8 мая 2026."
        );
        assert_eq!(
            refusal(Lang::En, &Refusal::Older { package: 1_770_000_000, installed: 1_778_243_655 }, "ishtar", pc, 0),
            "This package was built on February 2, 2026, before the current system, built on May 8, 2026."
        );
        assert_eq!(refusal(RU, &Refusal::Battery { level: 12, charging: true }, "ishtar", pc, 0), "Заряд батареи 12%. Зарядите её выше 15%, затем повторите попытку.");
        assert_eq!(
            refusal(RU, &Refusal::Pending(SnapshotState::Snapshotted), "ishtar", pc, 0),
            "Обновление записано, ожидает объединения. Перезагрузитесь в систему, чтобы завершить последнее обновление, затем повторите попытку."
        );
        assert_eq!(
            refusal(Lang::En, &Refusal::Pending(SnapshotState::Snapshotted), "ishtar", pc, 0),
            "Update written, waiting to merge. Reboot to the system to finish the last update, then try again."
        );
        assert_eq!(failure(RU, &Failure::Engine(26), pc), ("Подпись пакета не подтвердилась. Отправьте официальный пакет.".into(), Some("Код ошибки: 26".into())));
        assert_eq!(failure(RU, &Failure::Error("open ep0: busy".into()), pc).1.as_deref(), Some("open ep0: busy"));
        assert_eq!(failure(RU, &Failure::Script(1), pc), ("Скрипт установки завершился с кодом 1.".into(), None));
        assert_eq!(refusal(RU, &Refusal::Unsigned, "ishtar", pc, 0), "Этот zip-архив не подписан. Отключите «Проверять подписи zip», затем повторите попытку.");
        assert_eq!(
            refusal(RU, &Refusal::Unverified, "ishtar", pc, 0),
            "Подпись этого zip-архива не прошла проверку. Отключите «Проверять подписи zip», затем повторите попытку."
        );
    }

    #[test]
    fn a_file_chosen_is_answered_with_choosing_another() {
        let file = Source::File(Place::Storage);
        assert_eq!(
            refusal(RU, &Refusal::Unknown, "ishtar", file, 0),
            "Этот файл не распознан. Выберите полный пакет OTA A/B, zip-архив для прошивки или образ раздела."
        );
        assert_eq!(
            refusal(Lang::En, &Refusal::Unknown, "ishtar", file, 0),
            "This file isn't recognized. Choose an A/B full OTA package, a flashable zip or a partition image."
        );
        assert_eq!(refusal(RU, &Refusal::Incremental, "ishtar", file, 0), "Инкрементальный пакет OTA нельзя установить. Выберите полный пакет OTA.");
        assert_eq!(failure(RU, &Failure::Engine(26), file).0, "Подпись пакета не подтвердилась. Выберите официальный пакет.");
        assert_eq!(failure(RU, &Failure::Changed, file).0, "Файл изменился во время чтения. Выберите его снова.");
        let gone = "read /sdcard/Download/ota.zip: Input/output error (os error 5)";
        assert_eq!(failure(RU, &Failure::Unreadable(gone.into()), file), ("Не удалось прочитать файл.".into(), Some(gone.into())));
        let pulled = failure(RU, &Failure::Unreadable(gone.into()), Source::File(Place::UsbDrive));
        assert_eq!(pulled.0, "Не удалось прочитать файл. Проверьте USB-накопитель и повторите попытку.", "a drive that came out is checked first");
    }

    #[test]
    fn erasing_that_stops_says_what_was_left_and_why() {
        assert_eq!(users(RU, &[0]), "владелец");
        assert_eq!(users(RU, &[0, 999]), "владелец и пользователь 999");
        assert_eq!(users(RU, &[0, 10, 999]), "владелец, пользователь 10 и пользователь 999");
        assert!(erase_stop(RU, &EraseStop::Locked(vec![0, 999])).0.starts_with("Данные (владелец и пользователь 999) не расшифрованы,"));
        assert!(erase_stop(RU, &EraseStop::Locked(vec![0])).0.starts_with("Данные (владелец) не расшифрованы,"));
        assert_eq!(erase_stop(RU, &EraseStop::Failed(Part::Art, "x".into())).0, "Не удалось очистить: Кэш ART.");
        assert_eq!(erase_stop(RU, &EraseStop::Failed(Part::Data, "x".into())).0, "Не удалось очистить: раздел data.");
        assert_eq!(erase_stop(RU, &EraseStop::Failed(Part::Logs, "x".into())), ("Не удалось очистить: Журналы.".into(), Some("x".into())));
        assert_eq!(erase_stop(Lang::En, &EraseStop::Failed(Part::Data, "x".into())).0, "Couldn't wipe the data partition.");
        assert_eq!(erase_stop(Lang::En, &EraseStop::Failed(Part::AppData, "x".into())).0, "Couldn't wipe app data.");
    }

    #[test]
    fn work_on_partitions_is_told_in_the_language() {
        let repair = Verb::of(&PartKind::Repair);
        assert_eq!([repair.going(RU), repair.complete(RU), repair.failed(RU), repair.not_done(RU)], ["Идёт проверка", "Проверка завершена", "Не удалось проверить", "Не проверено"]);
        assert_eq!(
            [repair.going(Lang::En), repair.complete(Lang::En), repair.failed(Lang::En), repair.not_done(Lang::En)],
            ["Repairing", "Repair complete", "Repair failed", "Not repaired"]
        );
        assert_eq!(repair.failed_on(RU, "vendor"), "Не удалось проверить vendor");
        assert_eq!(repair.failed_on(Lang::En, "vendor"), "Couldn't repair vendor");
        assert_eq!(Verb::of(&PartKind::Disable).incomplete(Lang::En), "Wipe incomplete");
        assert_eq!(Verb::of(&PartKind::Disable).incomplete(RU), "Очистка не завершена");
        assert_eq!(item_end(Lang::En, &ItemEnd::Failed(None), Verb::of(&PartKind::Remake("f2fs".into()))), "Format failed");
        assert_eq!(item_end(RU, &ItemEnd::Fixed, repair), "Восстановлено");
    }

    #[test]
    fn an_installed_update_says_whether_its_slot_starts_next() {
        let installed = Standing::Installed(Slot::A);
        assert_eq!(update(RU, &installed, Some(Slot::A)).1.unwrap(), "После перезагрузки новая система запустится со слота A и объединит обновление после запуска");
        assert_eq!(
            update(RU, &installed, Some(Slot::B)).1.unwrap(),
            "Перезагрузитесь из «Обновления системы» в системе, чтобы запустить новую систему слота A",
            "the system's updater has not had slot A start next yet"
        );
        assert_eq!(update(RU, &installed, None).1.unwrap(), "Объединится, когда запустится новая система слота A", "the boot control could not tell");
        assert_eq!(update(Lang::En, &installed, Some(Slot::A)).1.unwrap(), "After a reboot the new system starts from slot A, and merges once it has started");
    }

    #[test]
    fn a_speed_is_written_to_three_figures_over_its_unit() {
        let s = |bytes| {
            let (figures, unit) = speed(bytes);
            format!("{figures} {unit}")
        };
        // As the 17 Ultra's status bar showed slow ones.
        assert_eq!(s(0), "0.00 KB/s");
        assert_eq!(s(61), "0.06 KB/s");
        assert_eq!(s(7188), "7.02 KB/s");
        assert_eq!(s(12_800), "12.5 KB/s");
        assert_eq!(s(512_000), "500 KB/s");
        assert_eq!(s(1_023_000), "999 KB/s");
        assert_eq!(s(1_024_000), "0.98 MB/s", "past 999 KB/s the unit grows");
        assert_eq!(s(42_772_090), "40.8 MB/s");
    }

    #[test]
    fn a_network_short_of_the_internet_says_how_and_what_is_left_to_do() {
        use std::net::Ipv4Addr;
        let joined = |reach: Option<Reach>| {
            WlanStage::Connected(skyward_net::wlan::Link {
                address: Ipv4Addr::new(192, 168, 72, 76),
                mask: Ipv4Addr::new(255, 255, 240, 0),
                router: Some(Ipv4Addr::new(192, 168, 64, 254)),
                dns: vec![Ipv4Addr::new(119, 29, 29, 29)],
                signal: -52,
                frequency: 5745,
                speed: None,
                generation: None,
                mac: "02:00:00:00:00:05".into(),
                reach,
            })
        };
        assert_eq!(joining(RU, &joined(None)), "Подключено", "until it has been looked at");
        assert_eq!(joining(RU, &joined(Some(Reach::Online))), "Подключено");
        assert_eq!(joining(RU, &joined(Some(Reach::Login(None)))), "Требуется вход");
        assert_eq!(joining(Lang::En, &joined(Some(Reach::Login(None)))), "Sign-in required");
        assert_eq!(joining(RU, &joined(Some(Reach::Offline("no answer in 5 s".into())))), "Подключено, без интернета");
        assert_eq!(reach(RU, &Reach::Online), None, "the page says nothing of a network that reaches the internet");
        let login = reach(RU, &Reach::Login(Some("http://192.168.64.254:2060/redirect/".into()))).expect("a login is said");
        assert_eq!(
            login,
            "Эта сеть требует входа в браузере, прежде чем получит доступ к интернету, а в Skyward нет браузера для входа. Используйте сеть, не требующую входа, например точку доступа телефона."
        );
        assert_eq!(reach(RU, &Reach::Offline("no answer in 5 s".into())).as_deref(), Some("Эта сеть не имеет доступа в интернет: no answer in 5 s"));
        assert!(unanswered(RU, &Unanswered::Login).starts_with("Не удалось проверить обновления. Сеть Wi-Fi, к которой выполнено подключение, требует входа"));
        assert!(unanswered(Lang::En, &Unanswered::Login).starts_with("Couldn't check for updates. The Wi-Fi network joined needs a sign-in in a browser"));
        assert_eq!(
            unanswered(RU, &Unanswered::Offline("could not reach connect.rom.miui.com".into())),
            "Не удалось проверить обновления. Сеть Wi-Fi, к которой выполнено подключение, не имеет доступа к интернету: could not reach connect.rom.miui.com"
        );
        assert_eq!(
            unanswered(RU, &Unanswered::Failed("io: received fatal alert: HandshakeFailure".into())),
            "Не удалось проверить обновления: io: received fatal alert: HandshakeFailure",
            "a network that reaches the internet leaves how asking failed"
        );
    }

    #[test]
    fn a_shell_tells_how_it_ended() {
        assert_eq!(term_end(RU, &TermEnd::Exited(0)), "Оболочка завершилась с кодом 0");
        assert_eq!(term_end(RU, &TermEnd::Signalled(9)), "Сигнал 9 остановил оболочку");
        assert_eq!(term_end(Lang::En, &TermEnd::Exited(127)), "The shell exited with code 127");
    }
}
