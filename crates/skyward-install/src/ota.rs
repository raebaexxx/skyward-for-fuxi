//! update_engine_sideload, the phone's own program that applies an A/B
//! update package: how it is started, and what it reports while it works.

use crate::package::Update;

/// Where the recovery ramdisk keeps it.
pub const PROGRAM: &str = "/system/bin/update_engine_sideload";

/// Its arguments for `update`, read from the file `package`; it writes
/// its status lines to `status_fd`.
pub fn arguments(package: &str, update: &Update, status_fd: i32) -> Vec<String> {
    vec![
        format!("--payload=file://{package}"),
        format!("--offset={}", update.payload_offset),
        format!("--headers={}", update.properties),
        format!("--status_fd={status_fd}"),
    ]
}

/// A line it writes to the status descriptor that matters here.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Status {
    /// Step 1 writes the partitions; step 2 runs the package's last steps,
    /// after the written partitions are read back.
    Step(u32),
    /// It failed with this error code.
    Error(i32),
}

pub fn status(line: &str) -> Option<Status> {
    let text = line.strip_prefix("ui_print ")?;
    if let Some(step) = text.strip_prefix("Step ") {
        return step.split_once('/')?.0.parse().ok().map(Status::Step);
    }
    let code = text.strip_prefix("Error applying update: ")?;
    code.split_whitespace().next()?.parse().ok().map(Status::Error)
}

/// What its log on stderr tells beyond the status lines.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Log {
    /// A partition the payload writes, listed once before writing starts.
    Partition(String),
    /// Super has no room for snapshots, so it writes over the partitions
    /// of the slot the phone runs from.
    Overwriting,
    /// It started reading back the `index`th partition it wrote, from 0.
    Hashing { index: usize, partition: String },
    /// The share of the payload it has taken so far, in whole percent, as
    /// it says every ten percent.
    Progress(u32),
}

pub fn log(line: &str) -> Option<Log> {
    if let Some(rest) = after(line, "overall progress ") {
        if let Some(number) = rest.split('%').next() {
            if let Ok(at) = number.trim().parse::<u32>() {
                return Some(Log::Progress(at.min(100)));
            }
        }
    }
    if let Some(rest) = after(line, "PartitionInfo new ") {
        let (name, _) = rest.split_once(' ')?;
        return Some(Log::Partition(name.to_string()));
    }
    if line.contains("Will overwrite existing partitions.") {
        return Some(Log::Overwriting);
    }
    let rest = after(line, "Hashing partition ")?;
    let (index, rest) = rest.split_once(" (")?;
    let (name, _) = rest.split_once(')')?;
    Some(Log::Hashing { index: index.parse().ok()?, partition: name.to_string() })
}

fn after<'a>(line: &'a str, marker: &str) -> Option<&'a str> {
    line.find(marker).map(|at| &line[at + marker.len()..])
}

/// How many payload bytes the writing step has behind it: the bytes read so
/// far through the package's filesystem, and no more than ten percent beyond
/// what update_engine itself says it has taken. Reads reach ahead of the
/// writing — the zip's table at the end of the file is read before any of
/// the payload is written — so the furthest read alone would show the step
/// done before it began.
pub fn written(served: u64, offset: u64, total: u64, reported: Option<u32>) -> u64 {
    if total == 0 {
        return 0;
    }
    let read = served.saturating_sub(offset).min(total);
    let Some(at) = reported else { return read };
    let floor = total.saturating_mul(u64::from(at.min(100))) / 100;
    let ceiling = total.saturating_mul(u64::from(at.saturating_add(10).min(100))) / 100;
    read.clamp(floor, ceiling)
}

/// What an update_engine error code comes down to for whoever installs.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Trouble {
    /// The package is not signed with the keys this phone trusts.
    Signature,
    /// The package is damaged.
    Damaged,
    /// It is older than what the recovery allows.
    Older,
    /// Super has no room for the new partitions.
    NoSpace,
    /// Reading the package failed: the computer stopped sending.
    Transfer,
    /// Writing a partition failed.
    Write,
    /// The written partitions read back wrong.
    Verify,
    /// The package's own last step failed.
    PostInstall,
    /// The package is in a format this update_engine does not know.
    Unsupported,
    Other,
}

