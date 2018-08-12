macro_rules! read_until {
    ($gen:expr, $state:expr, $pat:pat) => {{
        let mut text = Vec::new();
        // taken from take_mut and modified to allow forwarding errors
        unsafe {
            let old_state = ::std::ptr::read($state);
            let (new_state, err) = ::std::panic::catch_unwind(::std::panic::AssertUnwindSafe(|| {
                let State { events, out, stack } = old_state;
                let mut state = State {
                    events,
                    out: &mut text,
                    stack,
                };
                let err = loop {
                    match state.events.next().unwrap() {
                        Event::End($pat) => break None,
                        evt => if let Err(e) = $gen.visit_event(evt, &mut state) {
                            break Some(e)
                        }
                    }
                };
                (State {
                    events: state.events,
                    out,
                    stack: state.stack,
                }, err)
            })).unwrap_or_else(|_| ::std::process::abort());
            ::std::ptr::write($state, new_state);
            match err {
                Some(e) => Err(e)?,
                None => String::from_utf8(text).expect("invalid UTF-8"),
            }
        }
    }}
}

macro_rules! handle_until {
    ($gen:expr, $state:expr, $pat:pat) => {
        loop {
            match $state.events.next().unwrap() {
                Event::End($pat) => break,
                evt => $gen.visit_event(evt, $state)?,
            }
        }
    }
}

