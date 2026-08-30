use std::thread;

use super::ui;
use crate::muxi::{Plugin, Settings};
use miette::Result;

pub fn init() -> Result<()> {
    let settings = Settings::from_lua()?;
    let plugins = settings.plugins;

    if plugins.is_empty() {
        return Ok(());
    }

    let errors = if settings.parallel_plugin_loading {
        run_parallel(plugins, Plugin::source)
    } else {
        run_sequential(plugins, Plugin::source)
    };

    if errors.is_empty() {
        Ok(())
    } else {
        Err(ui::format_plugin_errors(&errors, "source"))
    }
}

fn run_sequential<T, E, F>(items: Vec<T>, operation: F) -> Vec<(T, E)>
where
    F: Fn(&T) -> Result<(), E>,
{
    items
        .into_iter()
        .filter_map(|item| operation(&item).err().map(|error| (item, error)))
        .collect()
}

fn run_parallel<T, E, F>(items: Vec<T>, operation: F) -> Vec<(T, E)>
where
    T: Send,
    E: Send,
    F: Fn(&T) -> Result<(), E> + Sync,
{
    thread::scope(|scope| {
        let handles = items
            .into_iter()
            .map(|item| {
                let operation = &operation;
                scope.spawn(move || {
                    let result = operation(&item);
                    (item, result)
                })
            })
            .collect::<Vec<_>>();

        handles
            .into_iter()
            .filter_map(|handle| {
                let (item, result) = handle.join().unwrap();
                result.err().map(|error| (item, error))
            })
            .collect()
    })
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Condvar, Mutex};
    use std::thread;
    use std::time::Duration;

    use super::{run_parallel, run_sequential};

    #[test]
    fn sequential_loading_preserves_order_and_collects_errors() {
        let calls = Mutex::new(Vec::new());
        let errors = run_sequential(vec![1, 2, 3], |item| {
            calls.lock().unwrap().push(*item);

            if item % 2 == 0 { Ok(()) } else { Err(*item) }
        });

        assert_eq!(*calls.lock().unwrap(), vec![1, 2, 3]);
        assert_eq!(errors, vec![(1, 1), (3, 3)]);
    }

    #[test]
    fn parallel_loading_overlaps_operations() {
        #[derive(Default)]
        struct State {
            release: bool,
            started: usize,
        }

        let state = Arc::new((Mutex::new(State::default()), Condvar::new()));
        let operation_state = Arc::clone(&state);

        let runner = thread::spawn(move || {
            run_parallel(vec![1, 2], move |_| {
                let (lock, ready) = &*operation_state;
                let mut state = lock.lock().unwrap();
                state.started += 1;
                ready.notify_all();

                while !state.release {
                    state = ready.wait(state).unwrap();
                }

                Ok::<(), ()>(())
            })
        });

        let (lock, ready) = &*state;
        let state_guard = lock.lock().unwrap();
        let (mut state_guard, timeout) = ready
            .wait_timeout_while(state_guard, Duration::from_secs(1), |state| {
                state.started < 2
            })
            .unwrap();

        let started = state_guard.started;
        state_guard.release = true;
        ready.notify_all();
        drop(state_guard);

        let errors = runner.join().unwrap();

        assert!(!timeout.timed_out(), "operations did not overlap");
        assert_eq!(started, 2);
        assert!(errors.is_empty());
    }

    #[test]
    fn parallel_loading_reports_errors_in_configuration_order() {
        let errors = run_parallel(vec![1, 2, 3], |item| Err::<(), _>(*item));

        assert_eq!(errors, vec![(1, 1), (2, 2), (3, 3)]);
    }
}