impl Trouble {
    pub fn of(code: i32) -> Trouble {
        match code {
            12 | 18 | 22 | 24 | 25 | 26 | 33 | 39 => Trouble::Signature,
            10 | 11 | 21 | 23 | 27 | 28 | 29 | 32 | 38 => Trouble::Damaged,
            51 => Trouble::Older,
            60 => Trouble::NoSpace,
            9 => Trouble::Transfer,
            7 | 14 | 20 => Trouble::Write,
            15 | 16 | 47 | 56 => Trouble::Verify,
            5 | 63 => Trouble::PostInstall,
            6 | 17 | 44 | 45 => Trouble::Unsupported,
            _ => Trouble::Other,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::package::Metadata;

    #[test]
    fn the_command_line_is_the_one_recovery_builds() {
        let update = Update {
            payload_offset: 2365,
            payload_size: 7_103_041_280,
            properties: "FILE_HASH=abc=\nFILE_SIZE=7103041280\n".into(),
            metadata: Metadata::default(),
        };
        assert_eq!(
            arguments("/sideload/package.zip", &update, 9),
            [
                "--payload=file:///sideload/package.zip",
                "--offset=2365",
                "--headers=FILE_HASH=abc=\nFILE_SIZE=7103041280\n",
                "--status_fd=9",
            ]
        );
    }

    #[test]
    fn status_lines_give_steps_and_errors() {
        assert_eq!(status("ui_print Step 1/2"), Some(Status::Step(1)));
        assert_eq!(status("ui_print Step 2/2"), Some(Status::Step(2)));
        assert_eq!(
            status("ui_print Error applying update: 51 (ErrorCode::kPayloadTimestampError)"),
            Some(Status::Error(51))
        );
        assert_eq!(status("progress 0.5 0"), None);
        assert_eq!(status("set_progress 1"), None);
    }

    #[test]
    fn the_log_names_partitions_and_the_one_being_read_back() {
        assert_eq!(
            log("[INFO:delta_performer.cc(267)] PartitionInfo new vendor_boot sha256: 3f8a size: 100663296"),
            Some(Log::Partition("vendor_boot".into()))
        );
        assert_eq!(
            log("[INFO:filesystem_verifier_action.cc(377)] Hashing partition 3 (odm) on device /dev/block/dm-12"),
            Some(Log::Hashing { index: 3, partition: "odm".into() })
        );
        assert_eq!(
            log("[INFO:dynamic_partition_control_android.cc(1349)] Will overwrite existing partitions. Slot B may be unbootable until update finishes!"),
            Some(Log::Overwriting)
        );
        assert_eq!(log("[INFO:delta_performer.cc(267)] PartitionInfo old odm sha256: 00 size: 4"), None);
        assert_eq!(log("[INFO:delta_performer.cc(108)] Completed 10/400 operations (2%)"), None);
    }

    #[test]
    fn a_progress_line_gives_the_share_taken() {
        assert_eq!(
            log("[INFO:delta_performer.cc(120)] Completed 394/3496 operations (11%), 251101184/2510975561 bytes downloaded (10%), overall progress 10%"),
            Some(Log::Progress(10))
        );
        assert_eq!(log("[INFO:delta_performer.cc(120)] overall progress 100%"), Some(Log::Progress(100)));
        assert_eq!(log("overall progress 140%"), Some(Log::Progress(100)), "a share stops at all of it");
    }

    #[test]
    fn the_writing_share_follows_the_engine_and_not_the_furthest_read() {
        let (offset, total) = (4914, 2_510_975_561u64);
        // The zip's own table is read at the end before the writing starts:
        // the furthest read is the whole file, the share is not.
        assert_eq!(written(total + 4914, offset, total, Some(0)), total * 10 / 100);
        assert_eq!(written(total + 4914, offset, total, Some(40)), total * 50 / 100);
        // A read that runs on smoothly leads the engine by less than a step.
        assert_eq!(written(offset + total * 43 / 100, offset, total, Some(40)), total * 43 / 100);
        // The engine's word is a floor of its own.
        assert_eq!(written(offset, offset, total, Some(70)), total * 70 / 100);
        // With nothing said at all, the furthest read is all there is.
        assert_eq!(written(offset + total / 2, offset, total, None), total / 2);
        assert_eq!(written(offset, offset, 0, Some(10)), 0);
    }

    #[test]
    fn error_codes_come_down_to_their_trouble() {
        assert_eq!(Trouble::of(26), Trouble::Signature);
        assert_eq!(Trouble::of(51), Trouble::Older);
        assert_eq!(Trouble::of(9), Trouble::Transfer);
        assert_eq!(Trouble::of(60), Trouble::NoSpace);
        assert_eq!(Trouble::of(1), Trouble::Other);
    }
}
