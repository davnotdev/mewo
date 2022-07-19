use mewo_ecs::{debug_request_dump, DebugDumpTargets, DebugMessage};
use std::io::Write;

pub trait OutputFileLocation {
    fn location() -> String;
}

pub struct OutputFileLocationHere;
pub struct OutputFileLocationTemp;

impl OutputFileLocation for OutputFileLocationHere {
    fn location() -> String {
        "./mewotk.log".to_string()
    }
}

impl OutputFileLocation for OutputFileLocationTemp {
    fn location() -> String {
        std::env::temp_dir().to_str().unwrap().to_string() + "/mewotk.log"
    }
}

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

pub fn log_hook_stderr(msg: &DebugMessage) {
    eprintln!("{}", log_hook_msg(msg))
}

pub fn log_hook_file(msg: &DebugMessage) {
    log_hook_file_generic::<OutputFileLocationTemp>(msg)
}

pub fn log_hook_file_generic<OL: OutputFileLocation>(msg: &DebugMessage) {
    let mut file = if let Ok(file) = std::fs::OpenOptions::new()
        .write(true)
        .append(true)
        .open(OL::location())
    {
        file
    } else {
        return;
    };
    let msg = log_hook_msg(msg);
    let _ = file.write_all(msg.as_bytes());
}

fn dump_hook_msg(target: DebugDumpTargets) -> String {
    let dump = debug_request_dump(target).unwrap_or("Unavaliable".to_string());
    format!("Dumping `{:?}` >>\n{}\n<<\n\n", target, dump)
}

pub fn dump_hook_stderr(target: DebugDumpTargets) {
    eprintln!("{}", dump_hook_msg(target));
}

pub fn dump_hook_file(target: DebugDumpTargets) {
    dump_hook_file_generic::<OutputFileLocationTemp>(target)
}

pub fn dump_hook_file_generic<OL: OutputFileLocation>(target: DebugDumpTargets) {
    let mut file = if let Ok(file) = std::fs::OpenOptions::new()
        .append(true)
        .create(true)
        .open(OL::location())
    {
        file
    } else {
        return;
    };
    let msg = dump_hook_msg(target);
    let _ = file.write_all(msg.as_bytes());
}
