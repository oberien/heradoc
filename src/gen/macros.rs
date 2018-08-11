macro_rules! read_until {
    ($gen:expr, $events:expr, $pat:pat) => {{
        let mut text = Vec::new();
        loop {
            match $events.next().unwrap() {
                Event::End($pat) => break,
                evt => $gen.visit_event(evt, $events, &mut text)?,
            }
        }
        // taken from take_mut and modified to allow forwarding errors
        String::from_utf8(text).expect("invalid UTF-8")
    }}
}

macro_rules! handle_until {
    ($gen:expr, $events:expr, $out:expr, $pat:pat) => {
        loop {
            match $events.next().unwrap() {
                Event::End($pat) => break,
                evt => $gen.visit_event(evt, $events, $out)?,
            }
        }
    }
}

