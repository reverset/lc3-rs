pub fn get_position(args: &[&str], long: &str, short: Option<&str>) -> Option<usize> {
    args.iter()
        .enumerate()
        .find(|(_, v)| {
            let v = v.trim();
            if v == format!("--{long}").as_str() {
                true
            } else if let Some(short) = short {
                v == format!("-{short}").as_str()
            } else {
                false
            }
        })
        .map(|(i, _)| i)
}

pub fn get_flag<'a>(args: &[&str], long: &str, short: impl Into<Option<&'a str>>) -> bool {
    if args.contains(&format!("--{long}").as_str()) {
        true
    } else if let Some(short) = short.into() {
        args.contains(&format!("-{short}").as_str())
    } else {
        false
    }
}

pub fn get_param<'a>(
    args: &[&str],
    long: &str,
    short: impl Into<Option<&'a str>>,
) -> Option<String> {
    let short = short.into();
    if let Some(pos) = get_position(args, long, short) {
        if pos + 1 >= args.len() {
            panic!("--{long}/-{short:?} requires an input after.");
        }
        Some(args[pos + 1].to_string())
    } else {
        None
    }
}
