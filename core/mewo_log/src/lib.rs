use mewo_ecs::{debug_request_dump, DebugDumpHook, DebugDumpTargets, DebugLogHook, DebugMessage};
use std::io::Write;

fn log_hook_msg(msg: &DebugMessage) -> String {
    let mut out = format!(
        "
        ---   Dump Start   ---\n
        Requested {:?}\n
        ",
        msg.dumps
    );
    for &dump_target in msg.dumps.iter() {
        let dump = debug_request_dump(dump_target).unwrap_or("Unavaliable".to_string());
        out += &format!("Dumping `{:?}` >>\n{}\n<<", dump_target, dump);
    }
    out += &format!(
        "
        ---   Dump End   ---\n
        {} Error: {:?}\n
        Line {} @ {}\n",
        msg.level.to_string(),
        msg.msg,
        msg.line,
        msg.file
    );
    out
}

pub fn log_hook_stderr() -> DebugLogHook {
    Box::new(|msg| eprintln!("{}", log_hook_msg(msg)))
}

pub fn log_hook_file(location: Option<String>) -> DebugLogHook {
    Box::new(move |msg| {
        let default_location =
            std::env::temp_dir().to_str().unwrap().to_string() + "/mewotk_log.log";
        let location = location.as_ref().unwrap_or(&default_location);
        let mut file = if let Ok(file) = std::fs::OpenOptions::new()
            .write(true)
            .append(true)
            .open(location)
        {
            file
        } else {
            return;
        };
        let msg = log_hook_msg(msg);
        let _ = file.write_all(msg.as_bytes());
    })
}

#[derive(Clone)]
pub struct DumpTargetMask(Vec<DebugDumpTargets>);

fn dump_hook_msg(target: DebugDumpTargets) -> String {
    let dump = debug_request_dump(target).unwrap_or("Unavaliable".to_string());
    format!("Dumping `{:?}` >>\n{}\n<<\n\n", target, dump)
}

pub fn dump_hook_stderr() -> DebugDumpHook {
    Box::new(|target| {
        eprintln!("{}", dump_hook_msg(target));
    })
}

pub fn dump_hook_file(location: Option<String>, mask: Option<DumpTargetMask>) -> DebugDumpHook {
    Box::new(move |target| {
        let default_mask = DumpTargetMask(vec![]);
        let mask = mask.as_ref().unwrap_or(&default_mask);
        if mask.0.iter().position(|&p| p == target).is_some() {
            return;
        }
        let default_location =
            std::env::temp_dir().to_str().unwrap().to_string() + "/mewotk_dump.log";
        let location = location.as_ref().unwrap_or(&default_location);
        let mut file = if let Ok(file) = std::fs::OpenOptions::new()
            .append(true)
            .create(true)
            .open(location)
        {
            file
        } else {
            return;
        };
        let msg = dump_hook_msg(target);
        let _ = file.write_all(msg.as_bytes());
    })
}
