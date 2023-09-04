use crate::*;

#[test]
fn test_log() {
    let sub = { Logger::get_global_logger().write().subscribe() };
    {
        let _fold = mfold!("f");
        minfo!("Got Some Value! {}", 10);
        minfo!("mmm");
    }
    let join = std::thread::spawn(|| {
        let _fold = mfold!("Loop Instance");
        minfo!("Calling from another thread!");
    });

    join.join().unwrap();

    #[derive(Debug, PartialEq, Eq)]
    enum LogRes {
        FoldStart,
        Record,
        FoldEnd,
    }

    let expected = vec![
        LogRes::FoldStart,
        LogRes::Record,
        LogRes::Record,
        LogRes::FoldEnd,
        LogRes::FoldStart,
        LogRes::Record,
        LogRes::FoldEnd,
    ];
    let mut res = vec![];

    let mut logger = Logger::get_global_logger().write();
    while let Some(record) = logger.take(sub) {
        res.push(match record {
            LogEvent::FoldStart(_) => LogRes::FoldStart,
            LogEvent::Record(_) => LogRes::Record,
            LogEvent::FoldEnd => LogRes::FoldEnd,
        });
    }
    assert_eq!(res, expected);
}
