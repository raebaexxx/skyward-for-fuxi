use crate::app::Screen;
use crate::lang::{Lang, Text};

/// A feature or partition the search finds, and the page it opens.
pub struct Entry {
    pub title: Text,
    /// Where it lives, shown under the title.
    pub place: Text,
    pub screen: Screen,
    /// What it answers to besides its title: English and Russian words the
    /// keyboard types.
    pub keys: &'static [&'static str],
}

const fn entry(title: Text, place: Text, screen: Screen, keys: &'static [&'static str]) -> Entry {
    Entry { title, place, screen, keys }
}

/// A partition, named the same in every language.
const fn partition(name: &'static str, place: Text, screen: Screen) -> Entry {
    Entry { title: [name; 2], place, screen, keys: &[] }
}

const HOME: Text = ["Home", "Главная"];
const INSTALL: Text = ["Install", "Установка"];
const ADVANCED: Text = ["Advanced", "Дополнительно"];
const SETTINGS: Text = ["Settings", "Настройки"];
const MOUNT: Text = ["Mount", "Монтирование"];

pub const ENTRIES: &[Entry] = &[
    entry(["Wi-Fi", "Wi-Fi"], HOME, Screen::Wlan, &["wifi", "wlan", "вайфай", "вай-фай", "сеть", "сети"]),
    entry(
        ["Dual boot", "Вторая система"],
        HOME,
        Screen::Systems,
        &["dual boot", "dualboot", "second system", "multiboot", "вторая система", "второй системы", "две системы"],
    ),
    entry(INSTALL, HOME, Screen::Install, &["install", "установка", "установить", "прошивка", "прошить"]),
    entry(
        ["Install from internal storage", "Установка из внутренней памяти"],
        INSTALL,
        Screen::Install,
        &["internal storage", "sdcard", "zip", "внутренняя память", "внутренней памяти", "карта"],
    ),
    entry(
        ["Install from a USB drive", "Установка с USB-накопителя"],
        INSTALL,
        Screen::Install,
        &["usb", "otg", "drive", "флешка", "usb-накопитель", "накопителя"],
    ),
    entry(
        ["Install from a computer", "Установка с компьютера"],
        INSTALL,
        Screen::Install,
        &["computer", "sideload", "adb", "компьютер", "компьютера"],
    ),
    entry(["Backup", "Резервная копия"], HOME, Screen::Backup, &["backup", "резервная копия", "бэкап", "копия"]),
    entry(["Restore", "Восстановить"], HOME, Screen::Restore, &["restore", "восстановить", "восстановление"]),
    entry(MOUNT, HOME, Screen::Mount, &["mount", "umount", "монтирование", "смонтировать", "отмонтировать"]),
    entry(
        ["Wipe data", "Очистить данные"],
        HOME,
        Screen::Wipe,
        &["wipe", "format", "очистить", "очистка", "данные", "формат"],
    ),
    entry(ADVANCED, HOME, Screen::Advanced, &["advanced", "дополнительно"]),
    entry(
        ["File manager", "Файловый менеджер"],
        ADVANCED,
        Screen::Files,
        &["files", "file", "файлы", "файловый менеджер", "менеджер"],
    ),
    entry(["Terminal", "Терминал"], ADVANCED, Screen::Terminal, &["terminal", "shell", "терминал", "оболочка"]),
    entry(
        ["Fix SELinux contexts", "Исправить контексты SELinux"],
        ADVANCED,
        Screen::Relabel,
        &["selinux", "context", "restorecon", "label", "контекст", "контексты"],
    ),
    entry(
        ["Filesystems", "Файловые системы"],
        ADVANCED,
        Screen::Filesystems,
        &["filesystem", "fsck", "f2fs", "ext4", "файловая система", "файловые системы"],
    ),
    entry(
        ["Export logs", "Экспорт журналов"],
        ADVANCED,
        Screen::Logs,
        &["logs", "dmesg", "журнал", "журналы", "логи", "экспорт"],
    ),
    entry(
        ["System update", "Обновление системы"],
        ADVANCED,
        Screen::Update,
        &["update", "ota", "merge", "snapshot", "download", "обновление", "объединить", "загрузка"],
    ),
    entry(SETTINGS, HOME, Screen::Settings, &["settings", "настройки"]),
    entry(["Dark mode", "Тёмная тема"], SETTINGS, Screen::Settings, &["dark", "theme", "тёмная тема", "темная тема"]),
    entry(
        ["Remember the screen lock", "Запоминать пароль экрана"],
        SETTINGS,
        Screen::Settings,
        &["remember", "password", "lock", "запоминать", "пароль", "блокировка"],
    ),
    entry(
        ["Screen brightness", "Яркость экрана"],
        SETTINGS,
        Screen::Settings,
        &["brightness", "яркость", "экрана"],
    ),
    entry(
        ["Screen timeout", "Отключение экрана"],
        SETTINGS,
        Screen::ScreenOff,
        &["timeout", "отключение", "экрана", "тайм-аут"],
    ),
    entry(
        ["Vibration intensity", "Сила вибрации"],
        SETTINGS,
        Screen::Settings,
        &["intensity", "strength", "вибрация", "сила", "интенсивность"],
    ),
    entry(
        ["Touch vibration", "Вибрация при касании"],
        SETTINGS,
        Screen::Settings,
        &["vibrate", "haptic", "вибрация", "касание"],
    ),
    entry(
        ["File transfer", "Передача файлов"],
        SETTINGS,
        Screen::Settings,
        &["mtp", "usb", "передача файлов", "передача"],
    ),
    entry(
        ["Wireless transfer", "Беспроводная передача"],
        ADVANCED,
        Screen::Wireless,
        &["wireless", "http", "web", "беспроводная передача", "беспроводная"],
    ),
    entry(
        ["Download by link", "Загрузка по ссылке"],
        ADVANCED,
        Screen::Fetch,
        &["download", "url", "https", "загрузка", "ссылка", "ссылка"],
    ),
    entry(
        ["File order", "Сортировка файлов"],
        ADVANCED,
        Screen::Sort,
        &["sort", "order", "сортировка", "порядок", "файлов"],
    ),
    entry(
        ["Language", "Язык"],
        SETTINGS,
        Screen::Language,
        &["language", "english", "russian", "язык", "русский", "английский"],
    ),
    entry(["Time zone", "Часовой пояс"], SETTINGS, Screen::TimeZone, &["timezone", "time zone", "часовой пояс", "время"]),
    entry(
        ["Reboot", "Перезагрузка"],
        HOME,
        Screen::Reboot,
        &["reboot", "fastboot", "bootloader", "edl", "перезагрузка", "перезагрузить", "выключить"],
    ),
    entry(
        ["Boot slot", "Загрузочный слот"],
        ["Reboot", "Перезагрузка"],
        Screen::Slot,
        &["slot", "boot", "слот", "загрузочный"],
    ),
    entry(
        ["About phone", "О телефоне"],
        SETTINGS,
        Screen::About,
        &["about", "version", "о телефоне", "версия"],
    ),
    entry(
        ["Skyward update", "Обновление Skyward"],
        SETTINGS,
        Screen::SkywardUpdate,
        &["skyward", "update", "upgrade", "release", "github", "обновление"],
    ),
    entry(
        ["All specs", "Все характеристики"],
        ["Settings > About phone", "Настройки > О телефоне"],
        Screen::Specs,
        &["specs", "характеристики", "параметры"],
    ),
    entry(
        ["Open source licenses", "Лицензии открытого кода"],
        ["Settings > About phone > All specs", "Настройки > О телефоне > Все характеристики"],
        Screen::License,
        &["license", "gpl", "лицензии", "лицензия"],
    ),
    entry(
        ["UI fonts", "Шрифты интерфейса"],
        ["Settings > About phone > All specs", "Настройки > О телефоне > Все характеристики"],
        Screen::FontLicense,
        &["font", "misans", "kirkmono", "шрифт", "шрифты"],
    ),
    partition("boot", INSTALL, Screen::Install),
    partition("init_boot", INSTALL, Screen::Install),
    partition("vendor_boot", INSTALL, Screen::Install),
    partition("recovery", INSTALL, Screen::Install),
    partition("dtbo", INSTALL, Screen::Install),
    partition("vbmeta", INSTALL, Screen::Install),
    partition("system", MOUNT, Screen::Mount),
    partition("system_ext", MOUNT, Screen::Mount),
    partition("product", MOUNT, Screen::Mount),
    partition("vendor", MOUNT, Screen::Mount),
    partition("odm", MOUNT, Screen::Mount),
    partition("vendor_dlkm", MOUNT, Screen::Mount),
    partition("system_dlkm", MOUNT, Screen::Mount),
    partition("odm_dlkm", MOUNT, Screen::Mount),
];

