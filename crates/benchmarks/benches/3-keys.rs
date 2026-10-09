#![allow(elided_lifetimes_in_paths)]

#[cfg(feature = "alloc-profiler")]
#[global_allocator]
static ALLOC: divan::AllocProfiler = divan::AllocProfiler::system();

const NUM_ENTRIES: &[usize] = &[10, 100];

mod toml_edit {
    use crate::NUM_ENTRIES;

    #[divan::bench(args = NUM_ENTRIES)]
    fn dump(bencher: divan::Bencher, entries: usize) {
        let mut document = ::toml_edit::DocumentMut::new();
        for i in 0..entries {
            let key = if i % 2 == 0 {
                format!("key_{i}")
            } else {
                format!("key {i}")
            };
            let value = match i % 4 {
                0 => ::toml_edit::Value::from("a generated string"),
                1 => ::toml_edit::Value::from(i64::try_from(i).unwrap()),
                2 => ::toml_edit::Value::from(true),
                _ => ::toml_edit::Value::from(1.25),
            };
            document.insert(&key, ::toml_edit::Item::Value(value));
        }
        bencher.bench(|| std::hint::black_box(&document).to_string());
    }
}

fn main() {
    divan::main();
}