/// The entries `query` finds, best first: a title in `l` holding it, then a
/// key starting with it, then a key holding it.
pub fn find(query: &str, l: Lang) -> Vec<&'static Entry> {
    let q = query.trim().to_lowercase();
    if q.is_empty() {
        return Vec::new();
    }
    let rank = |e: &Entry| {
        if l.pick(e.title).to_lowercase().contains(&q) {
            Some(0)
        } else if e.keys.iter().any(|k| k.starts_with(&q)) {
            Some(1)
        } else if e.keys.iter().any(|k| k.contains(&q)) {
            Some(2)
        } else {
            None
        }
    };
    let mut found: Vec<(u8, usize, &Entry)> = ENTRIES.iter().enumerate().filter_map(|(k, e)| rank(e).map(|r| (r, k, e))).collect();
    found.sort_by_key(|&(r, k, _)| (r, k));
    found.into_iter().map(|(_, _, e)| e).collect()
}

/// What [`find`] finds that the phone has, as `offers` tells of each page:
/// a phone whose WLAN the recovery does not bring up has no WLAN page.
pub fn find_on(query: &str, l: Lang, offers: impl Fn(Screen) -> bool) -> Vec<&'static Entry> {
    find(query, l).into_iter().filter(|e| offers(e.screen)).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn titles(query: &str, l: Lang) -> Vec<&'static str> {
        find(query, l).iter().map(|e| l.pick(e.title)).collect()
    }

    #[test]
    fn english_and_russian_words_find_a_feature() {
        assert_eq!(titles("backup", Lang::En), ["Backup"]);
        assert_eq!(titles("резервная", Lang::Ru), ["Резервная копия"]);
        assert_eq!(titles("Backup", Lang::Ru), ["Резервная копия"], "the English key works in Russian too");
        assert_eq!(titles("яркость", Lang::Ru), ["Яркость экрана"]);
        assert_eq!(titles("brightness", Lang::Ru), ["Яркость экрана"]);
        assert_eq!(titles("язык", Lang::Ru), ["Язык"]);
    }

    #[test]
    fn a_title_holding_the_query_comes_before_keys_that_do() {
        let found = titles("boot", Lang::En);
        assert_eq!(&found[..3], ["Dual boot", "Reboot", "Boot slot"], "a title holding the query comes first");
        assert!(found.contains(&"vendor_boot"), "{found:?}");
        assert!(found.contains(&"Reboot"), "{found:?}");
    }

    #[test]
    fn a_russian_title_matches_as_typed() {
        assert_eq!(titles("монтирование", Lang::Ru), ["Монтирование"]);
        assert_eq!(titles("файловые", Lang::Ru), ["Файловые системы"]);
    }

    #[test]
    fn the_wlan_is_found_where_the_phone_has_it() {
        assert_eq!(titles("wifi", Lang::En), ["Wi-Fi"]);
        assert_eq!(find_on("wlan", Lang::En, |_| true).iter().map(|e| e.title[0]).collect::<Vec<_>>(), ["Wi-Fi"]);
        assert!(find_on("wlan", Lang::En, |s| s != Screen::Wlan).is_empty());
        assert_eq!(titles("загрузка", Lang::Ru), ["Загрузка по ссылке", "Перезагрузка", "Обновление системы"]);
    }

    #[test]
    fn the_second_system_is_found_where_the_phone_can_hold_one() {
        assert_eq!(titles("dual", Lang::En), ["Dual boot"]);
        assert_eq!(find("вторая", Lang::Ru).iter().map(|e| e.title[1]).collect::<Vec<_>>(), ["Вторая система"]);
        assert!(find_on("multiboot", Lang::En, |s| s != Screen::Systems).is_empty());
        assert!(find_on("вторая", Lang::Ru, |s| s != Screen::Systems).is_empty());
    }

    #[test]
    fn skyward_is_updated_from_its_own_page() {
        assert_eq!(titles("skyward", Lang::En), ["Skyward update"]);
        assert_eq!(titles("обновление", Lang::Ru), ["Обновление системы", "Обновление Skyward"]);
        assert!(find_on("skyward", Lang::En, |s| s != Screen::SkywardUpdate).is_empty());
    }

    #[test]
    fn a_title_is_found_in_the_language_the_pages_speak() {
        let english = |query| find(query, Lang::En).iter().map(|e| e.title[0]).collect::<Vec<_>>();
        assert_eq!(english("brightness"), ["Screen brightness"]);
        assert_eq!(english("about"), ["About phone"]);
        assert_eq!(english("wi-fi"), ["Wi-Fi"]);
        let russian = find("настройки", Lang::Ru);
        assert_eq!(russian.first().map(|e| e.title[1]), Some("Настройки"));
        assert!(find("Настройки", Lang::Ru).iter().all(|e| e.title[1] == "Настройки"), "the Russian title belongs to the Russian pages");
    }

    #[test]
    fn nothing_typed_finds_nothing() {
        assert!(find("", Lang::En).is_empty());
        assert!(find("   ", Lang::Ru).is_empty());
        assert!(find("zzzz", Lang::Ru).is_empty());
    }
}
